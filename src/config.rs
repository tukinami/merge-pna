use std::{
    io::{BufRead, Write},
    path::PathBuf,
};

use clap::Parser;

use crate::error::ConfigError;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub(crate) struct ConfigRaw {
    /// Path to input png file.
    #[arg(short, long)]
    pub input_path: PathBuf,
    /// Path to pna file [default: PATH_TO_PNG_DIR/PNG_NAME.pna]
    #[arg(short, long)]
    pub pna_path: Option<PathBuf>,
    /// Path to output png file [default: PATH_TO_PNG_DIR/PNG_NAME_new.png]
    #[arg(short, long)]
    pub output_path: Option<PathBuf>,
    /// Flag of force overwriting output png.
    #[arg(short, long, default_value_t = false)]
    pub force: bool,
}

#[derive(Debug)]
pub(crate) struct Config {
    pub png_path: PathBuf,
    pub pna_path: PathBuf,
    pub output_path: PathBuf,
}

impl ConfigRaw {
    pub(crate) fn to_config_with_force_flag(self) -> Result<(Config, bool), ConfigError> {
        let png_path = self.input_path;
        if !png_path.exists() || !png_path.is_file() {
            return Err(ConfigError::PngIsNotExist);
        }

        let pna_path = if let Some(p) = self.pna_path {
            p
        } else {
            let mut p = png_path.clone();
            p.set_extension("pna");
            p
        };
        if !pna_path.exists() || !pna_path.is_file() {
            return Err(ConfigError::InvalidPnaPath);
        }

        let output_path = if let Some(p) = self.output_path {
            p
        } else {
            let mut p = png_path.clone();
            let mut p_file_name = p
                .file_stem()
                .expect("It's already checked that png file path is valid")
                .to_os_string();
            p_file_name.push("_new.png");

            p.set_file_name(p_file_name);
            p
        };

        Ok((
            Config {
                png_path,
                pna_path,
                output_path,
            },
            self.force,
        ))
    }
}

impl Config {
    pub(crate) fn confirm_overwriting(&self) -> Result<(), ConfigError> {
        if self.output_path.exists() {
            let stdin = std::io::stdin();
            let mut buf_reader = std::io::BufReader::new(stdin);

            let stdout = std::io::stdout();
            let stdout_lock = stdout.lock();
            let mut buf_writer = std::io::BufWriter::new(stdout_lock);

            let mut s = String::new();
            buf_writer.write_all(b"The output file already exists.\n")?;

            loop {
                buf_writer.write_all(b"Do you want to overwrite the file? [Y/n]: ")?;
                buf_writer.flush()?;

                s.clear();
                buf_reader.read_line(&mut s)?;

                match s.trim() {
                    "Y" => {
                        buf_writer.write_all(b"The file will be overwritten.\n")?;
                        buf_writer.flush()?;
                        break;
                    }
                    "n" => {
                        buf_writer.write_all(b"Closing this program...\n")?;
                        buf_writer.flush()?;
                        std::process::exit(0);
                    }
                    _ => {
                        buf_writer.write_all(
                            b"Please input 'Y' or 'n'. (for closing this program, input 'n')\n",
                        )?;
                    }
                }
            }
        }

        Ok(())
    }
}
