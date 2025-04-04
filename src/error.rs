use image::{ImageError, error::DecodingError};
use std::io::ErrorKind;

#[derive(Debug)]
pub enum PDFConError {
    DirectoryReadError,
    FileReadError(std::io::Error),
    MozDecompressBufferError,
    MozDecompressStartError,
    MozDecompressFinishError,
    MozCompressStartError,
    MozCompressFinishError,
    MozUnwindError,
    OxiPngOptimizeError,
    ImageDecodingError(DecodingError),
    ImageErrorMisc,
    BufferInnerError,
    ThreadPoolCreationError(rayon::ThreadPoolBuildError),
    PageCreationError,
    PDFImageError,
    Misc(std::io::Error),
}

impl std::fmt::Display for PDFConError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            PDFConError::DirectoryReadError => write!(f, "Failed to read directory"),
            PDFConError::FileReadError(e) => write!(f, "Failed to read file: {}", e.to_string()),
            PDFConError::MozDecompressBufferError => {
                write!(f, "Mozjpeg failed to construct buffreader")
            }
            PDFConError::MozDecompressStartError => {
                write!(f, "Mozjpeg failed to start decompression")
            }
            PDFConError::MozDecompressFinishError => {
                write!(f, "Mozjpeg failed to finish decompression")
            }
            PDFConError::MozCompressStartError => {
                write!(f, "Mozjpeg failed to start compressing")
            }
            PDFConError::MozCompressFinishError => {
                write!(f, "Mozjpeg failed to finish compressing")
            }
            PDFConError::MozUnwindError => {
                write!(f, "Mozjpeg paniced. Error unknown")
            }
            PDFConError::OxiPngOptimizeError => {
                write!(f, "Optipng failed to optimize image")
            }
            PDFConError::ImageDecodingError(e) => {
                write!(f, "Failed to decode image: {}", e.to_string())
            }
            PDFConError::BufferInnerError => {
                write!(f, "Error getting the inner value of a buffer")
            }
            PDFConError::ImageErrorMisc => {
                write!(f, "Unknown image error")
            }
            PDFConError::PageCreationError => write!(f, "Failed to create page"),
            PDFConError::PDFImageError => write!(f, "Failed to create pdf image"),
            PDFConError::ThreadPoolCreationError(e) => {
                write!(f, "Failed to create threadpool: {}", e.to_string())
            }
            PDFConError::Misc(e) => write!(f, "Unknown error: {}", e.to_string()),
        }
    }
}

impl std::error::Error for PDFConError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            PDFConError::DirectoryReadError => None,
            PDFConError::FileReadError(e) => Some(e),
            PDFConError::MozDecompressBufferError => None,
            PDFConError::MozDecompressStartError => None,
            PDFConError::MozDecompressFinishError => None,
            PDFConError::MozCompressStartError => None,
            PDFConError::MozCompressFinishError => None,
            PDFConError::MozUnwindError => None,
            PDFConError::OxiPngOptimizeError => None,
            PDFConError::ImageDecodingError(e) => Some(e),
            PDFConError::BufferInnerError => None,
            PDFConError::ImageErrorMisc => None,
            PDFConError::PageCreationError => None,
            PDFConError::PDFImageError => None,
            PDFConError::ThreadPoolCreationError(e) => Some(e),
            PDFConError::Misc(e) => Some(e),
        }
    }
}

impl From<std::io::Error> for PDFConError {
    fn from(err: std::io::Error) -> PDFConError {
        match err.kind() {
            ErrorKind::NotADirectory => PDFConError::DirectoryReadError,
            ErrorKind::InvalidFilename => PDFConError::FileReadError(err),
            _ => PDFConError::Misc(err),
        }
    }
}

impl From<rayon::ThreadPoolBuildError> for PDFConError {
    fn from(err: rayon::ThreadPoolBuildError) -> PDFConError {
        PDFConError::ThreadPoolCreationError(err)
    }
}

impl From<ImageError> for PDFConError {
    fn from(err: ImageError) -> PDFConError {
        match err {
            ImageError::Decoding(e) => PDFConError::ImageDecodingError(e),
            _ => PDFConError::ImageErrorMisc,
        }
    }
}
