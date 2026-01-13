use sysinfo::System;

#[derive(Debug, Clone, Default)]
pub struct OsInfo {
    pub name: String,
    pub version: String,
    pub kernel_version: String,
    pub host_name: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OsType {
    Linux,
    Windows,
    MacOs,
    Other,
}

impl OsInfo {

    pub fn new() -> Self {
        Self {
            name: System::name().unwrap_or_default(),
            version: System::os_version().unwrap_or_default(),
            kernel_version: System::kernel_version().unwrap_or_default(),
            host_name: System::host_name().unwrap_or_default(),
        }
    }

    pub fn refresh(&mut self) {
        *self = Self::new();
    }

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

    pub fn display_name(&self) {
        println!("OS Name: {}", self.name);
    }

    pub fn display_version(&self) {
        println!("OS Version: {}", self.version);
    }

    pub fn display_kernel_version(&self) {
        println!("Kernel Version: {}", self.kernel_version);
    }

    pub fn display_host_name(&self) {
        println!("Host Name: {}", self.host_name);
    }

    pub fn display_all(&self) {
        self.display_name();
        self.display_version();
        self.display_kernel_version();
        self.display_host_name();
    }
}