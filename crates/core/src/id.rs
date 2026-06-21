//! Stable string ids that join the [`Catalog`](crate::Catalog) (content) to the
//! [`GameState`](crate::GameState) (live world).
//!
//! Targets and actions reference templates by id rather than by borrow, so the
//! state owns no borrows, stays cheaply `Clone`, and is trivially serialisable
//! later.

/// Stable string id for a target template (and the instances spawned from it),
/// e.g. `"hero"`. Used as the join key between [`Catalog`](crate::Catalog) and
/// [`GameState`](crate::GameState).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TargetId(pub String);

impl TargetId {
    /// Builds an id from anything string-like (`"hero"`, `String`, …).
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// Borrows the id as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&str> for TargetId {
    fn from(value: &str) -> Self {
        Self(value.to_owned())
    }
}

impl From<String> for TargetId {
    fn from(value: String) -> Self {
        Self(value)
    }
}

/// Stable string id for an action template, e.g. `"forest_exploration"`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ActionId(pub String);

impl ActionId {
    /// Builds an id from anything string-like.
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// Borrows the id as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&str> for ActionId {
    fn from(value: &str) -> Self {
        Self(value.to_owned())
    }
}

impl From<String> for ActionId {
    fn from(value: String) -> Self {
        Self(value)
    }
}
