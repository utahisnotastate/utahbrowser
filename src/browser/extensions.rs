//! WASM-native extension runtime (wasmi sandbox). Vibe-coded modules live in the vault.

use crate::browser::storage_bridge;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use wasmi::{Engine, Instance, Linker, Module, Store};

/// Minimal Wasm module: `(func (export "on_event") (param i32) (result i32) local.get 0)`
const STUB_EXTENSION_WASM: &[u8] = &[
    0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00, 0x01, 0x06, 0x01, 0x60, 0x01, 0x7f, 0x01,
    0x7f, 0x03, 0x02, 0x01, 0x00, 0x07, 0x0a, 0x01, 0x06, 0x6f, 0x6e, 0x5f, 0x65, 0x76, 0x65,
    0x6e, 0x74, 0x00, 0x00, 0x0a, 0x06, 0x01, 0x00, 0x20, 0x00, 0x0b,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExtensionTrigger {
    #[serde(rename = "DOM_LOADED")]
    DomLoaded,
    #[serde(rename = "CLICK")]
    Click,
    #[serde(rename = "NAVIGATION")]
    Navigation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UtahExtensionManifest {
    pub name: String,
    pub trigger: ExtensionTrigger,
    pub intent: String,
    pub wasm_path: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ExtensionInfo {
    pub name: String,
    pub trigger: String,
    pub intent: String,
}

struct LoadedExtension {
    store: Store<()>,
    instance: Instance,
}

/// Sandboxed Wasm extension host.
pub struct ExtensionRuntime {
    engine: Engine,
    instances: HashMap<String, LoadedExtension>,
    manifests: HashMap<String, UtahExtensionManifest>,
}

impl ExtensionRuntime {
    pub fn new() -> Result<Self> {
        storage_bridge::ensure_vault()?;
        Ok(Self {
            engine: Engine::default(),
            instances: HashMap::new(),
            manifests: HashMap::new(),
        })
    }

    pub fn list_manifests(&self) -> Result<Vec<ExtensionInfo>> {
        let dir = storage_bridge::extensions_dir();
        let mut out = Vec::new();
        if !dir.is_dir() {
            return Ok(out);
        }
        for entry in fs::read_dir(&dir)? {
            let entry = entry?;
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }
            let manifest_path = path.join("manifest.json");
            if manifest_path.is_file() {
                if let Ok(raw) = fs::read_to_string(&manifest_path) {
                    if let Ok(m) = serde_json::from_str::<UtahExtensionManifest>(&raw) {
                        out.push(ExtensionInfo {
                            name: m.name,
                            trigger: format!("{:?}", m.trigger),
                            intent: m.intent,
                        });
                    }
                }
            }
        }
        Ok(out)
    }

    /// Materialize a vibe-coded extension (stub Wasm until full AI compile pipeline ships).
    pub fn vibe_create(
        &mut self,
        name: &str,
        intent: &str,
        trigger: ExtensionTrigger,
    ) -> Result<UtahExtensionManifest> {
        let safe_name = sanitize_name(name);
        let ext_dir = storage_bridge::extensions_dir().join(&safe_name);
        fs::create_dir_all(&ext_dir)?;
        let wasm_path = ext_dir.join("module.wasm");
        fs::write(&wasm_path, STUB_EXTENSION_WASM)?;
        let manifest = UtahExtensionManifest {
            name: safe_name.clone(),
            trigger,
            intent: intent.to_string(),
            wasm_path: wasm_path.display().to_string(),
        };
        fs::write(
            ext_dir.join("manifest.json"),
            serde_json::to_string_pretty(&manifest)?,
        )?;
        self.load_from_manifest(&manifest)?;
        tracing::info!("[UTAH_RUNTIME] Extension {safe_name} injected (vibe Wasm stub).");
        Ok(manifest)
    }

    pub fn load_from_manifest(&mut self, manifest: &UtahExtensionManifest) -> Result<()> {
        let path = PathBuf::from(&manifest.wasm_path);
        let wasm = fs::read(&path)
            .with_context(|| format!("read extension wasm {}", path.display()))?;
        self.manifests.insert(manifest.name.clone(), manifest.clone());
        self.load_wasm(&manifest.name, &wasm)
    }

    pub fn load_all(&mut self) -> Result<usize> {
        let names: Vec<String> = self
            .list_manifests()?
            .into_iter()
            .map(|e| e.name)
            .collect();
        let mut n = 0;
        for name in names {
            let path = storage_bridge::extensions_dir().join(&name).join("manifest.json");
            if path.is_file() {
                let raw = fs::read_to_string(&path)?;
                let m: UtahExtensionManifest = serde_json::from_str(&raw)?;
                self.load_from_manifest(&m)?;
                n += 1;
            }
        }
        Ok(n)
    }

    fn load_wasm(&mut self, name: &str, wasm: &[u8]) -> Result<()> {
        let module = Module::new(&self.engine, wasm).context("parse extension wasm")?;
        let mut store = Store::new(&self.engine, ());
        let linker = Linker::new(&self.engine);
        let instance = linker
            .instantiate(&mut store, &module)
            .and_then(|i| i.start(&mut store))
            .context("instantiate extension")?;
        self.instances.insert(name.to_string(), LoadedExtension { store, instance });
        Ok(())
    }

    pub fn dispatch(&mut self, trigger: ExtensionTrigger) -> Vec<(String, Result<i32>)> {
        let code = trigger_code(trigger);
        let names: Vec<String> = self
            .manifests
            .values()
            .filter(|m| m.trigger == trigger)
            .map(|m| m.name.clone())
            .collect();
        names
            .into_iter()
            .map(|name| {
                let result = self.execute(&name, code);
                (name, result)
            })
            .collect()
    }

    pub fn execute(&mut self, name: &str, action_code: i32) -> Result<i32> {
        let ext = self
            .instances
            .get_mut(name)
            .with_context(|| format!("extension not loaded: {name}"))?;
        let on_event = ext
            .instance
            .get_typed_func::<i32, i32>(&mut ext.store, "on_event")
            .context("extension must export on_event(i32) -> i32")?;
        on_event.call(&mut ext.store, action_code).context("on_event")
    }
}

fn trigger_code(trigger: ExtensionTrigger) -> i32 {
    match trigger {
        ExtensionTrigger::Navigation => 1,
        ExtensionTrigger::DomLoaded => 2,
        ExtensionTrigger::Click => 3,
    }
}

fn sanitize_name(name: &str) -> String {
    name.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect()
}
