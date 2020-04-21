
use clap::Clap;
use crate::error::Result;
use crate::config::JvmsInstallation;
use std::env;
use std::path::PathBuf;

#[derive(Clap)]
#[clap(version = "0.1")]
pub struct Jvms {
    #[clap(subcommand)]
    command: JvmsCommand
}

#[derive(Clap)]
enum JvmsCommand {
    ///
    /// Change or read the current default installation name.
    ///
    #[clap(name = "default")]
    Default(DefaultCommand),

    ///
    /// Install JVMS into a new directory.
    ///
    #[clap(name = "install")]
    Install(InstallCommand),

    ///
    /// Add, remove, or list registered overrides.
    ///
    #[clap(name = "override")]
    Override(OverrideCommand),

    ///
    /// Add, remove, or list registered java toolchains.
    ///
    #[clap(name = "toolchain")]
    Toolchain(ToolchainCommand)
}

#[derive(Clap)]
struct DefaultCommand {
    ///
    /// If provided, sets the default toolchain to the toolchain with the provided name.
    ///
    toolchain: Option<String>,
    ///
    /// Force save configuration changes, even if configuration is invalid.
    ///
    #[clap(short = "f", long = "force")]
    force: bool
}

#[derive(Clap)]
struct InstallCommand {
    ///
    /// The directory in which to install JVMS.
    ///
    destination_path: PathBuf
}

#[derive(Clap)]
enum OverrideCommand {
    ///
    /// Cleans the override list of any override for a directory that no longer exists.
    ///
    #[clap(name = "clean")]
    Clean(OverrideCleanCommand),
    ///
    /// Lists all registered overrides.
    ///
    #[clap(name = "list")]
    List(OverrideListCommand),
    ///
    /// Removes the override for the provided directory.
    ///
    #[clap(name = "remove")]
    Remove(OverrideRemoveCommand),
    ///
    /// Sets the override for the current directory.
    ///
    #[clap(name = "set")]
    Set(OverrideSetCommand)
}

#[derive(Clap)]
struct OverrideListCommand { }

#[derive(Clap)]
struct OverrideCleanCommand {
    ///
    /// Force save configuration changes, even if configuration is invalid.
    ///
    #[clap(short = "f", long = "force")]
    force: bool
}

#[derive(Clap)]
struct OverrideRemoveCommand {
    ///
    /// The directory to remove the override for, defaults to the current working directory.
    ///
    path: Option<PathBuf>,
    ///
    /// Force save configuration changes, even if configuration is invalid.
    ///
    #[clap(short = "f", long = "force")]
    force: bool
}

#[derive(Clap)]
struct OverrideSetCommand {
    ///
    /// The name of the toolchain to use for this directory.
    ///
    toolchain_name: String,
    ///
    /// Force save configuration changes, even if configuration is invalid.
    ///
    #[clap(short = "f", long = "force")]
    force: bool
}

#[derive(Clap)]
enum ToolchainCommand {
    ///
    /// Adds a new java toolchain.
    ///
    #[clap(name = "add")]
    Add(ToolchainAddCommand),
    ///
    /// List registered java toolchains.
    ///
    #[clap(name = "list")]
    List(ToolchainListCommand),
    ///
    /// Removes a registered java toolchain.
    ///
    #[clap(name = "remove")]
    Remove(ToolchainRemoveCommand)
}

#[derive(Clap)]
struct ToolchainAddCommand {
    ///
    /// The name of the new toolchain.
    ///
    toolchain_name: String,
    ///
    /// The path to the JAVA_HOME of the toolchain.
    ///
    java_home: PathBuf,
    ///
    /// Force save configuration changes, even if configuration is invalid.
    ///
    #[clap(short = "f", long = "force")]
    force: bool
}

#[derive(Clap)]
struct ToolchainListCommand { }

#[derive(Clap)]
struct ToolchainRemoveCommand {
    ///
    /// The name of the toolchain to remove.
    ///
    toolchain_name: String,
    ///
    /// Force save configuration changes, even if configuration is invalid.
    ///
    #[clap(short = "f", long = "force")]
    force: bool
}

impl Jvms {

    pub fn execute(jvms_installation: &JvmsInstallation) -> Result<()> {
        let jvms_config = jvms_installation.load_configuration();

        let opts: Jvms = Jvms::parse();
        match opts.command {

            //
            // Default subcommand
            //

            JvmsCommand::Default(cmd) => {
                let mut config = jvms_config?;
                if let Some(toolchain_name) = cmd.toolchain {
                    if config.has_toolchain(&toolchain_name) {
                        println!("Setting default installation to {}", toolchain_name);
                        config.set_default_toolchain_name(toolchain_name);
                        jvms_installation.save_configuration(&config, cmd.force)?;

                    } else {
                        println!("No valid toolchain found for name: {}", toolchain_name);
                    }

                } else {
                    println!("Default installation: {}", config.get_default_toolchain_name().unwrap_or("None"));
                }
            }

            //
            // Install subcommand
            //

            JvmsCommand::Install(cmd) => {
                let new_installation = JvmsInstallation::new(cmd.destination_path);
                println!("Copying binaries to {:?}", new_installation.get_installation_path());
                if let Err(error) = new_installation.install_binaries() {
                    println!("Failed to copy binaries: {:?}", error);
                    return Ok(());
                }

                println!("Finished installing jvms to {:?}", new_installation.get_installation_path());
            },

            //
            // Override subcommands
            //

            JvmsCommand::Override(OverrideCommand::Clean(cmd)) => {
                let mut config = jvms_config?;
                config.clean_overrides();
                jvms_installation.save_configuration(&config, cmd.force)?;
            },
            JvmsCommand::Override(OverrideCommand::List(_)) => {
                let config = jvms_config?;
                println!("Registered overrides:");
                for o in config.get_overrides().unwrap_or(&[]) {
                    println!("  - {:?}:", o.path);
                    println!("    - Toolchain: {}", o.toolchain);
                }
            },
            JvmsCommand::Override(OverrideCommand::Remove(cmd)) => {
                let mut config = jvms_config?;
                let current_dir = env::current_dir().expect("Failed to get current working directory.");
                config.remove_override(&cmd.path.unwrap_or(current_dir));
                jvms_installation.save_configuration(&config, cmd.force)?;
            },
            JvmsCommand::Override(OverrideCommand::Set(cmd)) => {
                let mut config = jvms_config?;
                if config.has_toolchain(&cmd.toolchain_name) {
                    let current_dir = env::current_dir().expect("Failed to get current working directory.");
                    config.remove_override(&current_dir);
                    config.add_override(&current_dir, cmd.toolchain_name);
                    jvms_installation.save_configuration(&config, cmd.force)?;

                } else {
                    println!("No toolchain found for name: {}", cmd.toolchain_name);
                }
            },

            //
            // Toolchain subcommands
            //

            JvmsCommand::Toolchain(ToolchainCommand::Add(cmd)) => {
                let mut config = jvms_config?;
                if config.has_toolchain(&cmd.toolchain_name) {
                    println!("Installation already found for name: {}", cmd.toolchain_name);

                } else {
                    config.add_toolchain(cmd.toolchain_name, cmd.java_home);
                    jvms_installation.save_configuration(&config, cmd.force)?;
                }
            },
            JvmsCommand::Toolchain(ToolchainCommand::List(_)) => {
                let config = jvms_config?;
                println!("Available toolchains:");
                for toolchain in config.get_toolchains() {
                    println!("  - {}:", toolchain.0);
                    println!("    - JAVA_HOME = {:?}", toolchain.1.java_home);
                }
            },
            JvmsCommand::Toolchain(ToolchainCommand::Remove(cmd)) => {
                let mut config = jvms_config?;
                if config.has_toolchain(&cmd.toolchain_name) {
                    config.remove_toolchain(&cmd.toolchain_name);
                    jvms_installation.save_configuration(&config, cmd.force)?;

                } else {
                    println!("No toolchain found for name: {}", cmd.toolchain_name);
                }
            }
        }

        Ok(())
    }

}