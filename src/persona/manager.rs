//! # Daegonica Module: persona::manager
//!
//! **Purpose:** Hot-reload persona management system with file watching
//!
//! **Context:**
//! - Provides dynamic persona loading from YAML files
//! - Automatically reloads personas when files change
//! - (Note: Currently implemented but not integrated into main application flow)
//!
//! **Responsibilities:**
//! - Load all persona configurations from a directory
//! - Watch for file changes and reload automatically
//! - Provide thread-safe access to persona configurations
//! - Maintain in-memory cache of loaded personas
//!
//! **Author:** Daegonica Software
//! **Version:** 0.1.0
//! **Last Updated:** 2026-01-20
//!
//! ---------------------------------------------------------------
//! This file is part of the Daegonica Software codebase.
//! ---------------------------------------------------------------

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::path::PathBuf;
use tokio::sync::RwLock;
use notify::{RecommendedWatcher, RecursiveMode, Watcher, Event, EventKind};
use crate::persona::{Persona, PersonaRef};

/// # PersonaManager
///
/// **Summary:**
/// Manages a directory of persona YAML files with automatic hot-reload capability.
///
/// **Fields:**
/// - `dir`: Directory path containing persona YAML files
/// - `inner`: Thread-safe map of persona names to loaded configurations
/// - `_watcher`: File system watcher for automatic reloading
///
/// **Usage Example:**
/// ```rust
/// let manager = PersonaManager::new(PathBuf::from("personas")).await?;
/// if let Some(persona) = manager.get("shadow").await {
///     println!("Loaded: {}", persona.name);
/// }
/// ```
pub struct PersonaManager {
    dir: PathBuf,
    inner: Arc<RwLock<HashMap<String, PersonaRef>>>,
    _watcher: Mutex<Option<RecommendedWatcher>>>,
}

impl PersonaManager {
    /// # new
    ///
    /// **Purpose:**
    /// Creates a new PersonaManager and loads all personas from the specified directory.
    ///
    /// **Parameters:**
    /// - `dir`: Path to directory containing persona YAML files
    ///
    /// **Returns:**
    /// `anyhow::Result<Self>` - Initialized manager or error
    ///
    /// **Errors / Failures:**
    /// - Directory not found
    /// - Permission denied
    /// - Invalid YAML files in directory
    /// - File watcher initialization failures
    ///
    /// **Examples:**
    /// ```rust
    /// let manager = PersonaManager::new(PathBuf::from("personas")).await?;
    /// ```
    pub async fn new(dir: PathBuf) -> anyhow::Result<Self> {
        let inner = Arc::new(RwLock::new(HashMap::new()));
        let mut mgr = PersonaManager {
            dir: dir.clone(),
            inner: inner.clone(),
            _watcher: Mutex::new(None),
        };
        mgr.load_all().await?;
        mgr.start_watcher()?;
        Ok(mgr)
    }

    /// # load_all
    ///
    /// **Purpose:**
    /// Loads all YAML persona files from the manager's directory.
    ///
    /// **Parameters:**
    /// None
    ///
    /// **Returns:**
    /// `anyhow::Result<()>` - Success or error
    ///
    /// **Errors / Failures:**
    /// - Directory read failures
    /// - Invalid YAML syntax
    /// - Missing required persona fields
    ///
    /// **Details:**
    /// Replaces the entire internal persona map with newly loaded personas
    pub async fn load_all(&self) -> anyhow::Result<()> {
        let mut map = HashMap::new();
        for entry in std::fs::read_dir(&self.dir)? {
            let e = entry?;
            if e.path().extension().and_then(|s| s.to_str()) == Some("yaml") {
                let p = Persona::from_yaml_file(&e.path())?;
                map.insert(p.name.clone(), Arc::new(p));
            }
        }
        let mut guard = self.inner.write().await;
        *guard = map;
        Ok(())
    }

    /// # get
    ///
    /// **Purpose:**
    /// Retrieves a persona by name from the loaded personas.
    ///
    /// **Parameters:**
    /// - `name`: The persona name to look up
    ///
    /// **Returns:**
    /// `Option<PersonaRef>` - Arc-wrapped persona if found, None otherwise
    ///
    /// **Errors / Failures:**
    /// - None (returns None for missing personas)
    ///
    /// **Examples:**
    /// ```rust
    /// if let Some(persona) = manager.get("shadow").await {
    ///     println!("Found: {}", persona.name);
    /// }
    /// ```
    pub async fn get(&self, name: &str) -> Option<PersonaRef> {
        let guard = self.inner.read().await;
        guard.get(name).cloned()
    }

    /// # list
    ///
    /// **Purpose:**
    /// Returns a list of all currently loaded persona names.
    ///
    /// **Parameters:**
    /// None
    ///
    /// **Returns:**
    /// `Vec<String>` - Vector of persona names
    ///
    /// **Errors / Failures:**
    /// - None (infallible)
    ///
    /// **Examples:**
    /// ```rust
    /// let personas = manager.list().await;
    /// println!("Available: {:?}", personas);
    /// ```
    pub async fn list(&self) -> Vec<String> {
        let guard = self.inner.read().await;
        guard.keys().cloned().collect()
    }

    /// # start_watcher
    ///
    /// **Purpose:**
    /// Starts a file system watcher to automatically reload personas on file changes.
    ///
    /// **Parameters:**
    /// None
    ///
    /// **Returns:**
    /// `anyhow::Result<()>` - Success or watcher initialization error
    ///
    /// **Errors / Failures:**
    /// - File watcher initialization failures
    /// - Permission denied on directory
    ///
    /// **Details:**
    /// - Watches for Create, Modify, and Remove events
    /// - Automatically spawns async tasks to reload personas
    /// - Non-recursive (only watches immediate directory)
    fn start_watcher(&self) -> anyhow::Result<()> {
        let dir = self.dir.clone();
        let inner = self.inner.clone();
        let mut watcher = notify::recommended_watcher(move |res: Result<Event, _>| {
            if let Ok(event) = res {
                if matches!(event.kind, EventKind::Modify(_) | EventKind::Create(_) | EventKind::Remove(_)) {
                    let inner = inner.clone();
                    let dir = dir.clone();
                    tokio::spawn(async move {
                        let mut map = HashMap::new();
                        if let Ok(entries) = std::fs::read_dir(&dir) {
                            for entry in entries.flatten() {
                                if entry.path().extension().and_then(|s| s.to_str()) == Some("yaml") {
                                    if let Ok(p) = Persona::from_yaml_file(&entry.path()) {
                                        map.insert(p.name.clone(), Arc::new(p));
                                    }
                                }
                            }
                        }
                        let mut guard = inner.write().await;
                        *guard = map;
                    })
                }
            }
        })?;
        watcher.watch(&self.dir, RecursiveMode::NonRecursive)?;
        *self._watcher.lock().unwrap() =Some(watcher);
        Ok(())
    }
}