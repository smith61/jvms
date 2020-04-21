
use crate::error::{JvmsError, Result};
use serde::{Deserialize, Serialize};
use std::{env, fs};
use std::collections::HashMap;
use std::path::{Component, Path, PathBuf};
use crate::shim::Shim;

pub struct JvmsInstallation {
    installation_path: PathBuf
}

#[derive(Deserialize, Serialize)]
pub struct JvmsConfiguration {
    toolchains: Option<HashMap<String, JavaToolchain>>,
    default: Option<String>,
    overrides: Option<Vec<JvmsOverride>>
}

#[derive(Deserialize, Serialize)]
pub struct JvmsOverride {
    pub path: PathBuf,
    pub toolchain: String
}

#[derive(Deserialize, Serialize)]
pub struct JavaToolchain {
    pub java_home: PathBuf
}

fn normalize_path(path: impl AsRef<Path>) -> PathBuf {
    let mut stack: Vec<Component> = vec![];

    // We assume .components() removes redundant consecutive path separators.
    // Note that .components() also does some normalization of '.' on its own anyways.
    // This '.' normalization happens to be compatible with the approach below.
    for component in path.as_ref().components() {
        match component {
            // Drop CurDir components, do not even push onto the stack.
            Component::CurDir => {},

            // For ParentDir components, we need to use the contents of the stack.
            Component::ParentDir => {
                // Look at the top element of stack, if any.
                let top = stack.last();
                match top {
                    // A component is on the stack, need more pattern matching.
                    Some(c) => {
                        match c {
                            // Push the ParentDir on the stack.
                            Component::Prefix(_) => { stack.push(component); },

                            // The parent of a RootDir is itself, so drop the ParentDir (no-op).
                            Component::RootDir => {},

                            // A CurDir should never be found on the stack, since they are dropped when seen.
                            Component::CurDir => { unreachable!(); },

                            // If a ParentDir is found, it must be due to it piling up at the start of a path.
                            // Push the new ParentDir onto the stack.
                            Component::ParentDir => { stack.push(component); },

                            // If a Normal is found, pop it off.
                            Component::Normal(_) => { let _ = stack.pop(); }
                        }
                    },

                    // Stack is empty, so path is empty, just push.
                    None => { stack.push(component); }
                }
            },

            // All others, simply push onto the stack.
            _ => { stack.push(component); },
        }
    }

    // If an empty PathBuf would be return, instead return CurDir ('.').
    if stack.is_empty() {
        stack.push(Component::CurDir);
    }

    let mut norm_path = PathBuf::new();
    for item in &stack {
        norm_path.push(item);
    }

    norm_path
}

fn make_absolute(path: impl AsRef<Path>) -> PathBuf {
    let path = path.as_ref();
    let absolute_path = if path.is_absolute() {
        path.to_path_buf()

    } else {
        env::current_dir()
            .expect("Failed to get current directory.")
            .join(path)
    };

    normalize_path(absolute_path)
}

impl JvmsInstallation {

    pub fn new(installation_path: PathBuf) -> JvmsInstallation {
        JvmsInstallation {
            installation_path
        }
    }

    pub fn get_current_installation() -> JvmsInstallation {
        let current_exe_dir =
            env::current_exe().expect("Could not locate the currently executing binary.");

        let installation_dir =
            current_exe_dir.parent().expect("Could not locate the currently executing installation directory.");

        JvmsInstallation::new(installation_dir.to_path_buf())
    }

    pub fn get_installation_path(&self) -> &Path {
        self.installation_path.as_path()
    }

    pub fn install_binaries(&self) -> Result<()> {
        let jvms_source_binary = env::current_exe().expect("Failed to get current executing binary.");

        fs::create_dir_all(&self.installation_path)?;

        //
        // Copy the jvms binary into the installation path.
        //

        let jvms_dest_binary = {
            let mut path = self.installation_path.clone();
            path.push("jvms");

            #[cfg(target_os="windows")]
            {
                assert!(path.set_extension("exe"));
            }

            path
        };

        println!("Copying {:?} to {:?}", jvms_source_binary, jvms_dest_binary);
        fs::copy(&jvms_source_binary, &jvms_dest_binary)?;

        //
        // Create a hard link from all shims to the destination jvms binary.
        //

        let mut source_path = self.installation_path.clone();
        for shim in Shim::get_shims() {
            source_path.push(shim.name);

            #[cfg(target_os="windows")]
            {
                assert!(source_path.set_extension("exe"));
            }

            println!("Linking {:?} to {:?}", source_path, jvms_dest_binary);
            fs::hard_link(&jvms_dest_binary, &source_path)?;
            assert!(source_path.pop());
        }

        Ok(())
    }

    pub fn load_configuration(&self) -> Result<JvmsConfiguration> {
        let config_file_path = self.get_config_file_path();
        if !config_file_path.is_file() {
            return Ok(JvmsConfiguration::new());
        }

        let reader =
            fs::File::open(config_file_path)
                .map_err(|io_error| {
                    JvmsError::InvalidConfiguration(format!("Failed to open jvms configuration file: {:?}", io_error))
                })?;

        let config =
            serde_json::from_reader(reader)
                .map_err(|serde_error| {
                    JvmsError::InvalidConfiguration(format!("Failed to parse jvms configuration file: {:?}", serde_error))
                })?;

        Ok(config)
    }

    pub fn save_configuration(&self, config: &JvmsConfiguration, force: bool) -> Result<()> {
        if !force {
            config.validate_configuration()?;
        }

        let config_file_path = self.get_config_file_path();
        let writer =
            fs::File::create(config_file_path)
                .map_err(|io_error| {
                    JvmsError::InvalidConfiguration(format!("Failed to open jvms configuration file: {:?}", io_error))
                })?;

        serde_json::to_writer_pretty(writer, config)
            .map_err(|serde_error| {
                JvmsError::InvalidConfiguration(format!("Failed to write jvms configuration file: {:?}", serde_error))
            })
    }

    fn get_config_file_path(&self) -> PathBuf {
        let mut installation_path = self.installation_path.clone();
        installation_path.push("jvms.conf");
        installation_path
    }

}

impl JvmsConfiguration {

    pub fn new() -> JvmsConfiguration {
        JvmsConfiguration {
            toolchains: None,
            default: None,
            overrides: None
        }
    }

    pub fn get_toolchain(&self, toolchain_name: &str) -> Option<&JavaToolchain> {
        self.toolchains
            .as_ref()
            .map(|i| i.get(toolchain_name))
            .flatten()
    }

    pub fn get_default_toolchain(&self) -> Option<&JavaToolchain> {
        self.get_default_toolchain_name()
            .map(|name| self.get_toolchain(name))
            .flatten()
    }

    pub fn get_environment_toolchain(&self, environment_path: &Path) -> Option<&JavaToolchain> {
        let environment_path = make_absolute(environment_path);
        let mut best_override: Option<&JvmsOverride> = None;
        if let Some(overrides) = &self.overrides {
            for ovrride in overrides {
                if !environment_path.starts_with(&ovrride.path) {
                    continue;
                }

                if best_override.map(|o| ovrride.path.starts_with(&o.path)).unwrap_or(true) {
                    best_override = Some(ovrride);
                }
            }
        }

        best_override
            .map(|o| self.get_toolchain(&o.toolchain))
            .flatten()
    }

    pub fn has_toolchain(&self, toolchain_name: &str) -> bool {
        self.get_toolchain(toolchain_name).is_some()
    }

    pub fn get_default_toolchain_name(&self) -> Option<&str> {
        self.default.as_ref().map(|v| v.as_str())
    }

    pub fn set_default_toolchain_name(&mut self, toolchain_name: String) {
        self.default = Some(toolchain_name);
    }

    pub fn add_toolchain(&mut self, toolchain_name: String, java_home: PathBuf) {
        let java_home = make_absolute(java_home);
        if self.toolchains.is_none() {
            self.toolchains = Some(HashMap::new());
        }

        self.toolchains.as_mut().unwrap().insert(toolchain_name, JavaToolchain::new(java_home));
    }

    pub fn get_toolchains(&self) -> impl Iterator<Item = (&String, &JavaToolchain)> {
        self.toolchains
            .iter()
            .map(|i| i.iter())
            .flatten()
    }

    pub fn remove_toolchain(&mut self, toolchain_name: &str) {
        if let Some(toolchains) = self.toolchains.as_mut() {
            toolchains.remove(toolchain_name);
        }
    }

    pub fn add_override(&mut self, path: &Path, toolchain_name: String) {
        if self.overrides.is_none() {
            self.overrides = Some(Vec::new());
        }

        self.overrides
            .as_mut()
            .unwrap()
            .push(JvmsOverride {
                path: make_absolute(path),
                toolchain: toolchain_name
            });
    }

    pub fn clean_overrides(&mut self) {
        if let Some(overrides) = self.overrides.as_mut() {
            overrides.retain(|o| o.path.exists());
        }
    }

    pub fn get_overrides(&self) -> Option<&[JvmsOverride]> {
        self.overrides.as_ref().map(|o| &**o)
    }

    pub fn remove_override(&mut self, override_path: &Path) {
        let override_path = make_absolute(override_path);
        if let Some(overrides) = self.overrides.as_mut() {
            overrides.retain(|o| o.path != override_path)
        }
    }

    pub fn validate_configuration(&self) -> Result<()> {
        if self.toolchains.is_none() || self.toolchains.as_ref().unwrap().is_empty() {
            return Err(JvmsError::InvalidConfiguration("Configuration has no installations.".to_owned()));
        }

        for toolchain in self.toolchains.as_ref().unwrap() {
            if !toolchain.1.java_home.exists() {
                return Err(JvmsError::InvalidConfiguration(format!("Installation {} does not point to a valid java home.", toolchain.0)));
            }
        }

        if let Some(default) = self.get_default_toolchain_name() {
            if !self.has_toolchain(default) {
                return Err(JvmsError::InvalidConfiguration(format!("Default installation references an unknown installation: {}", default)))
            }

        } else {
            return Err(JvmsError::InvalidConfiguration("Configuration does not have a default installation.".to_owned()));
        }

        if let Some(overrides) = self.overrides.as_ref() {
            for o in overrides {
                if !self.has_toolchain(&o.toolchain) {
                    return Err(JvmsError::InvalidConfiguration(format!("Override at {:?} references an unknown installation: {}", o.path, o.toolchain)));
                }
            }
        }

        Ok(())
    }

}

impl JavaToolchain {

    pub fn new(java_home: PathBuf) -> JavaToolchain {
        JavaToolchain {
            java_home
        }
    }

}
