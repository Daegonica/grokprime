use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::path::PathBuf;
use tokio::sync::RwLock;
use notify::{RecommendedWatcher, RecursiveMode, Watcher, Event, EventKind};
use crate::persona::{Persona, PersonaRef};

pub struct PersonaManager {
    dir: PathBuf,
    inner: Arc<RwLock<HashMap<String, PersonaRef>>>,
    _watcher: Mutex<Option<RecommendedWatcher>>>,
}

impl PersonaManager {
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

    pub async fn get(&self, name: &str) -> Option<PersonaRef> {
        let guard = self.inner.read().await;
        guard.get(name).cloned()
    }

    pub async fn list(&self) -> Vec<String> {
        let guard = self.inner.read().await;
        guard.keys().cloned().collect()
    }

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