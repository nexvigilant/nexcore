//! # K-D Tree Spatial Index
//!
//! **T1 Components**: λ(Location) × N(Quantity) × ∂(Boundary) × σ(Sequence)
//!
//! A k-d tree is a binary space-partitioning structure that enables
//! sub-linear nearest-neighbour and range queries in K-dimensional
//! Euclidean space. Each internal node splits the remaining point set
//! on one coordinate axis (cycling through dimensions level-by-level);
//! leaves store individual points.
//!
//! ## T1 Grounding
//!
//! | Primitive | Role in k-d tree |
//! |-----------|-----------------|
//! | λ Location | Every point occupies a position; the tree partitions location space |
//! | N Quantity | Distances are measured quantities; `k` in k-nearest is a count |
//! | ∂ Boundary | Split hyperplanes are boundaries; neighbourhood radius is a boundary |
//! | σ Sequence | Tree traversal follows a sequence of split decisions |
//!
//! ## PV Transfer
//!
//! Drug-event spatial clustering: encode each report as a K-dimensional
//! vector (e.g. dose, time-to-onset, severity score) and use
//! [`KdTree::k_nearest`] to find the `k` most similar reports to a query
//! report — enabling signal-neighbourhood analysis without pairwise
//! O(n²) comparison.
//!
//! ## Algorithm Summary
//!
//! - **Build**: sort points by median on the current axis, recurse on
//!   both halves — O(n log n), produces a balanced tree.
//! - **Nearest**: recurse into the half that contains the query; after
//!   returning, check whether the opposing half could harbour a closer
//!   point by comparing the split distance to the best-so-far — O(log n)
//!   amortised.
//! - **k-Nearest**: same pruning, maintain a max-heap of capacity `k`.
//! - **Range**: prune any subtree whose split hyperplane is farther than
//!   the query radius; collect all points whose Euclidean distance is
//!   within the [`Neighborhood`].

use std::cmp::Reverse;
use std::collections::BinaryHeap;

use nexcore_error::Error;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::spatial::{Distance, Neighborhood};

// ============================================================================
// Error type
// ============================================================================

/// Errors produced by k-d tree operations.
///
/// # Examples
///
/// ```rust
/// use stem_math::spatial_index::{KdTree, KdPoint, SpatialIndexError};
///
/// let tree: KdTree<2> = KdTree::new();
/// let query = KdPoint { coords: [0.0, 0.0] };
/// assert_eq!(tree.k_nearest(&query, 0), Err(SpatialIndexError::InvalidK));
/// assert_eq!(tree.k_nearest(&query, 1), Err(SpatialIndexError::EmptyTree));
/// ```
#[derive(Debug, Error, PartialEq, Eq, Clone)]
pub enum SpatialIndexError {
    /// Returned by [`KdTree::k_nearest`] when the tree contains no points.
    #[error("empty tree: no points inserted")]
    EmptyTree,

    /// Returned by [`KdTree::k_nearest`] when `k == 0`.
    #[error("k must be at least 1 for k-nearest search")]
    InvalidK,

    /// Returned when a point has the wrong number of coordinates.
    ///
    /// This variant is reserved for future dynamic-dimension APIs; the
    /// const-generic [`KdPoint<K>`] makes it statically impossible in
    /// the current API.
    #[error("dimension mismatch in point data")]
    DimensionMismatch,
}

// ============================================================================
// Point type
// ============================================================================

/// A point in K-dimensional Euclidean space.
///
/// The coordinate type is `f64`; NaN values produce unspecified but
/// memory-safe behaviour (no panics, no undefined behaviour).
///
/// Serde support is provided via manual `Serialize`/`Deserialize` impls
/// because the derive macro cannot resolve the `[f64; K]: Serialize` bound
/// for const-generic arrays in the serde version used by this workspace.
///
/// # Examples
///
/// ```rust
/// use stem_math::spatial_index::KdPoint;
///
/// let p: KdPoint<3> = KdPoint { coords: [1.0, 2.0, 3.0] };
/// assert_eq!(p.coords[0], 1.0);
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct KdPoint<const K: usize> {
    /// The K coordinate values of this point.
    pub coords: [f64; K],
}

// Manual Serialize: emit as a fixed-length sequence of f64 values.
impl<const K: usize> Serialize for KdPoint<K> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeSeq;
        let mut seq = serializer.serialize_seq(Some(K))?;
        for v in &self.coords {
            seq.serialize_element(v)?;
        }
        seq.end()
    }
}

// Manual Deserialize: read a sequence of exactly K f64 values.
impl<'de, const K: usize> Deserialize<'de> for KdPoint<K> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        use serde::de::{Error, SeqAccess, Visitor};
        use std::fmt;

        struct KdPointVisitor<const N: usize>;

        impl<'de, const N: usize> Visitor<'de> for KdPointVisitor<N> {
            type Value = KdPoint<N>;

            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "a sequence of {N} f64 values")
            }

            fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<KdPoint<N>, A::Error> {
                // Build the array element-by-element.
                let mut coords = [0.0_f64; N];
                for (i, slot) in coords.iter_mut().enumerate() {
                    *slot = seq
                        .next_element()?
                        .ok_or_else(|| A::Error::invalid_length(i, &self))?;
                }
                // Reject any trailing elements.
                if seq.next_element::<serde::de::IgnoredAny>()?.is_some() {
                    return Err(A::Error::invalid_length(N + 1, &self));
                }
                Ok(KdPoint { coords })
            }
        }

        deserializer.deserialize_seq(KdPointVisitor::<K>)
    }
}

impl<const K: usize> KdPoint<K> {
    /// Compute the squared Euclidean distance to `other`.
    ///
    /// Working in squared space avoids a square-root during tree traversal
    /// (we only take the root when producing a [`Distance`] value).
    #[must_use]
    fn squared_distance_to(&self, other: &Self) -> f64 {
        self.coords
            .iter()
            .zip(other.coords.iter())
            .map(|(a, b)| (a - b) * (a - b))
            .sum()
    }

    /// Euclidean distance to `other` as a [`Distance`].
    #[must_use]
    pub fn euclidean_distance_to(&self, other: &Self) -> Distance {
        Distance::new(self.squared_distance_to(other).sqrt())
    }
}

// ============================================================================
// Query result
// ============================================================================

/// The result of a nearest-neighbour query: the matching point and its
/// Euclidean distance from the query location.
///
/// # Examples
///
/// ```rust
/// use stem_math::spatial_index::{KdTree, KdPoint};
///
/// let mut tree: KdTree<2> = KdTree::new();
/// tree.insert(KdPoint { coords: [1.0, 0.0] });
///
/// let query = KdPoint { coords: [0.0, 0.0] };
/// let result = tree.nearest(&query).unwrap();
/// assert_eq!(result.point.coords, [1.0, 0.0]);
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct NearestResult<const K: usize> {
    /// The closest (or one of the closest) points found.
    pub point: KdPoint<K>,
    /// Euclidean distance from the query to `point`.
    pub distance: Distance,
}

// ============================================================================
// Internal tree node
// ============================================================================

/// Internal representation of a k-d tree node.
///
/// An `Leaf` stores a single point. A `Split` stores the splitting point,
/// the axis on which the split was made, and optional children.
#[derive(Debug, Clone)]
enum Node<const K: usize> {
    /// A leaf holding exactly one point.
    Leaf(KdPoint<K>),
    /// An internal split node.
    Split {
        /// The point stored at this node (the "median" chosen during build).
        point: KdPoint<K>,
        /// The coordinate axis used to split (0..K).
        axis: usize,
        /// Left subtree: points where `coord[axis] <= point.coord[axis]`.
        left: Option<Box<Node<K>>>,
        /// Right subtree: points where `coord[axis] > point.coord[axis]`.
        right: Option<Box<Node<K>>>,
    },
}

impl<const K: usize> Node<K> {
    /// Count the total number of points stored below (and at) this node.
    fn _count(&self) -> usize {
        match self {
            Node::Leaf(_) => 1,
            Node::Split { left, right, .. } => {
                1 + left.as_ref().map_or(0, |n| n._count())
                    + right.as_ref().map_or(0, |n| n._count())
            }
        }
    }

    /// Insert a point into the subtree rooted here, cycling axes.
    fn insert(&mut self, new_point: KdPoint<K>, depth: usize) {
        let axis = depth % K.max(1);
        match self {
            Node::Leaf(existing) => {
                // Promote the leaf into a split node.
                let existing = existing.clone();
                let go_right = new_point.coords[axis] > existing.coords[axis];
                let (left_child, right_child) = if go_right {
                    (None, Some(Box::new(Node::Leaf(new_point))))
                } else {
                    (Some(Box::new(Node::Leaf(new_point))), None)
                };
                *self = Node::Split {
                    point: existing,
                    axis,
                    left: left_child,
                    right: right_child,
                };
            }
            Node::Split {
                point,
                axis: split_axis,
                left,
                right,
                ..
            } => {
                let go_right = new_point.coords[*split_axis] > point.coords[*split_axis];
                if go_right {
                    match right {
                        Some(child) => child.insert(new_point, depth + 1),
                        None => *right = Some(Box::new(Node::Leaf(new_point))),
                    }
                } else {
                    match left {
                        Some(child) => child.insert(new_point, depth + 1),
                        None => *left = Some(Box::new(Node::Leaf(new_point))),
                    }
                }
            }
        }
    }

    /// Nearest-neighbour search. Updates `best` in place.
    ///
    /// `best` is `(squared_distance, point)` — using squared distances
    /// throughout avoids repeated square-root calls.
    fn nearest_search(&self, query: &KdPoint<K>, best: &mut Option<(f64, KdPoint<K>)>) {
        match self {
            Node::Leaf(p) => {
                let sq = query.squared_distance_to(p);
                Self::update_best(best, sq, p.clone());
            }
            Node::Split {
                point,
                axis,
                left,
                right,
            } => {
                let sq = query.squared_distance_to(point);
                Self::update_best(best, sq, point.clone());

                // Decide which child to visit first.
                let diff = query.coords[*axis] - point.coords[*axis];
                let (near, far) = if diff <= 0.0 {
                    (left, right)
                } else {
                    (right, left)
                };

                if let Some(near_child) = near {
                    near_child.nearest_search(query, best);
                }

                // Prune far child: only visit if the split plane is
                // closer than the current best.
                let plane_sq = diff * diff;
                let best_sq = best.as_ref().map_or(f64::INFINITY, |b| b.0);
                if plane_sq < best_sq {
                    if let Some(far_child) = far {
                        far_child.nearest_search(query, best);
                    }
                }
            }
        }
    }

    /// k-nearest search using a max-heap of capacity `k`.
    ///
    /// `heap` stores `(Reverse(squared_distance), point)` so that the
    /// farthest element in the heap sits at the top (max-heap by
    /// distance means we can quickly drop the worst candidate).
    fn k_nearest_search(
        &self,
        query: &KdPoint<K>,
        k: usize,
        heap: &mut BinaryHeap<(Reverse<OrderedFloat>, KdPoint<K>)>,
    ) {
        match self {
            Node::Leaf(p) => {
                let sq = query.squared_distance_to(p);
                Self::heap_push(heap, sq, p.clone(), k);
            }
            Node::Split {
                point,
                axis,
                left,
                right,
            } => {
                let sq = query.squared_distance_to(point);
                Self::heap_push(heap, sq, point.clone(), k);

                let diff = query.coords[*axis] - point.coords[*axis];
                let (near, far) = if diff <= 0.0 {
                    (left, right)
                } else {
                    (right, left)
                };

                if let Some(near_child) = near {
                    near_child.k_nearest_search(query, k, heap);
                }

                // Prune: visit far child only if the split plane could
                // improve on our current worst candidate.
                let plane_sq = diff * diff;
                let worst_sq = heap.peek().map_or(f64::INFINITY, |top| (top.0).0.0);
                if heap.len() < k || plane_sq < worst_sq {
                    if let Some(far_child) = far {
                        far_child.k_nearest_search(query, k, heap);
                    }
                }
            }
        }
    }

    /// Range search: collect all points within the given neighborhood.
    fn range_search(
        &self,
        query: &KdPoint<K>,
        neighborhood: &Neighborhood,
        results: &mut Vec<NearestResult<K>>,
    ) {
        match self {
            Node::Leaf(p) => {
                let dist = query.euclidean_distance_to(p);
                if neighborhood.contains(dist) {
                    results.push(NearestResult {
                        point: p.clone(),
                        distance: dist,
                    });
                }
            }
            Node::Split {
                point,
                axis,
                left,
                right,
            } => {
                // Check the point stored at this node.
                let dist = query.euclidean_distance_to(point);
                if neighborhood.contains(dist) {
                    results.push(NearestResult {
                        point: point.clone(),
                        distance: dist,
                    });
                }

                // Axis-aligned distance from query to the split plane.
                let diff = query.coords[*axis] - point.coords[*axis];
                let plane_dist = Distance::new(diff.abs());

                // Visit near side always; visit far side only if the
                // split plane is within the neighbourhood radius.
                let (near, far) = if diff <= 0.0 {
                    (left, right)
                } else {
                    (right, left)
                };

                if let Some(near_child) = near {
                    near_child.range_search(query, neighborhood, results);
                }

                if neighborhood.contains(plane_dist) {
                    if let Some(far_child) = far {
                        far_child.range_search(query, neighborhood, results);
                    }
                }
            }
        }
    }

    // ---- helpers ----

    fn update_best(best: &mut Option<(f64, KdPoint<K>)>, sq: f64, p: KdPoint<K>) {
        match best {
            None => *best = Some((sq, p)),
            Some((best_sq, _)) if sq < *best_sq => *best = Some((sq, p)),
            _ => {}
        }
    }

    fn heap_push(
        heap: &mut BinaryHeap<(Reverse<OrderedFloat>, KdPoint<K>)>,
        sq: f64,
        p: KdPoint<K>,
        k: usize,
    ) {
        let of = OrderedFloat(sq);
        if heap.len() < k {
            heap.push((Reverse(of), p));
        } else if let Some(top) = heap.peek() {
            if of < (top.0).0 {
                heap.pop();
                heap.push((Reverse(of), p));
            }
        }
    }
}

// ============================================================================
// OrderedFloat — a total-order wrapper for f64 (heap key)
// ============================================================================

/// Wrapper that provides a total ordering over `f64` values by treating
/// NaN as greater than everything else.  Used exclusively as a heap key
/// inside the k-d tree; never exposed publicly.
#[derive(Debug, Clone, Copy, PartialEq)]
struct OrderedFloat(f64);

impl Eq for OrderedFloat {}

impl PartialOrd for OrderedFloat {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for OrderedFloat {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // NaN sorts last (treated as +∞).
        self.0
            .partial_cmp(&other.0)
            .unwrap_or_else(|| match (self.0.is_nan(), other.0.is_nan()) {
                (true, false) => std::cmp::Ordering::Greater,
                (false, true) => std::cmp::Ordering::Less,
                _ => std::cmp::Ordering::Equal,
            })
    }
}

// The heap stores `(Reverse<OrderedFloat>, KdPoint<K>)`.
// `Reverse` turns the max-heap into a min-heap of distances so the
// farthest-current candidate is at the top for O(1) eviction.
// We need `PartialOrd` / `Ord` on `KdPoint<K>` for `BinaryHeap`.
impl<const K: usize> PartialOrd for KdPoint<K> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<const K: usize> Eq for KdPoint<K> {}

impl<const K: usize> Ord for KdPoint<K> {
    /// Lexicographic ordering, NaN-last.  Used only as a tiebreaker in
    /// the priority queue; semantics beyond ordering are not significant.
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        for (a, b) in self.coords.iter().zip(other.coords.iter()) {
            let ord = OrderedFloat(*a).cmp(&OrderedFloat(*b));
            if ord != std::cmp::Ordering::Equal {
                return ord;
            }
        }
        std::cmp::Ordering::Equal
    }
}

// ============================================================================
// Build helpers
// ============================================================================

/// Build a balanced k-d tree from a mutable slice of points at the
/// given tree `depth`.
///
/// Strategy: sort by the current axis, take the median as the split
/// point, recurse on both halves.
fn build_balanced<const K: usize>(points: &mut [KdPoint<K>], depth: usize) -> Option<Box<Node<K>>> {
    if points.is_empty() {
        return None;
    }
    if points.len() == 1 {
        return Some(Box::new(Node::Leaf(points[0].clone())));
    }

    let axis = depth % K.max(1);

    // Partial sort: put the median element in the middle position.
    let mid = points.len() / 2;
    points.select_nth_unstable_by(mid, |a, b| {
        OrderedFloat(a.coords[axis]).cmp(&OrderedFloat(b.coords[axis]))
    });

    let median_point = points[mid].clone();
    let left = build_balanced(&mut points[..mid], depth + 1);
    let right = build_balanced(&mut points[mid + 1..], depth + 1);

    Some(Box::new(Node::Split {
        point: median_point,
        axis,
        left,
        right,
    }))
}

// ============================================================================
// KdTree
// ============================================================================

/// A K-dimensional tree (k-d tree) for efficient nearest-neighbour and
/// range queries in K-dimensional Euclidean space.
///
/// The const generic parameter `K` fixes the number of dimensions at
/// compile time, eliminating runtime dimension-mismatch errors.
///
/// # Examples
///
/// ```rust
/// use stem_math::spatial_index::{KdTree, KdPoint};
/// use stem_math::spatial::{Distance, Neighborhood};
///
/// // Build from a batch of points (balanced tree).
/// let points = vec![
///     KdPoint { coords: [1.0, 2.0] },
///     KdPoint { coords: [3.0, 4.0] },
///     KdPoint { coords: [5.0, 6.0] },
/// ];
/// let tree = KdTree::from_points(points);
/// assert_eq!(tree.len(), 3);
///
/// // Nearest-neighbour query.
/// let query = KdPoint { coords: [2.0, 3.0] };
/// let nearest = tree.nearest(&query).unwrap();
/// assert_eq!(nearest.point.coords, [1.0, 2.0]);
///
/// // Range query.
/// let radius = Neighborhood::closed(Distance::new(3.0));
/// let within = tree.range_search(&query, &radius);
/// assert!(!within.is_empty());
/// ```
#[derive(Debug, Clone)]
pub struct KdTree<const K: usize> {
    /// Root node of the tree (None when the tree is empty).
    root: Option<Box<Node<K>>>,
    /// Total number of points stored.
    len: usize,
}

impl<const K: usize> KdTree<K> {
    /// Create a new, empty k-d tree.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use stem_math::spatial_index::KdTree;
    ///
    /// let tree: KdTree<3> = KdTree::new();
    /// assert!(tree.is_empty());
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self { root: None, len: 0 }
    }

    /// Build a balanced k-d tree from a vector of points.
    ///
    /// This is more efficient than repeated [`insert`](Self::insert) calls
    /// because it chooses median splits, producing a tree with height
    /// O(log n) even for sorted input.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use stem_math::spatial_index::{KdTree, KdPoint};
    ///
    /// let pts = vec![
    ///     KdPoint { coords: [1.0] },
    ///     KdPoint { coords: [3.0] },
    ///     KdPoint { coords: [2.0] },
    /// ];
    /// let tree = KdTree::from_points(pts);
    /// assert_eq!(tree.len(), 3);
    /// ```
    #[must_use]
    pub fn from_points(points: Vec<KdPoint<K>>) -> Self {
        let len = points.len();
        let mut pts = points;
        let root = build_balanced(&mut pts, 0);
        Self { root, len }
    }

    /// Insert a single point into the tree.
    ///
    /// The tree may become unbalanced after many insertions; prefer
    /// [`from_points`](Self::from_points) for bulk construction.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use stem_math::spatial_index::{KdTree, KdPoint};
    ///
    /// let mut tree: KdTree<2> = KdTree::new();
    /// tree.insert(KdPoint { coords: [1.0, 2.0] });
    /// assert_eq!(tree.len(), 1);
    /// ```
    pub fn insert(&mut self, point: KdPoint<K>) {
        self.len += 1;
        match &mut self.root {
            None => self.root = Some(Box::new(Node::Leaf(point))),
            Some(root) => root.insert(point, 0),
        }
    }

    /// Find the single nearest neighbour to `query`.
    ///
    /// Returns `None` if the tree is empty.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use stem_math::spatial_index::{KdTree, KdPoint};
    ///
    /// let mut tree: KdTree<2> = KdTree::new();
    /// assert!(tree.nearest(&KdPoint { coords: [0.0, 0.0] }).is_none());
    ///
    /// tree.insert(KdPoint { coords: [3.0, 4.0] });
    /// let r = tree.nearest(&KdPoint { coords: [0.0, 0.0] }).unwrap();
    /// assert!((r.distance.value() - 5.0).abs() < 1e-10);
    /// ```
    #[must_use]
    pub fn nearest(&self, query: &KdPoint<K>) -> Option<NearestResult<K>> {
        let root = self.root.as_ref()?;
        let mut best: Option<(f64, KdPoint<K>)> = None;
        root.nearest_search(query, &mut best);
        best.map(|(sq, point)| NearestResult {
            distance: Distance::new(sq.sqrt()),
            point,
        })
    }

    /// Find the `k` nearest neighbours to `query`, sorted by distance
    /// (closest first).
    ///
    /// # Errors
    ///
    /// - [`SpatialIndexError::InvalidK`] if `k == 0`.
    /// - [`SpatialIndexError::EmptyTree`] if the tree contains no points.
    ///
    /// If `k` exceeds the number of points in the tree, all points are
    /// returned (no error).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use stem_math::spatial_index::{KdTree, KdPoint, SpatialIndexError};
    ///
    /// let mut tree: KdTree<2> = KdTree::new();
    /// let q = KdPoint { coords: [0.0, 0.0] };
    ///
    /// assert_eq!(tree.k_nearest(&q, 0), Err(SpatialIndexError::InvalidK));
    /// assert_eq!(tree.k_nearest(&q, 1), Err(SpatialIndexError::EmptyTree));
    ///
    /// tree.insert(KdPoint { coords: [1.0, 0.0] });
    /// tree.insert(KdPoint { coords: [2.0, 0.0] });
    ///
    /// let results = tree.k_nearest(&q, 1).unwrap();
    /// assert_eq!(results.len(), 1);
    /// assert_eq!(results[0].point.coords, [1.0, 0.0]);
    /// ```
    pub fn k_nearest(
        &self,
        query: &KdPoint<K>,
        k: usize,
    ) -> Result<Vec<NearestResult<K>>, SpatialIndexError> {
        if k == 0 {
            return Err(SpatialIndexError::InvalidK);
        }
        let root = self.root.as_ref().ok_or(SpatialIndexError::EmptyTree)?;

        let mut heap: BinaryHeap<(Reverse<OrderedFloat>, KdPoint<K>)> =
            BinaryHeap::with_capacity(k + 1);

        root.k_nearest_search(query, k, &mut heap);

        // Drain heap and sort by ascending distance.
        let mut results: Vec<NearestResult<K>> = heap
            .into_iter()
            .map(|(Reverse(of), point)| NearestResult {
                distance: Distance::new(of.0.sqrt()),
                point,
            })
            .collect();

        results.sort_by(|a, b| {
            OrderedFloat(a.distance.value()).cmp(&OrderedFloat(b.distance.value()))
        });

        Ok(results)
    }

    /// Return all points within the given [`Neighborhood`] of `query`.
    ///
    /// The result order is unspecified.  The neighbourhood's `open` flag
    /// determines whether the boundary (exact radius) is included or
    /// excluded — matching the semantics of [`Neighborhood::contains`].
    ///
    /// # Examples
    ///
    /// ```rust
    /// use stem_math::spatial_index::{KdTree, KdPoint};
    /// use stem_math::spatial::{Distance, Neighborhood};
    ///
    /// let tree = KdTree::from_points(vec![
    ///     KdPoint { coords: [1.0, 0.0] },
    ///     KdPoint { coords: [5.0, 0.0] },
    /// ]);
    ///
    /// let q = KdPoint { coords: [0.0, 0.0] };
    /// let hood = Neighborhood::closed(Distance::new(2.0));
    /// let results = tree.range_search(&q, &hood);
    /// assert_eq!(results.len(), 1);
    /// assert_eq!(results[0].point.coords, [1.0, 0.0]);
    /// ```
    #[must_use]
    pub fn range_search(&self, query: &KdPoint<K>, radius: &Neighborhood) -> Vec<NearestResult<K>> {
        let mut results = Vec::new();
        if let Some(root) = &self.root {
            root.range_search(query, radius, &mut results);
        }
        results
    }

    /// Return the number of points stored in the tree.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use stem_math::spatial_index::{KdTree, KdPoint};
    ///
    /// let mut tree: KdTree<1> = KdTree::new();
    /// assert_eq!(tree.len(), 0);
    /// tree.insert(KdPoint { coords: [1.0] });
    /// assert_eq!(tree.len(), 1);
    /// ```
    #[must_use]
    pub fn len(&self) -> usize {
        self.len
    }

    /// Return `true` if the tree contains no points.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use stem_math::spatial_index::{KdTree, KdPoint};
    ///
    /// let tree: KdTree<2> = KdTree::new();
    /// assert!(tree.is_empty());
    /// ```
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
}

impl<const K: usize> Default for KdTree<K> {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spatial::{Distance, Neighborhood};

    // ---- helpers ----

    fn pt2(x: f64, y: f64) -> KdPoint<2> {
        KdPoint { coords: [x, y] }
    }

    fn pt3(x: f64, y: f64, z: f64) -> KdPoint<3> {
        KdPoint { coords: [x, y, z] }
    }

    fn pt1(x: f64) -> KdPoint<1> {
        KdPoint { coords: [x] }
    }

    /// Brute-force nearest neighbour for result verification.
    fn brute_nearest<const K: usize>(
        points: &[KdPoint<K>],
        query: &KdPoint<K>,
    ) -> Option<NearestResult<K>> {
        points
            .iter()
            .min_by(|a, b| {
                let da = a.squared_distance_to(query);
                let db = b.squared_distance_to(query);
                OrderedFloat(da).cmp(&OrderedFloat(db))
            })
            .map(|p| NearestResult {
                distance: query.euclidean_distance_to(p),
                point: p.clone(),
            })
    }

    // ---- empty tree ----

    #[test]
    fn empty_tree_len_is_zero() {
        let tree: KdTree<2> = KdTree::new();
        assert_eq!(tree.len(), 0);
    }

    #[test]
    fn empty_tree_is_empty() {
        let tree: KdTree<2> = KdTree::new();
        assert!(tree.is_empty());
    }

    #[test]
    fn empty_tree_nearest_returns_none() {
        let tree: KdTree<2> = KdTree::new();
        assert!(tree.nearest(&pt2(0.0, 0.0)).is_none());
    }

    #[test]
    fn empty_tree_k_nearest_returns_empty_tree_error() {
        let tree: KdTree<2> = KdTree::new();
        let q = pt2(0.0, 0.0);
        assert_eq!(tree.k_nearest(&q, 1), Err(SpatialIndexError::EmptyTree));
    }

    #[test]
    fn empty_tree_range_search_returns_empty_vec() {
        let tree: KdTree<2> = KdTree::new();
        let hood = Neighborhood::closed(Distance::new(100.0));
        assert!(tree.range_search(&pt2(0.0, 0.0), &hood).is_empty());
    }

    // ---- single point ----

    #[test]
    fn single_point_insert_len() {
        let mut tree: KdTree<2> = KdTree::new();
        tree.insert(pt2(1.0, 1.0));
        assert_eq!(tree.len(), 1);
        assert!(!tree.is_empty());
    }

    #[test]
    fn single_point_nearest_finds_it() {
        let mut tree: KdTree<2> = KdTree::new();
        tree.insert(pt2(3.0, 4.0));
        let r = tree.nearest(&pt2(0.0, 0.0)).unwrap();
        assert_eq!(r.point.coords, [3.0, 4.0]);
        assert!((r.distance.value() - 5.0).abs() < 1e-10);
    }

    #[test]
    fn single_point_k_nearest_returns_it() {
        let mut tree: KdTree<2> = KdTree::new();
        tree.insert(pt2(1.0, 0.0));
        let results = tree.k_nearest(&pt2(0.0, 0.0), 1).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].point.coords, [1.0, 0.0]);
    }

    #[test]
    fn single_point_range_includes_it_within_radius() {
        let mut tree: KdTree<2> = KdTree::new();
        tree.insert(pt2(1.0, 0.0));
        let hood = Neighborhood::closed(Distance::new(2.0));
        let results = tree.range_search(&pt2(0.0, 0.0), &hood);
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn single_point_range_excludes_it_outside_radius() {
        let mut tree: KdTree<2> = KdTree::new();
        tree.insert(pt2(10.0, 0.0));
        let hood = Neighborhood::closed(Distance::new(2.0));
        let results = tree.range_search(&pt2(0.0, 0.0), &hood);
        assert!(results.is_empty());
    }

    // ---- two points ----

    #[test]
    fn two_points_nearest_finds_closer() {
        let mut tree: KdTree<2> = KdTree::new();
        tree.insert(pt2(1.0, 0.0));
        tree.insert(pt2(10.0, 0.0));
        let r = tree.nearest(&pt2(0.0, 0.0)).unwrap();
        assert_eq!(r.point.coords, [1.0, 0.0]);
    }

    #[test]
    fn two_points_nearest_other_side() {
        let mut tree: KdTree<2> = KdTree::new();
        tree.insert(pt2(1.0, 0.0));
        tree.insert(pt2(10.0, 0.0));
        let r = tree.nearest(&pt2(8.0, 0.0)).unwrap();
        assert_eq!(r.point.coords, [10.0, 0.0]);
    }

    // ---- 2-D points ----

    #[test]
    fn two_d_nearest_various_queries() {
        let tree = KdTree::from_points(vec![
            pt2(0.0, 0.0),
            pt2(4.0, 0.0),
            pt2(0.0, 3.0),
            pt2(4.0, 3.0),
        ]);
        // Query near top-right corner
        let r = tree.nearest(&pt2(5.0, 4.0)).unwrap();
        assert_eq!(r.point.coords, [4.0, 3.0]);

        // Query near origin
        let r2 = tree.nearest(&pt2(0.1, 0.1)).unwrap();
        assert_eq!(r2.point.coords, [0.0, 0.0]);
    }

    #[test]
    fn from_points_len_correct() {
        let tree = KdTree::from_points(vec![pt2(1.0, 2.0), pt2(3.0, 4.0), pt2(5.0, 6.0)]);
        assert_eq!(tree.len(), 3);
    }

    // ---- 3-D points ----

    #[test]
    fn three_d_nearest() {
        let tree = KdTree::from_points(vec![
            pt3(1.0, 0.0, 0.0),
            pt3(0.0, 1.0, 0.0),
            pt3(0.0, 0.0, 1.0),
            pt3(5.0, 5.0, 5.0),
        ]);
        let r = tree.nearest(&pt3(0.0, 0.0, 0.0)).unwrap();
        // All three axis points are equidistant — any is acceptable.
        assert!((r.distance.value() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn three_d_k_nearest_two() {
        let tree = KdTree::from_points(vec![
            pt3(1.0, 0.0, 0.0),
            pt3(2.0, 0.0, 0.0),
            pt3(10.0, 0.0, 0.0),
        ]);
        let results = tree.k_nearest(&pt3(0.0, 0.0, 0.0), 2).unwrap();
        assert_eq!(results.len(), 2);
        assert!((results[0].distance.value() - 1.0).abs() < 1e-10);
        assert!((results[1].distance.value() - 2.0).abs() < 1e-10);
    }

    // ---- k_nearest edge cases ----

    #[test]
    fn k_nearest_k_zero_returns_invalid_k() {
        let tree = KdTree::from_points(vec![pt2(1.0, 0.0)]);
        let q = pt2(0.0, 0.0);
        assert_eq!(tree.k_nearest(&q, 0), Err(SpatialIndexError::InvalidK));
    }

    #[test]
    fn k_nearest_k_one_matches_nearest() {
        let tree = KdTree::from_points(vec![pt2(1.0, 2.0), pt2(5.0, 6.0), pt2(3.0, 4.0)]);
        let q = pt2(0.0, 0.0);
        let knn = tree.k_nearest(&q, 1).unwrap();
        let nn = tree.nearest(&q).unwrap();
        assert!((knn[0].distance.value() - nn.distance.value()).abs() < 1e-10);
    }

    #[test]
    fn k_nearest_k_exceeds_count_returns_all() {
        let tree = KdTree::from_points(vec![pt2(1.0, 0.0), pt2(2.0, 0.0)]);
        let q = pt2(0.0, 0.0);
        let results = tree.k_nearest(&q, 100).unwrap();
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn k_nearest_results_sorted_by_distance() {
        let tree = KdTree::from_points(vec![
            pt2(3.0, 0.0),
            pt2(1.0, 0.0),
            pt2(5.0, 0.0),
            pt2(2.0, 0.0),
        ]);
        let results = tree.k_nearest(&pt2(0.0, 0.0), 4).unwrap();
        for w in results.windows(2) {
            assert!(w[0].distance.value() <= w[1].distance.value());
        }
    }

    // ---- range_search ----

    #[test]
    fn range_search_finds_within_closed_radius() {
        let tree = KdTree::from_points(vec![pt2(1.0, 0.0), pt2(2.0, 0.0), pt2(10.0, 0.0)]);
        let hood = Neighborhood::closed(Distance::new(2.0));
        let results = tree.range_search(&pt2(0.0, 0.0), &hood);
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn range_search_excludes_boundary_when_open() {
        let tree = KdTree::from_points(vec![
            pt2(1.0, 0.0), // dist = 1.0 — inside open(1.0)? No, 1.0 < 1.0 is false
            pt2(0.5, 0.0), // dist = 0.5 — inside
        ]);
        let hood = Neighborhood::open(Distance::new(1.0));
        let results = tree.range_search(&pt2(0.0, 0.0), &hood);
        // Only the 0.5-distance point should be within the open ball.
        assert_eq!(results.len(), 1);
        assert!((results[0].point.coords[0] - 0.5).abs() < 1e-10);
    }

    #[test]
    fn range_search_includes_boundary_when_closed() {
        let tree = KdTree::from_points(vec![pt2(2.0, 0.0)]);
        let hood = Neighborhood::closed(Distance::new(2.0));
        let results = tree.range_search(&pt2(0.0, 0.0), &hood);
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn range_search_empty_radius_returns_nothing() {
        let tree = KdTree::from_points(vec![pt2(1.0, 0.0)]);
        let hood = Neighborhood::open(Distance::new(0.0));
        let results = tree.range_search(&pt2(0.0, 0.0), &hood);
        assert!(results.is_empty());
    }

    // ---- large dataset brute-force verification ----

    #[test]
    fn large_dataset_nearest_matches_brute_force() {
        // Deterministic "random" points via a simple LCG.
        let mut seed: u64 = 42;
        let mut next = || -> f64 {
            seed = seed
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1_442_695_040_888_963_407);
            // Map to [-50, 50]
            let raw = ((seed >> 33) as f64) / (u32::MAX as f64);
            raw * 100.0 - 50.0
        };

        let points: Vec<KdPoint<3>> = (0..100).map(|_| pt3(next(), next(), next())).collect();

        let tree = KdTree::from_points(points.clone());

        // Verify 10 different query points.
        let queries: Vec<KdPoint<3>> = (0..10).map(|_| pt3(next(), next(), next())).collect();

        for q in &queries {
            let kd_result = tree.nearest(q).unwrap();
            let bf_result = brute_nearest(&points, q).unwrap();
            assert!(
                (kd_result.distance.value() - bf_result.distance.value()).abs() < 1e-9,
                "k-d tree nearest distance {:.6} != brute force {:.6} for query {:?}",
                kd_result.distance.value(),
                bf_result.distance.value(),
                q.coords
            );
        }
    }

    // ---- degenerate: all same point ----

    #[test]
    fn degenerate_all_same_point() {
        let tree = KdTree::from_points(vec![pt2(1.0, 1.0), pt2(1.0, 1.0), pt2(1.0, 1.0)]);
        assert_eq!(tree.len(), 3);
        let r = tree.nearest(&pt2(0.0, 0.0)).unwrap();
        assert_eq!(r.point.coords, [1.0, 1.0]);
        assert!((r.distance.value() - 2.0_f64.sqrt()).abs() < 1e-10);
    }

    #[test]
    fn degenerate_all_same_point_k_nearest_returns_all() {
        let tree = KdTree::from_points(vec![pt2(1.0, 1.0), pt2(1.0, 1.0), pt2(1.0, 1.0)]);
        let results = tree.k_nearest(&pt2(0.0, 0.0), 3).unwrap();
        assert_eq!(results.len(), 3);
    }

    // ---- degenerate: collinear points ----

    #[test]
    fn degenerate_collinear_nearest() {
        let tree = KdTree::from_points((0..10).map(|i| pt2(i as f64, 0.0)).collect());
        let r = tree.nearest(&pt2(3.5, 0.0)).unwrap();
        // Nearest must be 3.0 or 4.0 — both at distance 0.5.
        assert!((r.distance.value() - 0.5).abs() < 1e-10);
    }

    #[test]
    fn degenerate_collinear_range_search() {
        let tree = KdTree::from_points((0..20).map(|i| pt2(i as f64, 0.0)).collect());
        let hood = Neighborhood::closed(Distance::new(3.0));
        let results = tree.range_search(&pt2(10.0, 0.0), &hood);
        // Points 7, 8, 9, 10, 11, 12, 13 are within distance 3.
        assert_eq!(results.len(), 7);
    }

    // ---- 1-D edge case ----

    #[test]
    fn one_dimensional_nearest() {
        let tree = KdTree::from_points(vec![pt1(5.0), pt1(1.0), pt1(9.0)]);
        let r = tree.nearest(&pt1(6.0)).unwrap();
        assert_eq!(r.point.coords, [5.0]);
        assert!((r.distance.value() - 1.0).abs() < 1e-10);
    }

    // ---- from_points balanced vs insert ----

    #[test]
    fn from_points_and_insert_give_same_len() {
        let pts = vec![pt2(1.0, 2.0), pt2(3.0, 4.0), pt2(5.0, 6.0)];
        let from = KdTree::from_points(pts.clone());

        let mut inserted: KdTree<2> = KdTree::new();
        for p in pts {
            inserted.insert(p);
        }

        assert_eq!(from.len(), inserted.len());
    }

    #[test]
    fn from_points_nearest_consistent_with_insert() {
        let pts = vec![pt2(1.0, 0.0), pt2(5.0, 0.0), pt2(3.0, 0.0)];
        let from = KdTree::from_points(pts.clone());

        let mut ins: KdTree<2> = KdTree::new();
        for p in pts {
            ins.insert(p);
        }

        let q = pt2(0.0, 0.0);
        let rf = from.nearest(&q).unwrap();
        let ri = ins.nearest(&q).unwrap();
        assert!((rf.distance.value() - ri.distance.value()).abs() < 1e-10);
    }

    // ---- error type properties ----

    #[test]
    fn spatial_index_error_empty_tree_display() {
        let msg = format!("{}", SpatialIndexError::EmptyTree);
        assert!(msg.contains("empty tree"));
    }

    #[test]
    fn spatial_index_error_invalid_k_display() {
        let msg = format!("{}", SpatialIndexError::InvalidK);
        assert!(msg.contains("k must be at least 1"));
    }

    #[test]
    fn spatial_index_error_dimension_mismatch_display() {
        let msg = format!("{}", SpatialIndexError::DimensionMismatch);
        assert!(msg.contains("dimension mismatch"));
    }

    #[test]
    fn spatial_index_error_is_clone_and_partial_eq() {
        let e = SpatialIndexError::InvalidK;
        assert_eq!(e.clone(), SpatialIndexError::InvalidK);
    }

    // ---- default impl ----

    #[test]
    fn kdtree_default_is_empty() {
        let tree: KdTree<2> = KdTree::default();
        assert!(tree.is_empty());
    }

    // ---- KdPoint euclidean distance ----

    #[test]
    fn kdpoint_euclidean_distance_3_4_5() {
        let a = pt2(0.0, 0.0);
        let b = pt2(3.0, 4.0);
        let d = a.euclidean_distance_to(&b);
        assert!((d.value() - 5.0).abs() < 1e-10);
    }

    #[test]
    fn kdpoint_distance_self_is_zero() {
        let p = pt3(1.0, 2.0, 3.0);
        let d = p.euclidean_distance_to(&p);
        assert!((d.value() - 0.0).abs() < 1e-10);
    }

    // ---- NearestResult properties ----

    #[test]
    fn nearest_result_distance_non_negative() {
        let tree = KdTree::from_points(vec![pt2(-5.0, -5.0)]);
        let r = tree.nearest(&pt2(0.0, 0.0)).unwrap();
        assert!(r.distance.value() >= 0.0);
    }
}
