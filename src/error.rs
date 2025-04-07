use thiserror::Error;
#[derive(Error, Debug)]
pub enum PDFConError {
    #[error("IO error {0}")]
    IOError(#[from] std::io::Error),
    #[error("Failed to create the mozjpeg buffer")]
    MozDecompressBufferError,
    #[error("Failed to start mozjpeg decompression")]
    MozDecompressStartError,
    #[error("Failed to finish mozjpeg decompression")]
    MozDecompressFinishError,
    #[error("Failed to start mozjpeg compression")]
    MozCompressStartError,
    #[error("Failed to finish mozjpeg compression")]
    MozCompressFinishError,
    #[error("mozjpeg unwind. Something went wrong C side")]
    MozUnwindError,
    #[error("Oxipng optimization failed {0}")]
    OxiPngOptimizeError(#[from] oxipng::PngError),
    #[error("Image error {0}")]
    ImageError(#[from] image::ImageError),
    #[error("Failed to get buffer innner components")]
    BufferInnerError,
    #[error("Rayon threadpool creation error {0}")]
    ThreadPoolCreationError(#[from] rayon::ThreadPoolBuildError),
    #[error("lopdf error {0}")]
    LopdfError(#[from] lopdf::Error),
    #[error("Error encountered when unpacking pdf")]
    UnpackError,
}
