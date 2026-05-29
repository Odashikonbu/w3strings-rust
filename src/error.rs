use thiserror::Error;

#[derive(Error, Debug)]
pub enum W3StringsError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("CSV error: {0}")]
    Csv(#[from] csv::Error),

    #[error("Invalid file format: magic number must be 'RTSW'")]
    InvalidMagic,

    #[error("Unknown language key: {0:#010X}")]
    UnknownLanguageKey(u32),

    #[error("Unknown language handle: {0}")]
    UnknownLanguageHandle(String),

    #[error("Utf16 decoding error: {0}")]
    Utf16Error(#[from] std::string::FromUtf16Error),

    #[error("Invalid VLQ (Bit6) encoding")]
    InvalidVlq,
}

pub type Result<T> = std::result::Result<T, W3StringsError>;
