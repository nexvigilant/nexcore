//! Type-safe ID newtypes for the PRPaaS domain.
//!
//! Every entity gets its own UUID newtype to prevent accidental mixing
//! (e.g., passing a TenantId where a UserId is expected is a compile error).

use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

macro_rules! define_id {
    ($name:ident, $prefix:expr) => {
        #[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
        #[serde(transparent)]
        pub struct $name(Uuid);

        impl $name {
            #[must_use]
            pub fn new() -> Self {
                Self(Uuid::new_v4())
            }

            #[must_use]
            pub fn from_uuid(id: Uuid) -> Self {
                Self(id)
            }

            #[must_use]
            pub fn as_uuid(&self) -> &Uuid {
                &self.0
            }

            #[must_use]
            pub fn into_uuid(self) -> Uuid {
                self.0
            }

            /// Parse from string, returning None on invalid UUID.
            #[must_use]
            pub fn parse(s: &str) -> Option<Self> {
                Uuid::parse_str(s).ok().map(Self)
            }
        }

        impl Default for $name {
            fn default() -> Self {
                Self::new()
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}_{}", $prefix, self.0)
            }
        }

        impl fmt::Debug for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}({})", stringify!($name), self.0)
            }
        }

        impl From<Uuid> for $name {
            fn from(id: Uuid) -> Self {
                Self(id)
            }
        }

        impl From<$name> for Uuid {
            fn from(id: $name) -> Self {
                id.0
            }
        }
    };
}

define_id!(TenantId, "ten");
define_id!(UserId, "usr");
define_id!(ProgramId, "prg");
define_id!(CompoundId, "cpd");
define_id!(AssayId, "asy");
define_id!(OrderId, "ord");
define_id!(ProviderId, "prv");
define_id!(ModelId, "mdl");
define_id!(InvoiceId, "inv");
define_id!(SignalId, "sig");
define_id!(DealId, "deal");
define_id!(AssetId, "ast");

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn ids_are_not_interchangeable() {
        let tenant = TenantId::new();
        let user = UserId::new();
        assert_ne!(tenant.as_uuid(), user.as_uuid());
    }

    #[test]
    fn display_includes_prefix() {
        let id = TenantId::from_uuid(Uuid::nil());
        assert!(id.to_string().starts_with("ten_"));
    }

    #[test]
    fn roundtrip_serde() {
        let id = TenantId::new();
        let json = serde_json::to_string(&id).unwrap();
        let back: TenantId = serde_json::from_str(&json).unwrap();
        assert_eq!(id, back);
    }

    #[test]
    fn parse_valid_uuid() {
        let id = TenantId::new();
        let uuid_str = id.as_uuid().to_string();
        let parsed = TenantId::parse(&uuid_str);
        assert_eq!(parsed, Some(id));
    }

    #[test]
    fn parse_invalid_returns_none() {
        assert_eq!(TenantId::parse("not-a-uuid"), None);
    }
}
