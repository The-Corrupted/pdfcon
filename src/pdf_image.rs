use std::io::{BufWriter, Write};

use crate::error::PDFConError;
use flate2::write::ZlibEncoder;
use image::{ImageEncoder, codecs::png};
use log::error;
use oxipng;
use std::path::PathBuf;

pub enum PDFConColorSpace {
    RGB8,
    RGB16,
    L8,
    L16,
    CMYK,
}

impl PDFConColorSpace {
    pub fn to_pdf_format(&self) -> (Vec<u8>, u32) {
        match *self {
            Self::RGB8 => (b"DeviceRGB".to_vec(), 8),
            Self::RGB16 => (b"DeviceRGB".to_vec(), 16),
            Self::L8 => (b"DeviceGray".to_vec(), 8),
            Self::L16 => (b"DeviceGray".to_vec(), 16),
            Self::CMYK => (b"DeviceCMYK".to_vec(), 8),
        }
    }

    pub fn from_pdf_format(info: (&[u8], u8)) -> Self {
        match info {
            (b"DeviceRGB", 8) => Self::RGB8,
            (b"DeviceRGB", 16) => Self::RGB16,
            (b"DeviceGray", 8) => Self::L8,
            (b"DeviceGray", 16) => Self::L16,
            (b"DeviceCMYK", 8) => Self::CMYK,
            _ => unreachable!(),
        }
    }

    pub fn into_extended(&self) -> image::ExtendedColorType {
        match self {
            Self::L8 => image::ExtendedColorType::L8,
            Self::L16 => image::ExtendedColorType::L16,
            Self::CMYK => image::ExtendedColorType::Cmyk8,
            Self::RGB8 => image::ExtendedColorType::Rgb8,
            Self::RGB16 => image::ExtendedColorType::Rgb16,
        }
    }

    pub fn into_bits(&self) -> u8 {
        match self {
            Self::L8 => 8,
            Self::L16 => 16,
            Self::CMYK => 8,
            Self::RGB8 => 8,
            Self::RGB16 => 16,
        }
    }
}

pub fn decompress_and_save_png(
    content: &[u8],
    page_num: u32,
    width: u32,
    height: u32,
    color_space: PDFConColorSpace,
    out_dir: &PathBuf,
    optimize: bool,
) -> Result<(), PDFConError> {
    let mut output = Vec::new();
    let out_writer = BufWriter::new(&mut output);
    let mut decompress =
        flate2::write::ZlibDecoder::new_with_decompress(out_writer, flate2::Decompress::new(true));
    decompress.write_all(&content)?;
    decompress.flush()?;

    let _ = decompress.finish()?;
    let mut encoded = Vec::new();
    let encoder_writer = BufWriter::new(&mut encoded);

    let encoder = png::PngEncoder::new_with_quality(
        encoder_writer,
        png::CompressionType::Best,
        png::FilterType::Adaptive,
    );

    match encoder.write_image(&output, width, height, color_space.into_extended()) {
        Ok(_) => {}
        Err(_e) => {
            error!("Failed to encode image");
            return Err(PDFConError::ImageErrorMisc);
        }
    }

    let options = oxipng::Options::default();
    let file_name = format!("{:0>5}.png", page_num);
    let file_path = out_dir.join(file_name);
    let file = std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .open(file_path)?;

    if optimize {
        match oxipng::optimize_from_memory(&encoded, &options) {
            Ok(b) => {
                let mut file_writer = BufWriter::new(file);
                file_writer.write_all(&b)?;
                file_writer.flush()?;
                return Ok(());
            }
            Err(_e) => {
                error!("Failed to optimize PNG size");
                return Err(PDFConError::OxiPngOptimizeError);
            }
        }
    } else {
        let mut file_writer = BufWriter::new(file);
        file_writer.write_all(&encoded)?;
        file_writer.flush()?;
        return Ok(());
    }
}

// Unless
impl From<mozjpeg::ColorSpace> for PDFConColorSpace {
    fn from(c: mozjpeg::ColorSpace) -> Self {
        match c {
            mozjpeg::ColorSpace::JCS_CMYK => Self::CMYK,
            mozjpeg::ColorSpace::JCS_RGB => Self::RGB8,
            mozjpeg::ColorSpace::JCS_GRAYSCALE => Self::L8,
            _ => Self::RGB8,
        }
    }
}

impl From<image::ColorType> for PDFConColorSpace {
    fn from(c: image::ColorType) -> Self {
        match c {
            image::ColorType::L8 | image::ColorType::La8 => Self::L8,
            image::ColorType::Rgb8 | image::ColorType::Rgba8 => Self::RGB8,
            _ => Self::RGB8,
        }
    }
}

pub fn compress_zlib(
    v: Vec<u8>,
    compression_method: flate2::Compression,
) -> Result<Vec<u8>, PDFConError> {
    let writer: BufWriter<Vec<u8>> = BufWriter::new(Vec::new());
    let mut compressor = ZlibEncoder::new(writer, compression_method);
    compressor.write_all(&v)?;

    let compressed_bytes = compressor.finish()?;

    let inner = compressed_bytes
        .into_inner()
        .map_err(|_| PDFConError::BufferInnerError)?;
    Ok(inner)
}

pub mod optimize {
    use super::{PDFConColorSpace, compress_zlib};
    use crate::error::PDFConError;
    use flate2::Compression;
    use image::{self, ColorType};
    use log::error;
    use mozjpeg;
    use std::io::{BufReader, BufWriter, Read, Seek, SeekFrom};

    pub enum ImageData {
        PNG(Vec<u8>, u32, u32, PDFConColorSpace),
        JPEG(Vec<u8>, u32, u32, PDFConColorSpace),
    }

    pub fn process_png_optimized(file: std::fs::File) -> Result<ImageData, PDFConError> {
        let reader = BufReader::new(file);

        let png_reader = image::ImageReader::with_format(reader, image::ImageFormat::Png);
        let decoder = png_reader.decode()?;
        let colorspace = decoder.color();

        match colorspace {
            ColorType::L8 | ColorType::La8 => {
                let temp = decoder.to_luma8();
                let width = temp.width();
                let height = temp.height();
                // Compress the buffer
                let compressed = compress_zlib(temp.to_vec(), Compression::best())?;
                Ok(ImageData::PNG(
                    compressed,
                    width,
                    height,
                    PDFConColorSpace::L8,
                ))
            }
            ColorType::L16 | ColorType::La16 => {
                let temp = decoder.to_luma16();
                let width = temp.width();
                let height = temp.height();
                // Compress the buffer
                let compressed = compress_zlib(
                    temp.to_vec()
                        .iter()
                        .flat_map(|&x| x.to_be_bytes())
                        .collect(),
                    Compression::best(),
                )?;
                Ok(ImageData::PNG(
                    compressed,
                    width,
                    height,
                    PDFConColorSpace::L16,
                ))
            }
            ColorType::Rgb8 | ColorType::Rgba8 => {
                let temp = decoder.to_rgb8();
                let width = temp.width();
                let height = temp.height();
                // Compress the buffer
                let compressed = compress_zlib(temp.to_vec(), Compression::best())?;
                Ok(ImageData::PNG(
                    compressed,
                    width,
                    height,
                    PDFConColorSpace::RGB8,
                ))
            }
            ColorType::Rgb16 | ColorType::Rgba16 => {
                let temp = decoder.to_rgb16();
                let width = temp.width();
                let height = temp.height();
                // Compress the buffer
                let compressed = compress_zlib(
                    temp.to_vec()
                        .iter()
                        .flat_map(|&x| x.to_be_bytes())
                        .collect(),
                    Compression::best(),
                )?;
                Ok(ImageData::PNG(
                    compressed,
                    width,
                    height,
                    PDFConColorSpace::RGB16,
                ))
            }
            ColorType::Rgba32F | ColorType::Rgb32F => {
                // Reduce bitdepth to be compatible with PDF and remove alpha
                let temp = decoder.to_rgb16();
                let width = temp.width();
                let height = temp.height();
                // Compress the buffer
                let compressed = compress_zlib(
                    temp.to_vec()
                        .iter()
                        .flat_map(|&x| x.to_be_bytes())
                        .collect(),
                    Compression::best(),
                )?;
                Ok(ImageData::PNG(
                    compressed,
                    width,
                    height,
                    PDFConColorSpace::RGB16,
                ))
            }
            _ => unreachable!(),
        }
    }

    pub fn optimize_jpeg_mem(content: &[u8]) -> Result<Vec<u8>, PDFConError> {
        let reader = BufReader::new(content);
        let mut decompress = match mozjpeg::decompress::Decompress::builder()
            .with_markers(mozjpeg::ALL_MARKERS)
            .from_reader(reader)
        {
            Ok(d) => d,
            Err(e) => {
                error!("Decompress err: {}", e.to_string());
                return Err(PDFConError::MozDecompressBufferError);
            }
        };

        let height = decompress.height();
        let width = decompress.width();

        let pixel_density = decompress
            .pixel_density()
            .unwrap_or(mozjpeg::PixelDensity::default());

        decompress.dct_method(mozjpeg::DctMethod::IntegerSlow);
        decompress.do_block_smoothing(true);

        let (pixels, output_color_space) = match decompress.color_space() {
            mozjpeg::ColorSpace::JCS_GRAYSCALE => {
                let mut gray_buff = decompress.grayscale()?;
                let pixels = gray_buff.read_scanlines()?;
                gray_buff.finish()?;
                (pixels, mozjpeg::ColorSpace::JCS_GRAYSCALE)
            }
            mozjpeg::ColorSpace::JCS_RGB | mozjpeg::ColorSpace::JCS_YCbCr => {
                let mut rgb_buff = decompress.rgb()?;
                let pixels = rgb_buff.read_scanlines()?;
                rgb_buff.finish()?;
                (pixels, mozjpeg::ColorSpace::JCS_RGB)
            }
            mozjpeg::ColorSpace::JCS_CMYK => {
                let mut cmyk_buffer = decompress.to_colorspace(mozjpeg::ColorSpace::JCS_CMYK)?;
                let pixels = cmyk_buffer.read_scanlines()?;
                cmyk_buffer.finish()?;
                (pixels, mozjpeg::ColorSpace::JCS_CMYK)
            }
            _ => return Err(PDFConError::MozDecompressBufferError),
        };

        let writer: BufWriter<Vec<u8>> = BufWriter::new(Vec::new());
        let mut compress = mozjpeg::compress::Compress::new(output_color_space);
        compress.set_pixel_density(pixel_density);
        compress.set_size(width, height);
        compress.set_optimize_scans(true);
        compress.set_optimize_coding(true);
        compress.set_progressive_mode();
        compress.set_quality(92.0);

        let mut compress_start = compress.start_compress(writer)?;

        compress_start.write_scanlines(&pixels[..])?;

        let finished_writer = compress_start.finish()?;
        let content = finished_writer
            .into_inner()
            .map_err(|_| PDFConError::BufferInnerError)?;
        Ok(content)
    }

    pub fn optimize_jpeg(file: std::fs::File) -> Result<ImageData, PDFConError> {
        let result = std::panic::catch_unwind(|| -> Result<ImageData, PDFConError> {
            let reader = BufReader::new(&file);
            let mut decompress = match mozjpeg::decompress::Decompress::builder()
                .with_markers(mozjpeg::ALL_MARKERS)
                .from_reader(reader)
            {
                Ok(d) => d,
                Err(e) => {
                    error!("Decompress err: {}", e.to_string());
                    return Err(PDFConError::MozDecompressBufferError);
                }
            };

            let height = decompress.height();
            let width = decompress.width();

            let pixel_density = decompress
                .pixel_density()
                .unwrap_or(mozjpeg::PixelDensity::default());

            decompress.dct_method(mozjpeg::DctMethod::IntegerSlow);
            decompress.do_block_smoothing(true);

            let (pixels, output_color_space) = match decompress.color_space() {
                mozjpeg::ColorSpace::JCS_GRAYSCALE => {
                    let mut gray_buff = decompress.grayscale()?;
                    let pixels = gray_buff.read_scanlines()?;
                    gray_buff.finish()?;
                    (pixels, mozjpeg::ColorSpace::JCS_GRAYSCALE)
                }
                mozjpeg::ColorSpace::JCS_RGB | mozjpeg::ColorSpace::JCS_YCbCr => {
                    let mut rgb_buff = decompress.rgb()?;
                    let pixels = rgb_buff.read_scanlines()?;
                    rgb_buff.finish()?;
                    (pixels, mozjpeg::ColorSpace::JCS_RGB)
                }
                mozjpeg::ColorSpace::JCS_CMYK => {
                    let mut cmyk_buffer =
                        decompress.to_colorspace(mozjpeg::ColorSpace::JCS_CMYK)?;
                    let pixels = cmyk_buffer.read_scanlines()?;
                    cmyk_buffer.finish()?;
                    (pixels, mozjpeg::ColorSpace::JCS_CMYK)
                }
                _ => return Err(PDFConError::MozDecompressBufferError),
            };

            let writer: BufWriter<Vec<u8>> = BufWriter::new(Vec::new());
            let mut compress = mozjpeg::compress::Compress::new(output_color_space);
            compress.set_pixel_density(pixel_density);
            compress.set_size(width, height);
            compress.set_optimize_scans(true);
            compress.set_optimize_coding(true);
            compress.set_progressive_mode();
            compress.set_quality(92.0);

            let mut compress_start = compress.start_compress(writer)?;

            compress_start.write_scanlines(&pixels[..])?;

            let finished_writer = compress_start.finish()?;
            let content = finished_writer
                .into_inner()
                .map_err(|_| PDFConError::BufferInnerError)?;

            Ok(ImageData::JPEG(
                content,
                width as u32,
                height as u32,
                PDFConColorSpace::from(output_color_space),
            ))
        });

        match result {
            Ok(r) => r,
            Err(e) => {
                error!("MozJpeg failed: {:?}", e);
                Err(PDFConError::MozUnwindError)
            }
        }
    }

    pub fn jpeg(mut file: std::fs::File) -> Result<ImageData, PDFConError> {
        let reader = BufReader::new(&file);

        let image = image::ImageReader::with_format(reader, image::ImageFormat::Jpeg);
        let decoder = image.decode()?;
        let width = decoder.width();
        let height = decoder.height();
        let color = decoder.color();

        drop(decoder);

        file.seek(SeekFrom::Start(0))?;

        let mut reader = BufReader::new(&file);
        let mut contents = Vec::new();
        reader.read_to_end(&mut contents)?;

        Ok(ImageData::JPEG(
            contents,
            width,
            height,
            PDFConColorSpace::from(color),
        ))
    }
}
