//! agentic-toolkit's installer (ADR 0009). The command line and, later, the
//! terminal UI are thin layers over these modules; only the engine writes files.

pub mod catalog;
pub mod check;
pub mod hash;
pub mod list;
pub mod manifest;
pub mod scope;
pub mod source;
pub mod status;
