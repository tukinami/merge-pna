use clap::Parser;

pub(crate) mod config;
pub(crate) mod error;
pub(crate) mod pna;
pub(crate) mod process;

fn main() {
    let config_raw = config::ConfigRaw::parse();

    let (config, force_flag) = match config_raw.to_config_with_force_flag() {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Error on parsing argumets: {:?}", e);
            std::process::exit(1);
        }
    };

    if !force_flag {
        if let Err(e) = config.confirm_overwriting() {
            eprintln!("Error on confirm overwriting: {:?}", e);
            std::process::exit(1);
        }
    }

    if let Err(e) = process::process(config) {
        eprintln!("Error on merging png and pna: {:?}", e);
        std::process::exit(1);
    }
}
