//! Integration tests for `#[derive(Error)]`.

use nexcore_error_derive::Error;

// ── Enum: basic format string ──────────────────────────────────────
#[derive(Debug, Error)]
enum BasicEnum {
    #[error("not found")]
    NotFound,
    #[error("parse error: {0}")]
    Parse(String),
    #[error("io failed: {msg}")]
    Io { msg: String },
}

#[test]
fn enum_unit_variant_display() {
    let e = BasicEnum::NotFound;
    assert_eq!(e.to_string(), "not found");
}

#[test]
fn enum_tuple_variant_display() {
    let e = BasicEnum::Parse("bad input".into());
    assert_eq!(e.to_string(), "parse error: bad input");
}

#[test]
fn enum_named_variant_display() {
    let e = BasicEnum::Io {
        msg: "disk full".into(),
    };
    assert_eq!(e.to_string(), "io failed: disk full");
}

#[test]
fn enum_is_std_error() {
    let e: Box<dyn std::error::Error> = Box::new(BasicEnum::NotFound);
    assert!(e.source().is_none());
}

// ── Enum: #[from] generates From impl ──────────────────────────────
#[derive(Debug, Error)]
enum WithFrom {
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("other")]
    Other,
}

#[test]
fn from_impl_works() {
    let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "gone");
    let e: WithFrom = io_err.into();
    assert_eq!(e.to_string(), "io: gone");
}

#[test]
fn from_variant_has_source() {
    use std::error::Error;
    let io_err = std::io::Error::new(std::io::ErrorKind::Other, "oops");
    let e: WithFrom = io_err.into();
    assert!(e.source().is_some());
}

// ── Enum: #[error(transparent)] ────────────────────────────────────
#[derive(Debug, Error)]
enum Transparent {
    #[error(transparent)]
    Inner(#[from] std::io::Error),
}

#[test]
fn transparent_delegates_display() {
    let io_err = std::io::Error::new(std::io::ErrorKind::BrokenPipe, "pipe broke");
    let e: Transparent = io_err.into();
    assert_eq!(e.to_string(), "pipe broke");
}

// ── Struct: single-field error ─────────────────────────────────────
#[derive(Debug, Error)]
#[error("config error: {msg}")]
struct ConfigError {
    msg: String,
}

#[test]
fn struct_display() {
    let e = ConfigError {
        msg: "missing key".into(),
    };
    assert_eq!(e.to_string(), "config error: missing key");
}

#[test]
fn struct_is_std_error() {
    let e: Box<dyn std::error::Error> = Box::new(ConfigError { msg: "x".into() });
    assert!(e.source().is_none());
}

// ── Struct: with #[source] ─────────────────────────────────────────
#[derive(Debug, Error)]
#[error("wrapper")]
struct Wrapper {
    #[source]
    inner: std::io::Error,
}

#[test]
fn struct_source_works() {
    use std::error::Error;
    let e = Wrapper {
        inner: std::io::Error::new(std::io::ErrorKind::Other, "root"),
    };
    assert!(e.source().is_some());
    assert_eq!(e.source().map(|s| s.to_string()).as_deref(), Some("root"));
}

// ── Single-variant enum ────────────────────────────────────────────
#[derive(Debug, Error)]
enum SingleVariant {
    #[error("only one")]
    Only,
}

#[test]
fn single_variant_display() {
    assert_eq!(SingleVariant::Only.to_string(), "only one");
}

// ── Multi-field struct ─────────────────────────────────────────────
#[derive(Debug, Error)]
#[error("error at {file}:{line}")]
struct Located {
    file: String,
    line: u32,
}

#[test]
fn multi_field_struct_display() {
    let e = Located {
        file: "main.rs".into(),
        line: 42,
    };
    assert_eq!(e.to_string(), "error at main.rs:42");
}

// ── Tuple struct ───────────────────────────────────────────────────
#[derive(Debug, Error)]
#[error("code: {0}")]
struct CodeError(i32);

#[test]
fn tuple_struct_display() {
    assert_eq!(CodeError(404).to_string(), "code: 404");
}

// ── Enum with mixed variant kinds ──────────────────────────────────
#[derive(Debug, Error)]
enum Mixed {
    #[error("unit")]
    Unit,
    #[error("tuple: {0}")]
    Tuple(String),
    #[error("named: {x}")]
    Named { x: i32 },
    #[error(transparent)]
    Transparent(std::io::Error),
}

#[test]
fn mixed_unit() {
    assert_eq!(Mixed::Unit.to_string(), "unit");
}

#[test]
fn mixed_tuple() {
    assert_eq!(Mixed::Tuple("hello".into()).to_string(), "tuple: hello");
}

#[test]
fn mixed_named() {
    assert_eq!(Mixed::Named { x: 99 }.to_string(), "named: 99");
}

#[test]
fn mixed_transparent() {
    let io = std::io::Error::new(std::io::ErrorKind::Other, "inner");
    assert_eq!(Mixed::Transparent(io).to_string(), "inner");
}

// ── Multiple #[from] sources ───────────────────────────────────────
#[derive(Debug, Error)]
enum MultipleSources {
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("fmt: {0}")]
    Fmt(#[from] std::fmt::Error),
}

#[test]
fn multiple_from_io() {
    let e: MultipleSources = std::io::Error::new(std::io::ErrorKind::Other, "x").into();
    assert!(matches!(e, MultipleSources::Io(_)));
}

#[test]
fn multiple_from_fmt() {
    let e: MultipleSources = std::fmt::Error.into();
    assert!(matches!(e, MultipleSources::Fmt(_)));
}

// ── Nested error chain ─────────────────────────────────────────────
#[derive(Debug, Error)]
enum Inner {
    #[error("deep failure")]
    Deep,
}

#[derive(Debug, Error)]
enum Outer {
    #[error("outer: {0}")]
    Wrapped(#[from] Inner),
}

#[test]
fn nested_error_chain() {
    use std::error::Error;
    let e: Outer = Inner::Deep.into();
    assert_eq!(e.to_string(), "outer: deep failure");
    assert!(e.source().is_some());
}
