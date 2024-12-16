#[derive(Debug)]
pub(crate) enum ConfigError {
    #[allow(dead_code)]
    Io(std::io::Error),
    PngIsNotExist,
    InvalidPnaPath,
}

#[derive(Debug)]
pub(crate) enum MergeError {
    #[allow(dead_code)]
    Io(std::io::Error),
    #[allow(dead_code)]
    DecodingError(png::DecodingError),
    #[allow(dead_code)]
    EncodingError(png::EncodingError),
    SizePngAndPnaAreDifferent,
    LessDataSize,
    PaletteNotFoundWhenIndexedPng,
    InvalidPalette,
    InvalidIndexForPalette,
}

impl From<std::io::Error> for ConfigError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}

impl From<std::io::Error> for MergeError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}

impl From<png::DecodingError> for MergeError {
    fn from(e: png::DecodingError) -> Self {
        Self::DecodingError(e)
    }
}

impl From<png::EncodingError> for MergeError {
    fn from(e: png::EncodingError) -> Self {
        Self::EncodingError(e)
    }
}
