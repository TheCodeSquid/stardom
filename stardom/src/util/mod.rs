// with codegen:
pub mod attrs;
pub mod elements;
pub mod events;

pub mod lists;

// Message structs for preventing invalid attribute and event statements at compile-time.

#[doc(hidden)]
#[allow(non_camel_case_types)]
pub struct this_node_to_be_an_element;

#[doc(hidden)]
#[allow(non_camel_case_types)]
pub struct a_fragment_node;

#[doc(hidden)]
#[allow(non_camel_case_types)]
pub struct a_component;
