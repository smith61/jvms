#![allow(dead_code)]

mod config;
mod error;
mod jvms;
mod shim;

use config::JvmsInstallation;
use error::{JvmsError, Result};
use jvms::Jvms;
use shim::Shim;

fn main() {
    match run_main() {
        Ok(_) => {},
        Err(JvmsError::IoError(error)) => {
            eprintln!("IO Error has occurred: {:?}", error);
        },
        Err(JvmsError::InvalidConfiguration(string)) => {
            eprintln!("Configuration error: {}", string);
        },
        Err(JvmsError::SerdeJsonError(error)) => {
            eprintln!("Serde error has occurred: {:?}", error);
        }
    }
}

fn run_main() -> Result<()> {
    let jvms_installation = JvmsInstallation::get_current_installation();
    if let Some(shim) = Shim::get_current_shim()? {
        shim.execute(&jvms_installation)

    } else {
        Jvms::execute(&jvms_installation)
    }
}
