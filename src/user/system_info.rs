//! # Daegonica Module: user::system_info
//!
//! **Purpose:** Operating system and host information retrieval
//!
//! **Context:**
//! - Provides system context for AI responses
//! - Used when user requests system information via 'system' command
//!
//! **Responsibilities:**
//! - Detect and report OS type, version, and kernel information
//! - Provide hostname and system details
//! - Format system information for display
//!
//! **Author:** Daegonica Software
//! **Version:** 0.1.0
//! **Last Updated:** 2026-01-18
//!
//! ---------------------------------------------------------------
//! This file is part of the Daegonica Software codebase.
//! ---------------------------------------------------------------

use sysinfo::System;

/// # OsInfo
///
/// **Summary:**
/// Container for operating system information gathered from the host machine.
///
/// **Fields:**
/// - `name`: Operating system name (e.g., "Windows", "Linux")
/// - `version`: OS version string
/// - `kernel_version`: Kernel or build version
/// - `host_name`: Network hostname of the machine
///
/// **Usage Example:**
/// ```rust
/// let os_info = OsInfo::new();
/// println!("{}", os_info.display_all());
/// ```
#[derive(Debug, Clone, Default)]
pub struct OsInfo {
    pub name: String,
    pub version: String,
    pub kernel_version: String,
    pub host_name: String,
}

/// # OsType
///
/// **Summary:**
/// Enumeration of supported operating system types for platform-specific behavior.
///
/// **Variants:**
/// - `Linux`: Linux-based operating systems
/// - `Windows`: Microsoft Windows
/// - `MacOs`: Apple macOS/Darwin
/// - `Other`: Unsupported or unknown OS
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OsType {
    Linux,
    Windows,
    MacOs,
    Other,
}

impl OsInfo {

    /// # new
    ///
    /// **Purpose:**
    /// Creates a new OsInfo instance by querying the current system.
    ///
    /// **Parameters:**
    /// None
    ///
    /// **Returns:**
    /// OsInfo populated with current system information
    ///
    /// **Errors / Failures:**
    /// - Falls back to empty strings if system information is unavailable
    ///
    /// **Examples:**
    /// ```rust
    /// let os_info = OsInfo::new();
    /// ```
    pub fn new() -> Self {
        Self {
            name: System::name().unwrap_or_default(),
            version: System::os_version().unwrap_or_default(),
            kernel_version: System::kernel_version().unwrap_or_default(),
            host_name: System::host_name().unwrap_or_default(),
        }
    }

    /// # refresh
    ///
    /// **Purpose:**
    /// Re-queries system information to update all fields.
    ///
    /// **Parameters:**
    /// None
    ///
    /// **Returns:**
    /// None (mutates self)
    ///
    /// **Errors / Failures:**
    /// - None (infallible)
    pub fn refresh(&mut self) {
        *self = Self::new();
    }

    /// # os_type
    ///
    /// **Purpose:**
    /// Determines the operating system type from the OS name.
    ///
    /// **Parameters:**
    /// None
    ///
    /// **Returns:**
    /// OsType enum variant representing the detected OS
    ///
    /// **Errors / Failures:**
    /// - Returns OsType::Other for unrecognized systems
    pub fn os_type(&self) -> OsType {
        let lower = self.name.to_lowercase();
        if lower.contains("linux") {
            OsType::Linux
        } else if lower.contains("windows") {
            OsType::Windows
        } else if lower.contains("mac") || lower.contains("darwin") {
            OsType::MacOs
        } else {
            OsType::Other
        }
    }

    /// # display_name
    ///
    /// **Purpose:**
    /// Formats the OS name for display.
    ///
    /// **Returns:**
    /// Formatted string with OS name
    pub fn display_name(&self) -> String {
        format!("OS Name: {}", self.name)
    }

    /// # display_version
    ///
    /// **Purpose:**
    /// Formats the OS version for display.
    ///
    /// **Returns:**
    /// Formatted string with OS version
    pub fn display_version(&self) -> String {
        format!("OS Version: {}", self.version)
    }

    /// # display_kernel_version
    ///
    /// **Purpose:**
    /// Formats the kernel version for display.
    ///
    /// **Returns:**
    /// Formatted string with kernel version
    pub fn display_kernel_version(&self) -> String {
        format!("Kernel Version: {}", self.kernel_version)
    }

    /// # display_host_name
    ///
    /// **Purpose:**
    /// Formats the hostname for display.
    ///
    /// **Returns:**
    /// Formatted string with hostname
    pub fn display_host_name(&self) -> String {
        format!("Host Name: {}", self.host_name)
    }

    /// # display_all
    ///
    /// **Purpose:**
    /// Formats all system information as a multi-line string.
    ///
    /// **Returns:**
    /// Complete formatted system information
    ///
    /// **Examples:**
    /// ```rust
    /// let output = os_info.display_all();
    /// println!("{}", output);
    /// ```
    pub fn display_all(&self)  -> String {
        format!(
            "{}\n{}\n{}\n{}",
            self.display_name(),
            self.display_version(),
            self.display_kernel_version(),
            self.display_host_name(),
        )
    }
}