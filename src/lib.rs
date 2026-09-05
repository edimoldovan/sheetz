//! Sheetz — a fast native spreadsheet.
//!
//! The binary is a thin wrapper around this library so that integration tests
//! can drive the engine directly.

pub mod app;
pub mod commands;
pub mod engine;
pub mod fonts;
pub mod mcp;
pub mod keymap;
pub mod state;
pub mod theme;
pub mod ui;
