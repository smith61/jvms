
use crate::config::JvmsInstallation;
use crate::error::{Result, JvmsError};
use std::{env, io, process};

static JAVA_SHIMS: [Shim; 7] = [
    Shim {
        name: "jar"
    },
    Shim {
        name: "java"
    },
    Shim {
        name: "javac"
    },
    Shim {
        name: "javadoc"
    },
    Shim {
        name: "javah"
    },
    Shim {
        name: "javap"
    },
    Shim {
        name: "javaw"
    }
];

pub struct Shim {
    pub name: &'static str
}

impl Shim {

    pub fn get_shims() -> &'static [Shim] {
        &JAVA_SHIMS
    }

    pub fn get_current_shim() -> Result<Option<&'static Shim>> {
        let current_exe_path = env::current_exe()?;
        let current_exe_name =
            current_exe_path.file_stem()
                .ok_or(io::Error::from(io::ErrorKind::InvalidInput))?
                .to_string_lossy();

        for shim in Shim::get_shims() {
            if *shim.name == current_exe_name {
                return Ok(Some(shim));
            }
        }

        Ok(None)
    }

    pub fn execute(&self, jvms_installation: &JvmsInstallation) -> Result<()> {
        let jvms_config = jvms_installation.load_configuration()?;
        let current_dir = env::current_dir()?;
        let toolchain = if let Some(env_toolchain) = jvms_config.get_environment_toolchain(&current_dir) {
            env_toolchain

        } else if let Some(default_toolchain) = jvms_config.get_default_toolchain() {
            default_toolchain

        } else {
            return Err(JvmsError::InvalidConfiguration(format!("Failed to find toolchain for {:?} and default toolchain not configured.", current_dir)));
        };

        let exe_path = {
            let mut path = toolchain.java_home.clone();
            path.push("bin");
            path.push(self.name);

            #[cfg(target_os="windows")]
            {
                assert!(path.set_extension("exe"));
            }

            path
        };

        let mut command = process::Command::new(exe_path);
        command.env("JAVA_HOME", toolchain.java_home.as_os_str());

        env::args_os().skip(1).for_each(|arg| {
            command.arg(arg);
        });

        command.spawn()?.wait()?;
        Ok(())
    }

}
