use std::{fs::File, io::BufWriter};

use png::{Decoder, Encoder};

use crate::{config::Config, error::MergeError, pna::merge_pna};

pub(crate) fn process(config: Config) -> Result<(), MergeError> {
    let png_file = File::open(&config.png_path)?;
    let png_decoder = Decoder::new(png_file);
    let mut png_reader = png_decoder.read_info()?;
    let mut png_buf = vec![0; png_reader.output_buffer_size()];
    let _png_output_info = png_reader.next_frame(&mut png_buf)?;
    let png_info = png_reader.info();

    let pna_file = File::open(&config.pna_path)?;
    let pna_decoder = Decoder::new(pna_file);
    let mut pna_reader = pna_decoder.read_info()?;
    let mut pna_buf = vec![0; pna_reader.output_buffer_size()];
    let _pna_output_info = pna_reader.next_frame(&mut pna_buf)?;
    let pna_info = pna_reader.info();

    let merged_buf = merge_pna(&png_buf, &png_info, &pna_buf, &pna_info)?;

    let output_file = File::create(&config.output_path)?;
    let ref mut output_buf_writer = BufWriter::new(output_file);
    let mut output_encoder = Encoder::new(output_buf_writer, png_info.width, png_info.height);
    output_encoder.set_color(png::ColorType::Rgba);
    output_encoder.set_depth(png::BitDepth::Eight);
    let mut output_writer = output_encoder.write_header()?;
    output_writer.write_image_data(&merged_buf)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    mod process {
        use std::path::PathBuf;

        use super::*;

        #[test]
        fn success_when_valid_config() {
            let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("test_target/png");
            let png_path = dir.clone().join("surface0010.png");
            let pna_path = dir.clone().join("surface0010.pna");
            let output_path = dir.clone().join("surface0010_new.png");
            let config = Config {
                png_path,
                pna_path,
                output_path,
            };

            process(config).unwrap();
        }
    }
}
