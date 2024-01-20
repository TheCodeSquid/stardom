// with codegen:
pub mod attrs;
pub mod elements;
pub mod events;

pub mod lists;

// Message structs for preventing invalid attribute and event statements at compile-time.

#[doc(hidden)]
pub struct ThisNodeToBeAnElement;

#[doc(hidden)]
pub struct AFragmentNode;

#[doc(hidden)]
pub struct AComponent;
