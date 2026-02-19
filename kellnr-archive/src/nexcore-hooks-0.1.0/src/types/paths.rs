//! Validated path types.

use std::fmt;
use std::path::{Component, Path, PathBuf};

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::error::{HookError, HookResult};

/// A validated file system path.
///
/// # Invariant
/// Path string is non-empty and valid UTF-8.
/// Does NOT guarantee path exists (use `ValidPath::exists()` to check).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ValidPath(PathBuf);

impl ValidPath {
    /// Create from path-like value.
    pub fn new(p: impl AsRef<Path>) -> HookResult<Self> {
        let path = p.as_ref();
        if path.as_os_str().is_empty() {
            return Err(HookError::ValidationFailed("path cannot be empty".into()));
        }
        Ok(Self(path.to_path_buf()))
    }

    /// Check if path exists on filesystem.
    #[inline]
    #[must_use]
    pub fn exists(&self) -> bool {
        self.0.exists()
    }

    /// Check if path is a file.
    #[inline]
    #[must_use]
    pub fn is_file(&self) -> bool {
        self.0.is_file()
    }

    /// Check if path is a directory.
    #[inline]
    #[must_use]
    pub fn is_dir(&self) -> bool {
        self.0.is_dir()
    }

    /// Check if path is absolute.
    #[inline]
    #[must_use]
    pub fn is_absolute(&self) -> bool {
        self.0.is_absolute()
    }

    /// Get as Path reference.
    #[inline]
    #[must_use]
    pub fn as_path(&self) -> &Path {
        &self.0
    }

    /// Get as string slice (if valid UTF-8).
    #[must_use]
    pub fn as_str(&self) -> Option<&str> {
        self.0.to_str()
    }

    /// Consume and return inner PathBuf.
    #[inline]
    #[must_use]
    pub fn into_inner(self) -> PathBuf {
        self.0
    }

    /// Get file extension if present.
    #[must_use]
    pub fn extension(&self) -> Option<&str> {
        self.0.extension().and_then(|e| e.to_str())
    }

    /// Get file name component.
    #[must_use]
    pub fn file_name(&self) -> Option<&str> {
        self.0.file_name().and_then(|n| n.to_str())
    }

    /// Join with another path component.
    #[must_use]
    pub fn join(&self, path: impl AsRef<Path>) -> Self {
        Self(self.0.join(path))
    }

    /// Get parent directory.
    #[must_use]
    pub fn parent(&self) -> Option<Self> {
        self.0.parent().map(|p| Self(p.to_path_buf()))
    }
}

impl AsRef<Path> for ValidPath {
    fn as_ref(&self) -> &Path {
        &self.0
    }
}

impl fmt::Display for ValidPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0.display())
    }
}

impl Serialize for ValidPath {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.0.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for ValidPath {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let path = PathBuf::deserialize(deserializer)?;
        ValidPath::new(path).map_err(serde::de::Error::custom)
    }
}

/// A path guaranteed to be within a project directory (no traversal).
///
/// # Invariant
/// After normalization, path does not escape project root.
/// No `..` components that would traverse above root.
///
/// # Security
/// Prevents path traversal attacks like `/project/../../../etc/passwd`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ProjectPath {
    /// The normalized path within project.
    path: PathBuf,
    /// The project root directory.
    project_root: PathBuf,
}

impl ProjectPath {
    /// Create project-scoped path.
    ///
    /// # Errors
    /// Returns error if path would escape project_root.
    pub fn new(path: impl AsRef<Path>, project_root: impl AsRef<Path>) -> HookResult<Self> {
        let path_ref = path.as_ref();
        let root = project_root.as_ref();

        if path_ref.as_os_str().is_empty() {
            return Err(HookError::ValidationFailed("path cannot be empty".into()));
        }

        // Normalize and check containment
        let normalized = Self::normalize_path(path_ref, root)?;

        Ok(Self {
            path: normalized,
            project_root: root.to_path_buf(),
        })
    }

    fn normalize_path(path: &Path, root: &Path) -> HookResult<PathBuf> {
        // If relative, join with root
        let full_path = if path.is_relative() {
            root.join(path)
        } else {
            path.to_path_buf()
        };

        // Normalize (remove . and ..)
        let mut normalized = PathBuf::new();
        let mut depth: i32 = 0;

        for component in full_path.components() {
            match component {
                Component::ParentDir => {
                    if normalized.pop() {
                        depth -= 1;
                    }
                    if depth < 0 {
                        return Err(HookError::ValidationFailed(
                            "path traversal beyond root".into(),
                        ));
                    }
                }
                Component::CurDir => {}
                Component::Normal(_) => {
                    normalized.push(component);
                    depth += 1;
                }
                c => {
                    normalized.push(c);
                }
            }
        }

        // Verify within project root
        if !normalized.starts_with(root) {
            return Err(HookError::ValidationFailed(format!(
                "path {} escapes project root {}",
                normalized.display(),
                root.display()
            )));
        }

        Ok(normalized)
    }

    /// Get the normalized path.
    #[inline]
    #[must_use]
    pub fn as_path(&self) -> &Path {
        &self.path
    }

    /// Get the project root.
    #[inline]
    #[must_use]
    pub fn project_root(&self) -> &Path {
        &self.project_root
    }

    /// Get path relative to project root.
    #[must_use]
    pub fn relative_path(&self) -> Option<&Path> {
        self.path.strip_prefix(&self.project_root).ok()
    }

    /// Check if path exists.
    #[inline]
    #[must_use]
    pub fn exists(&self) -> bool {
        self.path.exists()
    }
}

impl AsRef<Path> for ProjectPath {
    fn as_ref(&self) -> &Path {
        &self.path
    }
}

impl fmt::Display for ProjectPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.path.display())
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn valid_path_rejects_empty() {
        assert!(ValidPath::new("").is_err());
    }

    #[test]
    fn valid_path_accepts_relative() {
        let p = ValidPath::new("foo/bar.txt").unwrap();
        assert!(!p.is_absolute());
        assert_eq!(p.file_name(), Some("bar.txt"));
        assert_eq!(p.extension(), Some("txt"));
    }

    #[test]
    fn valid_path_accepts_absolute() {
        let p = ValidPath::new("/foo/bar.txt").unwrap();
        assert!(p.is_absolute());
    }

    #[test]
    fn valid_path_join() {
        let p = ValidPath::new("/project").unwrap();
        let joined = p.join("src/main.rs");
        assert_eq!(joined.as_str(), Some("/project/src/main.rs"));
    }

    #[test]
    fn project_path_allows_within_root() {
        let pp = ProjectPath::new("src/main.rs", "/project");
        assert!(pp.is_ok());
    }

    #[test]
    fn project_path_allows_nested() {
        let pp = ProjectPath::new("src/foo/../bar/baz.rs", "/project").unwrap();
        assert!(pp.as_path().ends_with("bar/baz.rs"));
    }

    #[test]
    fn project_path_blocks_traversal() {
        let pp = ProjectPath::new("../../../etc/passwd", "/project");
        assert!(pp.is_err());
    }

    #[test]
    fn project_path_blocks_absolute_escape() {
        let pp = ProjectPath::new("/etc/passwd", "/project");
        assert!(pp.is_err());
    }

    #[test]
    fn project_path_blocks_empty() {
        let pp = ProjectPath::new("", "/project");
        assert!(pp.is_err());
    }

    #[test]
    fn project_path_relative() {
        let pp = ProjectPath::new("src/lib.rs", "/project").unwrap();
        assert_eq!(
            pp.relative_path().map(|p| p.to_str().unwrap()),
            Some("src/lib.rs")
        );
    }

    #[test]
    fn valid_path_serde_roundtrip() {
        let p = ValidPath::new("/foo/bar.txt").unwrap();
        let json = serde_json::to_string(&p).unwrap();
        let parsed: ValidPath = serde_json::from_str(&json).unwrap();
        assert_eq!(p, parsed);
    }
}
