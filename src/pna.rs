use std::{borrow::Cow, u8};

use png::{BitDepth, ColorType, Info};

use crate::error::MergeError;

pub(crate) fn merge_pna(
    png_buf: &[u8],
    png_info: &Info,
    pna_buf: &[u8],
    pna_info: &Info,
) -> Result<Vec<u8>, MergeError> {
    if png_info.width != pna_info.width || png_info.height != pna_info.height {
        return Err(MergeError::SizePngAndPnaAreDifferent);
    }

    let pixel_size = (png_info.width * png_info.height) as usize;

    let mut png_rgba = buf_to_rgba(png_buf, png_info)?;
    adjust_length(&mut png_rgba, pixel_size * 4)?;

    let mut pna_alpha_mask = buf_to_alpha_mask(pna_buf, pna_info)?;
    adjust_length(&mut pna_alpha_mask, pixel_size)?;

    Ok(png_rgba
        .chunks_exact(4)
        .zip(pna_alpha_mask.iter())
        .flat_map(|v| [v.0[0], v.0[1], v.0[2], *v.1])
        .collect())
}

fn adjust_length(buf: &mut Vec<u8>, size: usize) -> Result<(), MergeError> {
    if buf.len() < size {
        Err(MergeError::LessDataSize)
    } else {
        buf.resize(size, 0);
        Ok(())
    }
}

fn buf_to_rgba(buf: &[u8], info: &Info) -> Result<Vec<u8>, MergeError> {
    let bytes = match info.color_type {
        ColorType::Indexed => {
            return buf_to_rgba_from_indexed(buf, &info.bit_depth, info.palette.as_ref())
        }
        _ => read_bytes_for_bit_depth_8(buf, &info.bit_depth),
    };

    match info.color_type {
        ColorType::Grayscale => Ok(bytes.iter().flat_map(|v| [*v, *v, *v, u8::MAX]).collect()),
        ColorType::GrayscaleAlpha => Ok(bytes
            .chunks_exact(2)
            .flat_map(|v| [v[0], v[0], v[0], v[1]])
            .collect()),
        ColorType::Rgb => Ok(bytes
            .chunks_exact(3)
            .flat_map(|v| [v[0], v[1], v[2], u8::MAX])
            .collect()),
        ColorType::Rgba => Ok(bytes
            .chunks_exact(4)
            .flat_map(|v| [v[0], v[1], v[2], v[3]])
            .collect()),
        ColorType::Indexed => unreachable!("early returned."),
    }
}

fn buf_to_rgba_from_indexed(
    buf: &[u8],
    bit_depth: &BitDepth,
    palette_raw: Option<&Cow<[u8]>>,
) -> Result<Vec<u8>, MergeError> {
    let pallete = match palette_raw {
        Some(v) => split_palette(v)?,
        None => return Err(MergeError::PaletteNotFoundWhenIndexedPng),
    };
    let indices = read_bytes_for_usize(buf, bit_depth);

    indices
        .iter()
        .try_fold(Vec::new(), |mut acc, v| {
            pallete.get(*v).map(|p| {
                acc.push(p[0]);
                acc.push(p[1]);
                acc.push(p[2]);
                acc.push(u8::MAX);
                acc
            })
        })
        .ok_or(MergeError::InvalidIndexForPalette)
}

fn buf_to_alpha_mask(buf: &[u8], info: &Info) -> Result<Vec<u8>, MergeError> {
    let rgba = buf_to_rgba(buf, info)?;

    Ok(rgba
        .chunks_exact(4)
        .flat_map(|v| {
            // TODO: alpha blend?
            let v = (v[0] as u16 + v[1] as u16 + v[2] as u16) / 3;
            [v as u8]
        })
        .collect())
}

fn read_bytes_for_bit_depth_8(buf: &[u8], bit_depth: &BitDepth) -> Vec<u8> {
    match bit_depth {
        BitDepth::One => buf
            .iter()
            .flat_map(read_byte_depth_1)
            .map(|v| bit_to_u8(v, 1))
            .collect(),
        BitDepth::Two => buf
            .iter()
            .flat_map(read_byte_depth_2)
            .map(|v| bit_to_u8(v, 2))
            .collect(),
        BitDepth::Four => buf
            .iter()
            .flat_map(read_byte_depth_4)
            .map(|v| bit_to_u8(v, 4))
            .collect(),
        BitDepth::Eight => buf.to_vec(),
        BitDepth::Sixteen => buf.chunks_exact(2).flat_map(|v| [v[0]]).collect(),
    }
}

fn read_bytes_for_usize(buf: &[u8], bit_depth: &BitDepth) -> Vec<usize> {
    match bit_depth {
        BitDepth::One => buf
            .iter()
            .flat_map(read_byte_depth_1)
            .map(|v| v as usize)
            .collect(),
        BitDepth::Two => buf
            .iter()
            .flat_map(read_byte_depth_2)
            .map(|v| v as usize)
            .collect(),
        BitDepth::Four => buf
            .iter()
            .flat_map(read_byte_depth_4)
            .map(|v| v as usize)
            .collect(),
        BitDepth::Eight => buf.iter().map(|v| *v as usize).collect(),
        BitDepth::Sixteen => buf
            .chunks_exact(2)
            .flat_map(|v| [(((v[0] as usize) << 8) | v[1] as usize)])
            .collect(),
    }
}

fn bit_to_u8(v: u8, bit: u32) -> u8 {
    let v = v << (8 - bit);
    if v.trailing_zeros() == (8 - bit) {
        v | (u8::MAX >> bit)
    } else {
        v
    }
}

fn read_byte_depth_1(v: &u8) -> [u8; 8] {
    [
        (v & (1 << 7)) >> 7,
        (v & (1 << 6)) >> 6,
        (v & (1 << 5)) >> 5,
        (v & (1 << 4)) >> 4,
        (v & (1 << 3)) >> 3,
        (v & (1 << 2)) >> 2,
        (v & (1 << 1)) >> 1,
        (v & (1 << 0)),
    ]
}

fn read_byte_depth_2(v: &u8) -> [u8; 4] {
    [
        (v & ((1 << 6) | (1 << 7))) >> 6,
        (v & ((1 << 4) | (1 << 5))) >> 4,
        (v & ((1 << 2) | (1 << 3))) >> 2,
        (v & ((1 << 0) | (1 << 1))),
    ]
}

fn read_byte_depth_4(v: &u8) -> [u8; 2] {
    [
        (v & ((1 << 4) | (1 << 5) | (1 << 6) | (1 << 7))) >> 4,
        (v & ((1 << 0) | (1 << 1) | (1 << 2) | (1 << 3))),
    ]
}

fn split_palette(palette_raw: &Cow<[u8]>) -> Result<Vec<[u8; 3]>, MergeError> {
    let mut result = Vec::new();
    let palette_splited = palette_raw.chunks(3);

    for p in palette_splited {
        if p.len() != 3 {
            return Err(MergeError::InvalidPalette);
        }

        result.push([p[0], p[1], p[2]]);
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    mod merge_pna {
        use super::*;

        #[test]
        fn success_when_valid_param() {
            let png_buf = [u8::MAX, u8::MAX, u8::MAX, u8::MAX, u8::MAX, u8::MAX];
            let mut png_info = Info::with_size(2, 1);
            png_info.color_type = ColorType::Rgb;
            png_info.bit_depth = BitDepth::Eight;

            let pna_buf = [0, 0];
            let mut pna_info = Info::with_size(2, 1);
            pna_info.color_type = ColorType::Grayscale;
            pna_info.bit_depth = BitDepth::Eight;

            let result = merge_pna(&png_buf, &png_info, &pna_buf, &pna_info).unwrap();

            assert_eq!(
                result,
                vec![u8::MAX, u8::MAX, u8::MAX, 0, u8::MAX, u8::MAX, u8::MAX, 0]
            );
        }
    }

    mod buf_to_rgba {
        use super::*;

        #[test]
        fn success_when_valid_buf_for_grayscale() {
            let buf = [0b11000000];
            let mut info = Info::with_size(2, 2);
            info.color_type = ColorType::Grayscale;
            info.bit_depth = BitDepth::Two;

            let result = buf_to_rgba(&buf, &info).unwrap();

            assert_eq!(
                result,
                vec![
                    u8::MAX,
                    u8::MAX,
                    u8::MAX,
                    u8::MAX,
                    0,
                    0,
                    0,
                    u8::MAX,
                    0,
                    0,
                    0,
                    u8::MAX,
                    0,
                    0,
                    0,
                    u8::MAX
                ]
            );
        }

        #[test]
        fn success_when_valid_buf_for_rgb() {
            let buf = [0b11000000, 0b00001111, 0b11110000, 0b11000000, 0b00000000];
            let mut info = Info::with_size(3, 1);
            info.color_type = ColorType::Rgb;
            info.bit_depth = BitDepth::Four;

            let result = buf_to_rgba(&buf, &info).unwrap();

            assert_eq!(
                result,
                vec![
                    0b11000000,
                    0,
                    0,
                    u8::MAX,
                    u8::MAX,
                    u8::MAX,
                    0,
                    u8::MAX,
                    0b11000000,
                    0,
                    0,
                    u8::MAX,
                ]
            );
        }

        #[test]
        fn success_when_valid_buf_for_indexed() {
            let buf = [0b11000000];
            let mut info = Info::with_size(2, 2);
            info.color_type = ColorType::Indexed;
            info.bit_depth = BitDepth::One;
            let palette_raw = [255, 0, 0, 0, 0, 255];
            info.palette = Some(Cow::from(&palette_raw[..]));

            let result = buf_to_rgba(&buf, &info).unwrap();

            assert_eq!(
                result,
                vec![
                    0,
                    0,
                    255,
                    u8::MAX,
                    0,
                    0,
                    255,
                    u8::MAX,
                    255,
                    0,
                    0,
                    u8::MAX,
                    255,
                    0,
                    0,
                    u8::MAX,
                    255,
                    0,
                    0,
                    u8::MAX,
                    255,
                    0,
                    0,
                    u8::MAX,
                    255,
                    0,
                    0,
                    u8::MAX,
                    255,
                    0,
                    0,
                    u8::MAX,
                ]
            );
        }

        #[test]
        fn success_when_valid_buf_for_grayscale_alpha() {
            let buf = [0b11000000, 0b00110000, 0b00001100, 0b00000011];
            let mut info = Info::with_size(2, 1);
            info.color_type = ColorType::GrayscaleAlpha;
            info.bit_depth = BitDepth::Eight;

            let result = buf_to_rgba(&buf, &info).unwrap();

            assert_eq!(
                result,
                vec![
                    0b11000000, 0b11000000, 0b11000000, 0b00110000, 0b00001100, 0b00001100,
                    0b00001100, 0b00000011,
                ]
            );
        }

        #[test]
        fn success_when_valid_buf_for_rgba() {
            let buf = [
                0b11000000, 0b00110000, 0b00001100, 0b00000011, 0b11000000, 0b00110000, 0b00001100,
                0b00000011,
            ];
            let mut info = Info::with_size(1, 1);
            info.color_type = ColorType::Rgba;
            info.bit_depth = BitDepth::Sixteen;

            let result = buf_to_rgba(&buf, &info).unwrap();

            assert_eq!(
                result,
                vec![0b11000000, 0b00001100, 0b11000000, 0b00001100,]
            );
        }
    }

    mod buf_to_alpha_mask {

        use super::*;

        #[test]
        fn success_when_valid_buf_for_grayscale() {
            let buf = [0b11000000];
            let mut info = Info::with_size(2, 2);
            info.color_type = ColorType::Grayscale;
            info.bit_depth = BitDepth::Two;

            let result = buf_to_alpha_mask(&buf, &info).unwrap();

            assert_eq!(result, vec![u8::MAX, 0, 0, 0]);
        }

        #[test]
        fn success_when_valid_buf_for_rgb() {
            let buf = [0b11000000, 0b00001111, 0b11110000, 0b11000000, 0b00000000];
            let mut info = Info::with_size(3, 1);
            info.color_type = ColorType::Rgb;
            info.bit_depth = BitDepth::Four;

            let result = buf_to_alpha_mask(&buf, &info).unwrap();

            assert_eq!(result, vec![64, 170, 64]);
        }

        #[test]
        fn success_when_valid_buf_for_indexed() {
            let buf = [0b11000000];
            let mut info = Info::with_size(2, 2);
            info.color_type = ColorType::Indexed;
            info.bit_depth = BitDepth::One;
            let palette_raw = [255, 0, 0, 0, 0, 255];
            info.palette = Some(Cow::from(&palette_raw[..]));

            let result = buf_to_alpha_mask(&buf, &info).unwrap();

            assert_eq!(result, vec![85, 85, 85, 85, 85, 85, 85, 85]);
        }

        #[test]
        fn success_when_valid_buf_for_grayscale_alpha() {
            let buf = [0b11000000, 0b00110000, 0b00001100, 0b00000011];
            let mut info = Info::with_size(2, 1);
            info.color_type = ColorType::GrayscaleAlpha;
            info.bit_depth = BitDepth::Eight;

            let result = buf_to_alpha_mask(&buf, &info).unwrap();

            assert_eq!(result, vec![192, 12]);
        }

        #[test]
        fn success_when_valid_buf_for_rgba() {
            let buf = [
                0b11000000, 0b00110000, 0b00001100, 0b00000011, 0b11000000, 0b00110000, 0b00001100,
                0b00000011,
            ];
            let mut info = Info::with_size(1, 1);
            info.color_type = ColorType::Rgba;
            info.bit_depth = BitDepth::Sixteen;

            let result = buf_to_alpha_mask(&buf, &info).unwrap();

            assert_eq!(result, vec![132]);
        }
    }

    mod read_bytes_for_bit_depth_8 {
        use super::*;

        #[test]
        fn success_when_valid_bytes_loaded_by_one() {
            let buf = [0b00110110, 0b11001001];
            let bit_depth = BitDepth::Four;

            let result = read_bytes_for_bit_depth_8(&buf, &bit_depth);

            assert_eq!(result, vec![0b00111111, 0b01100000, 0b11000000, 0b10011111]);
        }

        #[test]
        fn success_when_valid_bytes_loaded_by_two() {
            let buf = [0b00110110, 0b11001001];
            let bit_depth = BitDepth::Sixteen;

            let result = read_bytes_for_bit_depth_8(&buf, &bit_depth);

            assert_eq!(result, vec![0b00110110]);
        }

        // #[test]
        // fn failed_when_invalid_bytes() {
        //     let buf = [0b00110110, 0b11001001, 0b11110000];
        //     let bit_depth = BitDepth::Sixteen;

        //     assert!(read_bytes_for_bit_depth_8(&buf, &bit_depth).is_err());
        // }
    }

    mod read_bytes_for_usize {
        use super::*;

        #[test]
        fn success_when_valid_bytes_loaded_by_one() {
            let buf = [0b00110110, 0b11001001];
            let bit_depth = BitDepth::Four;

            let result = read_bytes_for_usize(&buf, &bit_depth);

            assert_eq!(
                result,
                vec![
                    0b00000011 as usize,
                    0b00000110 as usize,
                    0b00001100 as usize,
                    0b00001001 as usize
                ]
            );
        }

        #[test]
        fn success_when_valud_bytes_loaded_by_two() {
            let buf = [0b00110110, 0b11001001];
            let bit_depth = BitDepth::Sixteen;

            let result = read_bytes_for_usize(&buf, &bit_depth);

            assert_eq!(result, vec![0b0011011011001001 as usize]);
        }

        // #[test]
        // fn failed_when_invalid_bytes() {
        //     let buf = [0b00110110, 0b11001001, 0b11110000];
        //     let bit_depth = BitDepth::Sixteen;

        //     assert!(read_bytes_for_usize(&buf, &bit_depth).is_err());
        // }
    }
}
