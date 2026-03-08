//! Typed context bridge for Molt's context cache.
//!
//! Molt uses a type-erased context cache (`save_context<T>` / `context<T>`)
//! to compensate for `CommandFunc` being a plain `fn` pointer (no closures).
//! This module wraps the cache with typed accessors.

use molt::Interp;
use molt::types::ContextID;

/// A typed context bridge over Molt's context cache.
///
/// Provides `inject<T>` to store typed data and `get<T>` to retrieve it.
/// Each injected value gets a unique `ContextID` that must be passed to
/// commands that need access to the context.
pub struct ContextBridge<'a> {
    interp: &'a mut Interp,
}

impl<'a> ContextBridge<'a> {
    /// Create a new bridge over the given interpreter.
    pub fn new(interp: &'a mut Interp) -> Self {
        Self { interp }
    }

    /// Inject a typed value into the context cache.
    ///
    /// Returns a `ContextID` that can be used with `add_context_command`
    /// and retrieved later via `get`.
    pub fn inject<T: 'static>(&mut self, data: T) -> ContextID {
        self.interp.save_context(data)
    }

    /// Get a mutable reference to a previously injected value.
    ///
    /// # Panics
    ///
    /// Panics if the `ContextID` does not correspond to a value of type `T`.
    /// This mirrors Molt's own `context<T>` behavior.
    pub fn get<T: 'static>(&mut self, id: ContextID) -> &mut T {
        self.interp.context::<T>(id)
    }

    /// Allocate a new `ContextID` without storing data.
    ///
    /// Useful when you need the ID before you have the data.
    pub fn allocate_id(&mut self) -> ContextID {
        self.interp.context_id()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inject_and_retrieve() {
        let mut interp = Interp::new();
        let mut bridge = ContextBridge::new(&mut interp);

        let id = bridge.inject(42_i32);
        let val = bridge.get::<i32>(id);
        assert_eq!(*val, 42);
    }

    #[test]
    fn inject_struct() {
        #[derive(Debug, PartialEq)]
        struct Config {
            name: String,
            count: usize,
        }

        let mut interp = Interp::new();
        let mut bridge = ContextBridge::new(&mut interp);

        let id = bridge.inject(Config {
            name: "test".into(),
            count: 7,
        });
        let cfg = bridge.get::<Config>(id);
        assert_eq!(cfg.name, "test");
        assert_eq!(cfg.count, 7);
    }

    #[test]
    fn mutate_through_bridge() {
        let mut interp = Interp::new();
        let mut bridge = ContextBridge::new(&mut interp);

        let id = bridge.inject(vec![1, 2, 3]);
        bridge.get::<Vec<i32>>(id).push(4);
        let v = bridge.get::<Vec<i32>>(id);
        assert_eq!(*v, vec![1, 2, 3, 4]);
    }
}
