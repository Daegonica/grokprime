//! # Daegonica Module: lib
//!
//! **Purpose:** Library root for GrokPrime-Brain crate
//!
//! **Context:**
//! - Central module declaration point for the entire GrokPrime-Brain library
//! - Exposes all sub-modules for use by main.rs and external consumers
//!
//! **Responsibilities:**
//! - Declare and export all public modules
//! - Provide a single entry point for library functionality
//! - Does NOT contain implementation logic (only module declarations)
//!
//! **Author:** Daegonica Software
//! **Version:** 0.1.0
//! **Last Updated:** 2026-01-18
//!
//! ---------------------------------------------------------------
//! This file is part of the Daegonica Software codebase.
//! ---------------------------------------------------------------

pub mod models;
pub mod grok;
pub mod user;
pub mod tui;
pub mod utilities;
pub mod twitter;
pub mod persona;
pub mod prelude;
