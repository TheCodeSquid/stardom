// waiting on https://github.com/rust-lang/rust/issues/84908
#![allow(unstable_name_collisions)]
#![warn(clippy::use_self)]

pub mod config;
pub mod project;
pub mod shell;

mod tools;
mod util;

pub use self::{shell::shell, util::is_error_silent};

#[cfg(feature = "cli")]
mod cli;

#[cfg(feature = "cli")]
pub use cli::run;
