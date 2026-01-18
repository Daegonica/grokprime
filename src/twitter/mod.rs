//! # Daegonica Module: twitter
//!
//! **Purpose:** Twitter/X API integration for posting tweets
//!
//! **Context:**
//! - Provides OAuth 1.0a authentication and tweet posting
//! - Used for automated tweet generation and posting features
//!
//! **Responsibilities:**
//! - Expose Twitter client and models
//! - Re-export commonly used types
//!
//! **Author:** Daegonica Software
//! **Version:** 0.1.0
//! **Last Updated:** 2026-01-18
//!
//! ---------------------------------------------------------------
//! This file is part of the Daegonica Software codebase.
//! ---------------------------------------------------------------

pub mod models;
pub mod client;

pub use client::TwitterConnection;
pub use models::*;