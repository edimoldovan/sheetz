//! Model Context Protocol support.
//!
//! The server runs inside the GUI so an assistant edits the workbook the user
//! is looking at, and every change is visible as it happens. Starting Sheetz
//! any normal way (desktop launcher included) is all that is required — there
//! is no flag and no separate daemon.

pub mod bridge;
pub mod proto;
pub mod register;
pub mod skill;
pub mod templates;
pub mod tools;
