#![deny(
    clippy::disallowed_methods,
    clippy::suspicious,
    clippy::style,
    missing_debug_implementations,
    missing_copy_implementations
)]
#![warn(clippy::pedantic)]

pub mod commands;
pub mod config;
pub mod constants;
pub mod opt;
pub mod schema;
