//! # T1 Primitive Type Markers
//!
//! Zero-cost type markers representing the 15 irreducible T1 primitives.
//! These enable type-level reasoning about domain patterns.
//!
//! > "Primitives are built from primitives. There is no bottom."

use std::marker::PhantomData;

/// T1 Sequence primitive marker (σ).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct T1Sequence;

impl T1Sequence {
    pub const NAME: &'static str = "sequence";
    pub const SYMBOL: char = 'σ';
    pub const RUST_FORMS: &'static [&'static str] = &[
        "Iterator",
        "for",
        "while",
        "loop",
        ".iter()",
        ".into_iter()",
        ".chain()",
    ];
}

/// T1 Mapping primitive marker (μ).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct T1Mapping;

impl T1Mapping {
    pub const NAME: &'static str = "mapping";
    pub const SYMBOL: char = 'μ';
    pub const RUST_FORMS: &'static [&'static str] = &[
        "From",
        "Into",
        "TryFrom",
        "TryInto",
        "AsRef",
        "Deref",
        ".map()",
        ".and_then()",
    ];
}

/// T1 Recursion primitive marker (ρ).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct T1Recursion;

impl T1Recursion {
    pub const NAME: &'static str = "recursion";
    pub const SYMBOL: char = 'ρ';
    pub const RUST_FORMS: &'static [&'static str] = &[
        "enum",
        "Box<Self>",
        "Rc<Self>",
        "Arc<Self>",
        "match",
        "recursive fn",
    ];
}

/// T1 State primitive marker (ς).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct T1State;

impl T1State {
    pub const NAME: &'static str = "state";
    pub const SYMBOL: char = 'ς';
    pub const RUST_FORMS: &'static [&'static str] = &[
        "struct + impl",
        "PhantomData",
        "Cell",
        "RefCell",
        "Mutex",
        "RwLock",
        "Atomic*",
    ];
}

/// T1 Void primitive marker (∅).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct T1Void;

impl T1Void {
    pub const NAME: &'static str = "void";
    pub const SYMBOL: char = '∅';
    pub const RUST_FORMS: &'static [&'static str] = &[
        "Option::None",
        "PhantomData<T>",
        "()",
        "<T>",
        "trait bounds",
        "!",
    ];
}

/// T1 Boundary primitive marker (∂).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct T1Boundary;

impl T1Boundary {
    pub const NAME: &'static str = "boundary";
    pub const SYMBOL: char = '∂';
    pub const RUST_FORMS: &'static [&'static str] = &["HALT", "max_iterations", "Result", "break"];
}

/// T1 Frequency primitive marker (f).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct T1Frequency;

impl T1Frequency {
    pub const NAME: &'static str = "frequency";
    pub const SYMBOL: char = 'f';
    pub const RUST_FORMS: &'static [&'static str] = &["Loop Counter", "Rate Limiter", "f64"];
}

/// T1 Existence primitive marker (∃).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct T1Existence;

impl T1Existence {
    pub const NAME: &'static str = "existence";
    pub const SYMBOL: char = '∃';
    pub const RUST_FORMS: &'static [&'static str] =
        &["Session::new()", "fs::write()", "instantiation"];
}

/// T1 Persistence primitive marker (π).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct T1Persistence;

impl T1Persistence {
    pub const NAME: &'static str = "persistence";
    pub const SYMBOL: char = 'π';
    pub const RUST_FORMS: &'static [&'static str] =
        &["Database", "Brain Storage", "Logs", "static"];
}

/// T1 Causality primitive marker (→).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct T1Causality;

impl T1Causality {
    pub const NAME: &'static str = "causality";
    pub const SYMBOL: char = '→';
    pub const RUST_FORMS: &'static [&'static str] =
        &["Function Call", "Event Trigger", "Result transition"];
}

/// T1 Comparison primitive marker (κ).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct T1Comparison;

impl T1Comparison {
    pub const NAME: &'static str = "comparison";
    pub const SYMBOL: char = 'κ';
    pub const RUST_FORMS: &'static [&'static str] = &["==", "match", "if let", "cmp()"];
}

/// T1 Quantity primitive marker (N).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct T1Quantity;

impl T1Quantity {
    pub const NAME: &'static str = "quantity";
    pub const SYMBOL: char = 'N';
    pub const RUST_FORMS: &'static [&'static str] = &["u32", "f64", "usize", "magnitude"];
}

/// T1 Location primitive marker (λ).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct T1Location;

impl T1Location {
    pub const NAME: &'static str = "location";
    pub const SYMBOL: char = 'λ';
    pub const RUST_FORMS: &'static [&'static str] = &["Path", "Pointer", "URL", "Address"];
}

/// T1 Irreversibility primitive marker (∝).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct T1Irreversibility;

impl T1Irreversibility {
    pub const NAME: &'static str = "irreversibility";
    pub const SYMBOL: char = '∝';
    pub const RUST_FORMS: &'static [&'static str] = &["Drop", "Consuming methods", "Entropy floor"];
}

/// T1 Sum primitive marker (Σ).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct T1Sum;

impl T1Sum {
    pub const NAME: &'static str = "sum";
    pub const SYMBOL: char = 'Σ';
    pub const RUST_FORMS: &'static [&'static str] = &[
        "enum",
        "match",
        "if let",
        "Or<P, Q>",
        "Either<A, B>",
        "Result<T, E>",
    ];
}

/// Wrapper for a pattern tagged with its dominant primitive.
#[derive(Debug, Clone)]
pub struct PrimitiveTagged<P, T> {
    pub primitive: PhantomData<P>,
    pub value: T,
}

impl<P, T> PrimitiveTagged<P, T> {
    pub fn new(value: T) -> Self {
        Self {
            primitive: PhantomData,
            value,
        }
    }
    pub fn into_inner(self) -> T {
        self.value
    }
}

pub trait HasPrimitive {
    fn primitive_name(&self) -> &'static str;
    fn primitive_symbol(&self) -> char;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum T1Primitive {
    Sequence,
    Mapping,
    Recursion,
    State,
    Void,
    Boundary,
    Frequency,
    Existence,
    Persistence,
    Causality,
    Comparison,
    Quantity,
    Location,
    Irreversibility,
    Sum,
}

impl T1Primitive {
    pub const ALL: [T1Primitive; 15] = [
        T1Primitive::Sequence,
        T1Primitive::Mapping,
        T1Primitive::Recursion,
        T1Primitive::State,
        T1Primitive::Void,
        T1Primitive::Boundary,
        T1Primitive::Frequency,
        T1Primitive::Existence,
        T1Primitive::Persistence,
        T1Primitive::Causality,
        T1Primitive::Comparison,
        T1Primitive::Quantity,
        T1Primitive::Location,
        T1Primitive::Irreversibility,
        T1Primitive::Sum,
    ];

    pub const fn name(&self) -> &'static str {
        match self {
            Self::Sequence => T1Sequence::NAME,
            Self::Mapping => T1Mapping::NAME,
            Self::Recursion => T1Recursion::NAME,
            Self::State => T1State::NAME,
            Self::Void => T1Void::NAME,
            Self::Boundary => T1Boundary::NAME,
            Self::Frequency => T1Frequency::NAME,
            Self::Existence => T1Existence::NAME,
            Self::Persistence => T1Persistence::NAME,
            Self::Causality => T1Causality::NAME,
            Self::Comparison => T1Comparison::NAME,
            Self::Quantity => T1Quantity::NAME,
            Self::Location => T1Location::NAME,
            Self::Irreversibility => T1Irreversibility::NAME,
            Self::Sum => T1Sum::NAME,
        }
    }

    pub const fn symbol(&self) -> char {
        match self {
            Self::Sequence => T1Sequence::SYMBOL,
            Self::Mapping => T1Mapping::SYMBOL,
            Self::Recursion => T1Recursion::SYMBOL,
            Self::State => T1State::SYMBOL,
            Self::Void => T1Void::SYMBOL,
            Self::Boundary => T1Boundary::SYMBOL,
            Self::Frequency => T1Frequency::SYMBOL,
            Self::Existence => T1Existence::SYMBOL,
            Self::Persistence => T1Persistence::SYMBOL,
            Self::Causality => T1Causality::SYMBOL,
            Self::Comparison => T1Comparison::SYMBOL,
            Self::Quantity => T1Quantity::SYMBOL,
            Self::Location => T1Location::SYMBOL,
            Self::Irreversibility => T1Irreversibility::SYMBOL,
            Self::Sum => T1Sum::SYMBOL,
        }
    }

    pub const fn rust_forms(&self) -> &'static [&'static str] {
        match self {
            Self::Sequence => T1Sequence::RUST_FORMS,
            Self::Mapping => T1Mapping::RUST_FORMS,
            Self::Recursion => T1Recursion::RUST_FORMS,
            Self::State => T1State::RUST_FORMS,
            Self::Void => T1Void::RUST_FORMS,
            Self::Boundary => T1Boundary::RUST_FORMS,
            Self::Frequency => T1Frequency::RUST_FORMS,
            Self::Existence => T1Existence::RUST_FORMS,
            Self::Persistence => T1Persistence::RUST_FORMS,
            Self::Causality => T1Causality::RUST_FORMS,
            Self::Comparison => T1Comparison::RUST_FORMS,
            Self::Quantity => T1Quantity::RUST_FORMS,
            Self::Location => T1Location::RUST_FORMS,
            Self::Irreversibility => T1Irreversibility::RUST_FORMS,
            Self::Sum => T1Sum::RUST_FORMS,
        }
    }
}

impl std::fmt::Display for T1Primitive {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ({})", self.name(), self.symbol())
    }
}
