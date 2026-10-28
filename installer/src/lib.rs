//! agentic-toolkit's installer (ADR 0009). The command line and the terminal
//! UI are thin layers over these modules; only the engine writes files.

#![forbid(unsafe_code)]

pub mod brand;
pub mod catalog;
pub mod check;
pub mod engine;
pub mod hash;
pub mod list;
pub mod manifest;
pub mod scope;
pub mod source;
pub mod status;
pub mod tui;
