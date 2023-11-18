#[derive(Debug)]
pub(crate) enum ConfigError {
    Io(std::io::Error),
    PngIsNotExist,
    InvalidPnaPath,
}

#[derive(Debug)]
pub(crate) enum MergeError {
    Io(std::io::Error),
    DecodingError(png::DecodingError),
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
