//! Decoding of the images the user wants to turn into string art.

use std::io::Cursor;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use gpui::RenderImage;
use image::{
    DynamicImage, GenericImageView as _, ImageDecoder as _, ImageFormat, ImageReader, RgbaImage,
};
use rust_i18n::t;

/// Largest side accepted after decoding, before any downscaling.
const MAX_DIMENSION: u32 = 8192;
/// Largest number of pixels accepted after decoding.
const MAX_PIXELS: u64 = 40_000_000;
/// The source is downscaled to this size before it is kept for the
/// computation. It is far above any resolution the computation will use.
const MAX_SOURCE_DIMENSION: u32 = 4096;
/// The texture handed to the GPU is downscaled to this size.
const MAX_PREVIEW_DIMENSION: u32 = 2048;

const SUPPORTED_FORMATS: &[ImageFormat] = &[
    ImageFormat::Png,
    ImageFormat::Jpeg,
    ImageFormat::WebP,
    ImageFormat::Gif,
    ImageFormat::Bmp,
    ImageFormat::Tiff,
    ImageFormat::Ico,
    ImageFormat::Pnm,
    ImageFormat::Qoi,
    ImageFormat::Tga,
];

/// File extensions accepted by the platform file dialogs.
pub const SUPPORTED_EXTENSIONS: &[&str] = &[
    "png", "jpg", "jpeg", "webp", "gif", "bmp", "tif", "tiff", "ico", "pnm", "pbm", "pgm", "ppm",
    "pam", "qoi", "tga",
];

/// An image decoded and ready to be shown and, later, turned into string art.
pub struct ImportedImage {
    pub name: String,
    /// The decoded source, at most [`MAX_SOURCE_DIMENSION`] on its largest side.
    #[allow(dead_code)]
    pub rgba: RgbaImage,
    /// A downscaled BGRA texture for the preview.
    pub preview: Arc<RenderImage>,
    pub width: u32,
    pub height: u32,
}

/// Whether a path has an extension that looks like an image.
///
/// The content is still sniffed before decoding, so a wrong extension is not
/// enough to reject a file: this is only used to filter the dialogs and to
/// pick a file out of a drop. `None` means "no usable extension": the caller
/// should let the decoder decide by content.
pub fn extension_support(path: &Path) -> Option<bool> {
    let extension = path.extension()?.to_str()?;
    Some(
        SUPPORTED_EXTENSIONS
            .iter()
            .any(|supported| extension.eq_ignore_ascii_case(supported)),
    )
}

/// Picks the file of a drop to import: the first one with a supported image
/// extension, or, failing that, the first one without a usable extension since
/// the decoder can still identify it by content.
pub fn select_drop_path(paths: &[PathBuf]) -> Option<&PathBuf> {
    paths
        .iter()
        .find(|path| matches!(extension_support(path), Some(true)))
        .or_else(|| paths.iter().find(|path| extension_support(path).is_none()))
}

/// Error shown when the user picks or drops a file with a known but
/// unsupported extension, such as a video.
pub fn unsupported_file_message(name: &str) -> String {
    t!("import.error.unsupported_file", name = name).to_string()
}

/// Decodes an image from its bytes.
pub fn decode(name: impl Into<String>, bytes: &[u8]) -> Result<ImportedImage, ImportError> {
    let name = name.into();
    let format = detect_format(&name, bytes)?;
    if !SUPPORTED_FORMATS.contains(&format) {
        return Err(ImportError::UnsupportedFormat);
    }

    let mut decoder = ImageReader::with_format(Cursor::new(bytes), format)
        .into_decoder()
        .map_err(|error| ImportError::Decode(error.to_string()))?;
    let orientation = decoder
        .orientation()
        .unwrap_or(image::metadata::Orientation::NoTransforms);
    let mut image = DynamicImage::from_decoder(decoder)
        .map_err(|error| ImportError::Decode(error.to_string()))?;
    image.apply_orientation(orientation);

    let (width, height) = image.dimensions();
    if width == 0 || height == 0 {
        return Err(ImportError::Decode("the image is empty".to_string()));
    }
    if width > MAX_DIMENSION
        || height > MAX_DIMENSION
        || u64::from(width) * u64::from(height) > MAX_PIXELS
    {
        return Err(ImportError::TooLarge);
    }

    let (source_width, source_height) = fit(width, height, MAX_SOURCE_DIMENSION);
    let rgba = if (source_width, source_height) == (width, height) {
        image.to_rgba8()
    } else {
        image
            .resize_exact(
                source_width,
                source_height,
                image::imageops::FilterType::Triangle,
            )
            .to_rgba8()
    };

    let (preview_width, preview_height) = fit(rgba.width(), rgba.height(), MAX_PREVIEW_DIMENSION);
    let mut preview = if (preview_width, preview_height) == (rgba.width(), rgba.height()) {
        rgba.clone()
    } else {
        image::imageops::resize(
            &rgba,
            preview_width,
            preview_height,
            image::imageops::FilterType::Triangle,
        )
    };
    // GPUI renders images as BGRA.
    for pixel in preview.chunks_exact_mut(4) {
        pixel.swap(0, 2);
    }

    Ok(ImportedImage {
        name,
        width: rgba.width(),
        height: rgba.height(),
        rgba,
        preview: Arc::new(RenderImage::new(vec![image::Frame::new(preview)])),
    })
}

fn detect_format(name: &str, bytes: &[u8]) -> Result<ImageFormat, ImportError> {
    if let Ok(format) = image::guess_format(bytes) {
        return Ok(format);
    }
    // Formats without magic bytes (notably TGA) can still be identified by
    // their extension.
    ImageFormat::from_path(name).map_err(|_| ImportError::UnsupportedFormat)
}

fn fit(width: u32, height: u32, max: u32) -> (u32, u32) {
    let largest = width.max(height);
    if largest <= max {
        return (width, height);
    }
    let scale = f64::from(max) / f64::from(largest);
    (
        ((f64::from(width) * scale).round() as u32).max(1),
        ((f64::from(height) * scale).round() as u32).max(1),
    )
}

#[derive(Debug, thiserror::Error)]
pub enum ImportError {
    #[error("The file could not be read: {0}")]
    #[cfg(not(target_family = "wasm"))]
    Read(String),
    #[error("The file is not a supported image")]
    UnsupportedFormat,
    #[error("The image could not be decoded: {0}")]
    Decode(String),
    #[error("The image is too large (max. 8192 px per side and 40 megapixels)")]
    TooLarge,
}

impl ImportError {
    /// The message shown to the user. `Display` stays in English because it is
    /// also used by logs and tests; only the UI is localized.
    pub fn localized(&self) -> String {
        match self {
            #[cfg(not(target_family = "wasm"))]
            Self::Read(error) => t!("import.error.read", error = error).to_string(),
            Self::UnsupportedFormat => t!("import.error.unsupported").to_string(),
            Self::Decode(error) => t!("import.error.decode", error = error).to_string(),
            Self::TooLarge => t!("import.error.too_large").to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn png(width: u32, height: u32) -> Vec<u8> {
        let image =
            image::RgbImage::from_fn(width, height, |x, y| image::Rgb([x as u8, y as u8, 0]));
        let mut bytes = Cursor::new(Vec::new());
        image
            .write_to(&mut bytes, ImageFormat::Png)
            .expect("failed to encode the test image");
        bytes.into_inner()
    }

    #[test]
    fn decodes_a_png() {
        let image = decode("test.png", &png(64, 32)).expect("the image should decode");
        assert_eq!((image.width, image.height), (64, 32));
        assert_eq!((image.rgba.width(), image.rgba.height()), (64, 32));
        assert_eq!(image.preview.size(0).width.0, 64);
    }

    #[test]
    fn downsizes_large_sources_and_previews() {
        let image = decode("test.png", &png(5000, 1000)).expect("the image should decode");
        assert_eq!((image.width, image.height), (4096, 819));
        let size = image.preview.size(0);
        assert_eq!(size.width.0, 2048);
        assert_eq!(size.height.0, 410);
    }

    #[test]
    fn rejects_corrupt_data() {
        assert!(matches!(
            decode("test", b"definitely not an image"),
            Err(ImportError::UnsupportedFormat)
        ));
        assert!(matches!(
            decode("test.png", b"definitely not an image"),
            Err(ImportError::Decode(_))
        ));
    }

    #[test]
    fn recognizes_supported_extensions() {
        assert_eq!(extension_support(Path::new("photo.JPEG")), Some(true));
        assert_eq!(extension_support(Path::new("drawing.tga")), Some(true));
        assert_eq!(extension_support(Path::new("notes.txt")), Some(false));
        assert_eq!(extension_support(Path::new("no-extension")), None);
    }

    #[test]
    fn selects_the_file_of_a_drop_to_import() {
        let png = PathBuf::from("photo.png");
        let txt = PathBuf::from("notes.txt");
        let unknown = PathBuf::from("mystery");

        assert_eq!(
            select_drop_path(&[txt.clone(), png.clone()]),
            Some(&png),
            "the supported image wins over unsupported files"
        );
        assert_eq!(
            select_drop_path(&[txt.clone(), unknown.clone()]),
            Some(&unknown),
            "a file without an extension is attempted by content"
        );
        assert_eq!(select_drop_path(&[txt]), None);
        assert_eq!(select_drop_path(&[]), None);
    }
}
