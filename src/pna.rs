use std::{borrow::Cow, slice::Iter};

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
    let mut png_rgba = buf_to_rgba(png_buf, pixel_size, png_info)?;
    let pna_alpha_mask = buf_to_alpha_mask(pna_buf, pixel_size, pna_info)?;

    for i in 0..pixel_size {
        if let (Some(target), Some(value)) = (png_rgba.get_mut(i * 4 + 3), pna_alpha_mask.get(i)) {
            *target = *value;
        } else {
            return Err(MergeError::LessDataSize);
        }
    }

    Ok(png_rgba)
}

fn buf_to_rgba(buf: &[u8], pixel_size: usize, info: &Info) -> Result<Vec<u8>, MergeError> {
    match info.color_type {
        ColorType::Grayscale => {
            let bytes = read_bytes_for_bit_depth_8(buf, &info.bit_depth)?;
            let mut result = Vec::new();

            let mut bytes_iter = bytes.iter();
            for _i in 0..pixel_size {
                match bytes_iter.next() {
                    Some(g) => {
                        result.push(*g);
                        result.push(*g);
                        result.push(*g);
                        result.push(u8::MAX);
                    }
                    None => return Err(MergeError::LessDataSize),
                }
            }

            return Ok(result);
        }
        ColorType::Rgb => {
            let bytes = read_bytes_for_bit_depth_8(buf, &info.bit_depth)?;
            let mut result = Vec::new();

            let mut bytes_iter = bytes.iter();
            for _i in 0..pixel_size {
                match (bytes_iter.next(), bytes_iter.next(), bytes_iter.next()) {
                    (Some(r), Some(g), Some(b)) => {
                        result.push(*r);
                        result.push(*g);
                        result.push(*b);
                        result.push(u8::MAX);
                    }
                    _ => return Err(MergeError::LessDataSize),
                }
            }

            return Ok(result);
        }
        ColorType::Indexed => {
            let indices = read_bytes_for_usize(buf, &info.bit_depth)?;
            let mut result: Vec<u8> = Vec::new();

            let palette = match &info.palette {
                Some(v) => split_palette(v)?,
                None => return Err(MergeError::PaletteNotFoundWhenIndexedPng),
            };

            let mut indices_iter = indices.iter();
            for _i in 0..pixel_size {
                match indices_iter.next() {
                    Some(i) => {
                        if let Some(p) = palette.get(*i) {
                            result.push(p[0]);
                            result.push(p[1]);
                            result.push(p[2]);
                            result.push(u8::MAX);
                        } else {
                            return Err(MergeError::InvalidIndexForPalette);
                        }
                    }
                    _ => return Err(MergeError::LessDataSize),
                }
            }

            return Ok(result);
        }
        ColorType::GrayscaleAlpha => {
            let bytes = read_bytes_for_bit_depth_8(buf, &info.bit_depth)?;
            let mut result = Vec::new();

            let mut bytes_iter = bytes.iter();
            for _i in 0..pixel_size {
                match (bytes_iter.next(), bytes_iter.next()) {
                    (Some(g), Some(a)) => {
                        result.push(*g);
                        result.push(*g);
                        result.push(*g);
                        result.push(*a);
                    }
                    _ => return Err(MergeError::LessDataSize),
                }
            }

            return Ok(result);
        }
        ColorType::Rgba => {
            let mut bytes = read_bytes_for_bit_depth_8(buf, &info.bit_depth)?;

            if bytes.len() < pixel_size * 4 {
                return Err(MergeError::LessDataSize);
            } else {
                bytes.resize(pixel_size * 4, 0);
                return Ok(bytes);
            }
        }
    }
}

fn buf_to_alpha_mask(buf: &[u8], pixel_size: usize, info: &Info) -> Result<Vec<u8>, MergeError> {
    match info.color_type {
        ColorType::Grayscale => {
            let mut bytes = read_bytes_for_bit_depth_8(buf, &info.bit_depth)?;

            if bytes.len() < pixel_size {
                return Err(MergeError::LessDataSize);
            } else {
                bytes.resize(pixel_size, 0);
                return Ok(bytes);
            }
        }
        ColorType::Rgb => {
            let bytes = read_bytes_for_bit_depth_8(buf, &info.bit_depth)?;
            let mut result = Vec::new();

            let mut bytes_iter = bytes.iter();
            for _i in 0..pixel_size {
                match (bytes_iter.next(), bytes_iter.next(), bytes_iter.next()) {
                    (Some(r), Some(g), Some(b)) => {
                        let v = (*r as u16 + *g as u16 + *b as u16) / 3;
                        result.push(v as u8);
                    }
                    _ => return Err(MergeError::LessDataSize),
                }
            }
            Ok(result)
        }
        ColorType::Indexed => {
            let indices = read_bytes_for_usize(buf, &info.bit_depth)?;
            let mut result = Vec::new();

            let palette = match &info.palette {
                Some(v) => split_palette(v)?,
                None => return Err(MergeError::PaletteNotFoundWhenIndexedPng),
            };

            let mut indices_iter = indices.iter();
            for _i in 0..pixel_size {
                match indices_iter.next() {
                    Some(i) => {
                        if let Some(p) = palette.get(*i) {
                            let v = (p[0] as u16 + p[1] as u16 + p[2] as u16) / 3;
                            result.push(v as u8);
                        } else {
                            return Err(MergeError::InvalidIndexForPalette);
                        }
                    }
                    None => return Err(MergeError::LessDataSize),
                }
            }
            Ok(result)
        }

        ColorType::GrayscaleAlpha => {
            let bytes = read_bytes_for_bit_depth_8(buf, &info.bit_depth)?;
            let mut result = Vec::new();

            let mut bytes_iter = bytes.iter();
            for _i in 0..pixel_size {
                match (bytes_iter.next(), bytes_iter.next()) {
                    (Some(g), Some(_a)) => {
                        // TODO: alpha blend?
                        result.push(*g);
                    }
                    _ => return Err(MergeError::LessDataSize),
                }
            }
            Ok(result)
        }
        ColorType::Rgba => {
            let bytes = read_bytes_for_bit_depth_8(buf, &info.bit_depth)?;
            let mut result = Vec::new();

            let mut bytes_iter = bytes.iter();
            for _i in 0..pixel_size {
                match (
                    bytes_iter.next(),
                    bytes_iter.next(),
                    bytes_iter.next(),
                    bytes_iter.next(),
                ) {
                    (Some(r), Some(g), Some(b), Some(_a)) => {
                        // TODO: alpha blend?
                        let v = (*r as u16 + *g as u16 + *b as u16) / 3;
                        result.push(v as u8);
                    }
                    _ => return Err(MergeError::LessDataSize),
                }
            }
            Ok(result)
        }
    }
}

fn read_bytes_for_bit_depth_8(buf: &[u8], bit_depth: &BitDepth) -> Result<Vec<u8>, MergeError> {
    let mut result = Vec::new();

    let mut buf_iter = buf.iter();
    let mut tmp = [0; 8];

    while let Some(buf_size) = read_byte_for_bit_depth_8(&mut buf_iter, &mut tmp, bit_depth)? {
        result.extend(tmp[0..buf_size].iter());
    }

    Ok(result)
}

fn read_byte_for_bit_depth_8(
    buf_iter: &mut Iter<u8>,
    output: &mut [u8; 8],
    bit_depth: &BitDepth,
) -> Result<Option<usize>, MergeError> {
    if let Some(t1) = buf_iter.next() {
        match bit_depth {
            BitDepth::One => {
                for i in 0..8 {
                    output[i] = if (*t1 << i) | 0b01111111 == u8::MAX {
                        u8::MAX
                    } else {
                        0
                    };
                }
                Ok(Some(8))
            }
            BitDepth::Two => {
                for i in 0..4 {
                    let mut v = 0;
                    if (*t1 << (i * 2)) | 0b01111111 == u8::MAX {
                        v += 0b10000000;
                    }
                    if (*t1 << (i * 2 + 1)) | 0b01111111 == u8::MAX {
                        v += 0b01111111;
                    }

                    output[i] = v;
                }
                Ok(Some(4))
            }
            BitDepth::Four => {
                for i in 0..2 {
                    let mut v = 0;
                    if (*t1 << (i * 4)) | 0b01111111 == u8::MAX {
                        v += 0b10000000;
                    }
                    if (*t1 << (i * 4 + 1)) | 0b01111111 == u8::MAX {
                        v += 0b01000000;
                    }
                    if (*t1 << (i * 4 + 2)) | 0b01111111 == u8::MAX {
                        v += 0b00100000;
                    }
                    if (*t1 << (i * 4 + 3)) | 0b01111111 == u8::MAX {
                        v += 0b00011111;
                    }
                    output[i] = v;
                }
                Ok(Some(2))
            }
            BitDepth::Eight => {
                output[0] = *t1;
                Ok(Some(1))
            }
            BitDepth::Sixteen => {
                if let Some(t2) = buf_iter.next() {
                    let v = (*t1 as u16) << 8 | *t2 as u16;
                    output[0] = (v >> 8) as u8;

                    Ok(Some(1))
                } else {
                    Err(MergeError::LessDataSize)
                }
            }
        }
    } else {
        Ok(None)
    }
}

fn read_bytes_for_usize(buf: &[u8], bit_depth: &BitDepth) -> Result<Vec<usize>, MergeError> {
    let mut result = Vec::new();

    let mut buf_iter = buf.iter();
    let mut tmp = [0; 8];

    while let Some(buf_size) = read_byte_for_usize(&mut buf_iter, &mut tmp, bit_depth)? {
        result.extend(tmp[0..buf_size].iter());
    }

    Ok(result)
}

fn read_byte_for_usize(
    buf_iter: &mut Iter<u8>,
    output: &mut [usize; 8],
    bit_depth: &BitDepth,
) -> Result<Option<usize>, MergeError> {
    if let Some(t1) = buf_iter.next() {
        match bit_depth {
            BitDepth::One => {
                for i in 0..8 {
                    output[i] = if (*t1 << i) | 0b01111111 == u8::MAX {
                        1
                    } else {
                        0
                    };
                }
                Ok(Some(8))
            }
            BitDepth::Two => {
                for i in 0..4 {
                    // VVxxxxxx -> 000000VV
                    output[i] = (((*t1 << (i * 2)) >> 6) & 0b00000011) as usize;
                }
                Ok(Some(4))
            }
            BitDepth::Four => {
                for i in 0..2 {
                    // VVVVxxxx -> 0000VVVV
                    output[i] = (((*t1 << (i * 4)) >> 4) & 0b00001111) as usize;
                }
                Ok(Some(2))
            }
            BitDepth::Eight => {
                output[0] = *t1 as usize;
                Ok(Some(1))
            }
            BitDepth::Sixteen => {
                if let Some(t2) = buf_iter.next() {
                    // 1111111122222222
                    let mut v = 0;
                    v += (*t1 as u16) << 8 & 0b1111111100000000;
                    v += *t2 as u16;
                    output[0] = v as usize;

                    Ok(Some(1))
                } else {
                    Err(MergeError::LessDataSize)
                }
            }
        }
    } else {
        Ok(None)
    }
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
            let pixel_size = (info.width * info.height) as usize;
            info.color_type = ColorType::Grayscale;
            info.bit_depth = BitDepth::Two;

            let result = buf_to_rgba(&buf, pixel_size, &info).unwrap();

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
            let pixel_size = (info.width * info.height) as usize;
            info.color_type = ColorType::Rgb;
            info.bit_depth = BitDepth::Four;

            let result = buf_to_rgba(&buf, pixel_size, &info).unwrap();

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
            let pixel_size = (info.width * info.height) as usize;
            info.color_type = ColorType::Indexed;
            info.bit_depth = BitDepth::One;
            let palette_raw = [255, 0, 0, 0, 0, 255];
            info.palette = Some(Cow::from(&palette_raw[..]));

            let result = buf_to_rgba(&buf, pixel_size, &info).unwrap();

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
                ]
            );
        }

        #[test]
        fn success_when_valid_buf_for_grayscale_alpha() {
            let buf = [0b11000000, 0b00110000, 0b00001100, 0b00000011];
            let mut info = Info::with_size(2, 1);
            let pixel_size = (info.width * info.height) as usize;
            info.color_type = ColorType::GrayscaleAlpha;
            info.bit_depth = BitDepth::Eight;

            let result = buf_to_rgba(&buf, pixel_size, &info).unwrap();

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
            let pixel_size = (info.width * info.height) as usize;
            info.color_type = ColorType::Rgba;
            info.bit_depth = BitDepth::Sixteen;

            let result = buf_to_rgba(&buf, pixel_size, &info).unwrap();

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
            let pixel_size = (info.width * info.height) as usize;
            info.color_type = ColorType::Grayscale;
            info.bit_depth = BitDepth::Two;

            let result = buf_to_alpha_mask(&buf, pixel_size, &info).unwrap();

            assert_eq!(result, vec![u8::MAX, 0, 0, 0,]);
        }

        #[test]
        fn success_when_valid_buf_for_rgb() {
            let buf = [0b11000000, 0b00001111, 0b11110000, 0b11000000, 0b00000000];
            let mut info = Info::with_size(3, 1);
            let pixel_size = (info.width * info.height) as usize;
            info.color_type = ColorType::Rgb;
            info.bit_depth = BitDepth::Four;

            let result = buf_to_alpha_mask(&buf, pixel_size, &info).unwrap();

            assert_eq!(result, vec![64, 170, 64]);
        }

        #[test]
        fn success_when_valid_buf_for_indexed() {
            let buf = [0b11000000];
            let mut info = Info::with_size(2, 2);
            let pixel_size = (info.width * info.height) as usize;
            info.color_type = ColorType::Indexed;
            info.bit_depth = BitDepth::One;
            let palette_raw = [255, 0, 0, 0, 0, 255];
            info.palette = Some(Cow::from(&palette_raw[..]));

            let result = buf_to_alpha_mask(&buf, pixel_size, &info).unwrap();

            assert_eq!(result, vec![85, 85, 85, 85]);
        }

        #[test]
        fn success_when_valid_buf_for_grayscale_alpha() {
            let buf = [0b11000000, 0b00110000, 0b00001100, 0b00000011];
            let mut info = Info::with_size(2, 1);
            let pixel_size = (info.width * info.height) as usize;
            info.color_type = ColorType::GrayscaleAlpha;
            info.bit_depth = BitDepth::Eight;

            let result = buf_to_alpha_mask(&buf, pixel_size, &info).unwrap();

            assert_eq!(result, vec![192, 12]);
        }

        #[test]
        fn success_when_valid_buf_for_rgba() {
            let buf = [
                0b11000000, 0b00110000, 0b00001100, 0b00000011, 0b11000000, 0b00110000, 0b00001100,
                0b00000011,
            ];
            let mut info = Info::with_size(1, 1);
            let pixel_size = (info.width * info.height) as usize;
            info.color_type = ColorType::Rgba;
            info.bit_depth = BitDepth::Sixteen;

            let result = buf_to_alpha_mask(&buf, pixel_size, &info).unwrap();

            assert_eq!(result, vec![132]);
        }
    }

    mod read_bytes_for_bit_depth_8 {
        use super::*;

        #[test]
        fn success_when_valid_bytes_loaded_by_one() {
            let buf = [0b00110110, 0b11001001];
            let bit_depth = BitDepth::Four;

            let result = read_bytes_for_bit_depth_8(&buf, &bit_depth).unwrap();

            assert_eq!(result, vec![0b00111111, 0b01100000, 0b11000000, 0b10011111]);
        }

        #[test]
        fn success_when_valid_bytes_loaded_by_two() {
            let buf = [0b00110110, 0b11001001];
            let bit_depth = BitDepth::Sixteen;

            let result = read_bytes_for_bit_depth_8(&buf, &bit_depth).unwrap();

            assert_eq!(result, vec![0b00110110]);
        }

        #[test]
        fn failed_when_invalid_bytes() {
            let buf = [0b00110110, 0b11001001, 0b11110000];
            let bit_depth = BitDepth::Sixteen;

            assert!(read_bytes_for_bit_depth_8(&buf, &bit_depth).is_err());
        }
    }

    mod read_byte_for_bit_depth_8 {
        use super::*;

        #[test]
        fn success_when_value_exists_for_bit_depth_1() {
            let buf = [0b00110110];
            let mut buf_iter = buf.iter();
            let mut output = [0; 8];
            let bit_depth = BitDepth::One;

            let buf_size =
                read_byte_for_bit_depth_8(&mut buf_iter, &mut output, &bit_depth).unwrap();

            assert_eq!(buf_size, Some(8));
            assert_eq!(output, [0, 0, u8::MAX, u8::MAX, 0, u8::MAX, u8::MAX, 0]);
            assert!(buf_iter.next().is_none());
        }

        #[test]
        fn success_when_value_exits_for_bit_depth_2() {
            let buf = [0b00110110];
            let mut buf_iter = buf.iter();
            let mut output = [0; 8];
            let bit_depth = BitDepth::Two;

            let buf_size =
                read_byte_for_bit_depth_8(&mut buf_iter, &mut output, &bit_depth).unwrap();

            assert_eq!(buf_size, Some(4));
            assert_eq!(output, [0, u8::MAX, 0b01111111, 0b10000000, 0, 0, 0, 0]);
            assert!(buf_iter.next().is_none());
        }

        #[test]
        fn success_when_value_exits_for_bit_depth_4() {
            let buf = [0b00110110];
            let mut buf_iter = buf.iter();
            let mut output = [0; 8];
            let bit_depth = BitDepth::Four;

            let buf_size =
                read_byte_for_bit_depth_8(&mut buf_iter, &mut output, &bit_depth).unwrap();

            assert_eq!(buf_size, Some(2));
            assert_eq!(output, [0b00111111, 0b01100000, 0, 0, 0, 0, 0, 0]);
            assert!(buf_iter.next().is_none());
        }

        #[test]
        fn success_when_value_exits_for_bit_depth_8() {
            let buf = [0b00110110];
            let mut buf_iter = buf.iter();
            let mut output = [0; 8];
            let bit_depth = BitDepth::Eight;

            let buf_size =
                read_byte_for_bit_depth_8(&mut buf_iter, &mut output, &bit_depth).unwrap();

            assert_eq!(buf_size, Some(1));
            assert_eq!(output, [0b00110110, 0, 0, 0, 0, 0, 0, 0]);
            assert!(buf_iter.next().is_none());
        }

        #[test]
        fn success_when_value_exits_for_bit_depth_16() {
            let buf = [0b00110110, 0b11001001];
            let mut buf_iter = buf.iter();
            let mut output = [0; 8];
            let bit_depth = BitDepth::Sixteen;

            let buf_size =
                read_byte_for_bit_depth_8(&mut buf_iter, &mut output, &bit_depth).unwrap();

            assert_eq!(buf_size, Some(1));
            assert_eq!(output, [0b00110110, 0, 0, 0, 0, 0, 0, 0]);
            assert!(buf_iter.next().is_none());
        }

        #[test]
        fn failed_when_less_value_for_bit_depth_16() {
            let buf = [0b00110110];
            let mut buf_iter = buf.iter();
            let mut output = [0; 8];
            let bit_depth = BitDepth::Sixteen;

            assert!(read_byte_for_bit_depth_8(&mut buf_iter, &mut output, &bit_depth).is_err());
        }

        #[test]
        fn success_but_none_when_no_values() {
            let buf = [0b00110110];
            let mut buf_iter = buf.iter();
            let mut output = [0; 8];
            let bit_depth = BitDepth::One;

            // using value.
            buf_iter.next();

            let buf_size =
                read_byte_for_bit_depth_8(&mut buf_iter, &mut output, &bit_depth).unwrap();

            assert_eq!(buf_size, None);
        }
    }

    mod read_bytes_for_usize {
        use super::*;

        #[test]
        fn success_when_valid_bytes_loaded_by_one() {
            let buf = [0b00110110, 0b11001001];
            let bit_depth = BitDepth::Four;

            let result = read_bytes_for_usize(&buf, &bit_depth).unwrap();

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

            let result = read_bytes_for_usize(&buf, &bit_depth).unwrap();

            assert_eq!(result, vec![0b0011011011001001 as usize]);
        }

        #[test]
        fn failed_when_invalid_bytes() {
            let buf = [0b00110110, 0b11001001, 0b11110000];
            let bit_depth = BitDepth::Sixteen;

            assert!(read_bytes_for_usize(&buf, &bit_depth).is_err());
        }
    }

    mod read_byte_for_usize {
        use super::*;

        #[test]
        fn success_when_value_exists_for_bit_depth_1() {
            let buf = [0b00110110];
            let mut buf_iter = buf.iter();
            let mut output = [0; 8];
            let bit_depth = BitDepth::One;

            let buf_size = read_byte_for_usize(&mut buf_iter, &mut output, &bit_depth).unwrap();

            assert_eq!(buf_size, Some(8));
            assert_eq!(output, [0, 0, 1, 1, 0, 1, 1, 0]);
            assert!(buf_iter.next().is_none());
        }

        #[test]
        fn success_when_value_exits_for_bit_depth_2() {
            let buf = [0b00110110];
            let mut buf_iter = buf.iter();
            let mut output = [0; 8];
            let bit_depth = BitDepth::Two;

            let buf_size = read_byte_for_usize(&mut buf_iter, &mut output, &bit_depth).unwrap();

            assert_eq!(buf_size, Some(4));
            assert_eq!(output, [0, 3, 1, 2, 0, 0, 0, 0]);
            assert!(buf_iter.next().is_none());
        }

        #[test]
        fn success_when_value_exits_for_bit_depth_4() {
            let buf = [0b00110110];
            let mut buf_iter = buf.iter();
            let mut output = [0; 8];
            let bit_depth = BitDepth::Four;

            let buf_size = read_byte_for_usize(&mut buf_iter, &mut output, &bit_depth).unwrap();

            assert_eq!(buf_size, Some(2));
            assert_eq!(output, [3, 6, 0, 0, 0, 0, 0, 0]);
            assert!(buf_iter.next().is_none());
        }

        #[test]
        fn success_when_value_exits_for_bit_depth_8() {
            let buf = [0b00110110];
            let mut buf_iter = buf.iter();
            let mut output = [0; 8];
            let bit_depth = BitDepth::Eight;

            let buf_size = read_byte_for_usize(&mut buf_iter, &mut output, &bit_depth).unwrap();

            assert_eq!(buf_size, Some(1));
            assert_eq!(output, [0b00110110 as usize, 0, 0, 0, 0, 0, 0, 0]);
            assert!(buf_iter.next().is_none());
        }

        #[test]
        fn success_when_value_exits_for_bit_depth_16() {
            let buf = [0b00110110, 0b11001001];
            let mut buf_iter = buf.iter();
            let mut output = [0; 8];
            let bit_depth = BitDepth::Sixteen;

            let buf_size = read_byte_for_usize(&mut buf_iter, &mut output, &bit_depth).unwrap();

            assert_eq!(buf_size, Some(1));
            assert_eq!(output, [0b0011011011001001 as usize, 0, 0, 0, 0, 0, 0, 0]);
            assert!(buf_iter.next().is_none());
        }

        #[test]
        fn failed_when_less_value_for_bit_depth_16() {
            let buf = [0b00110110];
            let mut buf_iter = buf.iter();
            let mut output = [0; 8];
            let bit_depth = BitDepth::Sixteen;

            assert!(read_byte_for_usize(&mut buf_iter, &mut output, &bit_depth).is_err());
        }

        #[test]
        fn success_but_none_when_no_values() {
            let buf = [0b00110110];
            let mut buf_iter = buf.iter();
            let mut output = [0; 8];
            let bit_depth = BitDepth::One;

            // using value.
            buf_iter.next();

            let buf_size = read_byte_for_usize(&mut buf_iter, &mut output, &bit_depth).unwrap();

            assert_eq!(buf_size, None);
        }
    }
}
