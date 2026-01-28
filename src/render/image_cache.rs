//! Image cache for managing image loading states and GPU textures
//!
//! Handles async image loading, caching, and texture management.

use super::texture::TextureManager;
use super::gpu::GpuContext;
use image::DynamicImage;
use std::collections::HashMap;
use url::Url;

/// State of an image in the cache
#[derive(Debug, Clone)]
pub enum ImageState {
    /// Image is being loaded
    Loading,
    /// Image loaded successfully
    Loaded {
        texture_id: usize,
        width: u32,
        height: u32,
    },
    /// Image failed to load
    Failed {
        error: String,
    },
}

/// Image dimensions (intrinsic size)
#[derive(Debug, Clone, Copy, Default)]
pub struct ImageSize {
    pub width: u32,
    pub height: u32,
}

impl ImageSize {
    pub fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }
}

/// Cache for loaded images
pub struct ImageCache {
    /// Map from URL to image state
    images: HashMap<String, ImageState>,
    /// Texture manager for GPU textures
    texture_manager: TextureManager,
}

impl ImageCache {
    pub fn new() -> Self {
        Self {
            images: HashMap::new(),
            texture_manager: TextureManager::new(),
        }
    }

    /// Check if an image is in the cache
    pub fn contains(&self, url: &str) -> bool {
        self.images.contains_key(url)
    }

    /// Get the state of an image
    pub fn get(&self, url: &str) -> Option<&ImageState> {
        self.images.get(url)
    }

    /// Get the texture ID for a loaded image
    pub fn get_texture_id(&self, url: &str) -> Option<usize> {
        match self.images.get(url) {
            Some(ImageState::Loaded { texture_id, .. }) => Some(*texture_id),
            _ => None,
        }
    }

    /// Get the intrinsic size of a loaded image
    pub fn get_size(&self, url: &str) -> Option<ImageSize> {
        match self.images.get(url) {
            Some(ImageState::Loaded { width, height, .. }) => Some(ImageSize::new(*width, *height)),
            _ => None,
        }
    }

    /// Mark an image as loading
    pub fn start_loading(&mut self, url: &str) {
        self.images.insert(url.to_string(), ImageState::Loading);
    }

    /// Store a loaded image
    pub fn store_image(&mut self, url: &str, image: &DynamicImage, gpu: &GpuContext) {
        let (width, height) = (image.width(), image.height());
        let texture_id = self.texture_manager.load_image(gpu, image);

        self.images.insert(
            url.to_string(),
            ImageState::Loaded {
                texture_id,
                width,
                height,
            },
        );
    }

    /// Store an image loading error
    pub fn store_error(&mut self, url: &str, error: String) {
        self.images.insert(url.to_string(), ImageState::Failed { error });
    }

    /// Get texture view for rendering
    pub fn get_texture_view(&self, texture_id: usize) -> Option<&wgpu::TextureView> {
        self.texture_manager.get_view(texture_id)
    }

    /// Remove an image from the cache
    pub fn remove(&mut self, url: &str) {
        if let Some(ImageState::Loaded { texture_id, .. }) = self.images.remove(url) {
            self.texture_manager.remove(texture_id);
        }
    }

    /// Clear all cached images
    pub fn clear(&mut self) {
        self.images.clear();
        self.texture_manager.clear();
    }

    /// Get list of all image URLs in the cache
    pub fn urls(&self) -> impl Iterator<Item = &String> {
        self.images.keys()
    }

    /// Check if any images are still loading
    pub fn has_pending(&self) -> bool {
        self.images.values().any(|state| matches!(state, ImageState::Loading))
    }
}

impl Default for ImageCache {
    fn default() -> Self {
        Self::new()
    }
}

/// Load an image from bytes
pub fn decode_image(bytes: &[u8]) -> Result<DynamicImage, String> {
    if bytes.is_empty() {
        return Err("Empty image data".to_string());
    }

    // Log first bytes for debugging (both ASCII and hex)
    let preview_ascii: String = bytes.iter().take(100).map(|&b| {
        if b.is_ascii_graphic() || b == b' ' { b as char } else { '.' }
    }).collect();
    let preview_hex: String = bytes.iter().take(16).map(|b| format!("{:02x}", b)).collect::<Vec<_>>().join(" ");
    log::info!("Decoding {} bytes. First 16 hex: [{}]", bytes.len(), preview_hex);
    log::info!("First 100 chars: {}", preview_ascii);

    // Check if it looks like HTML (server error page)
    if bytes.starts_with(b"<!") || bytes.starts_with(b"<html") || bytes.starts_with(b"<HTML") {
        return Err("Received HTML instead of image (possibly redirect or error page)".to_string());
    }

    // Try standard image formats first
    match image::load_from_memory(bytes) {
        Ok(img) => {
            log::info!("Successfully decoded as standard image: {}x{}", img.width(), img.height());
            return Ok(img);
        },
        Err(e) => log::info!("Standard image decode failed: {}", e),
    }

    // Try SVG
    match decode_svg(bytes) {
        Ok(svg_img) => {
            log::info!("Successfully decoded as SVG: {}x{}", svg_img.width(), svg_img.height());
            return Ok(svg_img);
        },
        Err(e) => log::info!("SVG decode failed: {}", e),
    }

    Err(format!("Failed to decode image: unsupported format"))
}

/// Decode SVG to a raster image
pub fn decode_svg(bytes: &[u8]) -> Result<DynamicImage, String> {
    use resvg::tiny_skia;
    use resvg::usvg;

    // Parse SVG
    let options = usvg::Options::default();
    let tree = usvg::Tree::from_data(bytes, &options)
        .map_err(|e| format!("Failed to parse SVG: {}", e))?;

    // Get the SVG size
    let size = tree.size();
    let width = size.width().ceil() as u32;
    let height = size.height().ceil() as u32;

    // Ensure minimum size
    let width = width.max(1);
    let height = height.max(1);

    // Create a pixel buffer
    let mut pixmap = tiny_skia::Pixmap::new(width, height)
        .ok_or_else(|| "Failed to create pixmap".to_string())?;

    // Render the SVG
    resvg::render(&tree, tiny_skia::Transform::default(), &mut pixmap.as_mut());

    // Convert to DynamicImage
    let rgba_data = pixmap.data().to_vec();
    let img = image::RgbaImage::from_raw(width, height, rgba_data)
        .ok_or_else(|| "Failed to create image from SVG data".to_string())?;

    Ok(DynamicImage::ImageRgba8(img))
}

/// Resolve a potentially relative image URL against a base URL
pub fn resolve_image_url(src: &str, base: Option<&Url>) -> Option<String> {
    if src.starts_with("http://") || src.starts_with("https://") {
        Some(src.to_string())
    } else if src.starts_with("data:") {
        // Data URLs are handled inline
        Some(src.to_string())
    } else if let Some(base_url) = base {
        base_url.join(src).ok().map(|u| u.to_string())
    } else {
        None
    }
}

/// Decode a data URL image
pub fn decode_data_url(data_url: &str) -> Result<DynamicImage, String> {
    // Format: data:[<mediatype>][;base64],<data>
    let data_url = data_url.strip_prefix("data:").ok_or("Invalid data URL")?;

    let (header, data) = data_url.split_once(',').ok_or("Invalid data URL format")?;

    let is_base64 = header.contains(";base64");

    let bytes = if is_base64 {
        use base64::{Engine as _, engine::general_purpose::STANDARD};
        STANDARD.decode(data).map_err(|e| format!("Base64 decode error: {}", e))?
    } else {
        // URL-encoded data
        urlencoding::decode_binary(data.as_bytes()).into_owned()
    };

    decode_image(&bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_absolute_url() {
        let result = resolve_image_url("https://example.com/image.png", None);
        assert_eq!(result, Some("https://example.com/image.png".to_string()));
    }

    #[test]
    fn test_resolve_relative_url() {
        let base = Url::parse("https://example.com/page/").unwrap();
        let result = resolve_image_url("../images/photo.jpg", Some(&base));
        assert_eq!(result, Some("https://example.com/images/photo.jpg".to_string()));
    }

    #[test]
    fn test_resolve_data_url() {
        let result = resolve_image_url("data:image/png;base64,abc123", None);
        assert_eq!(result, Some("data:image/png;base64,abc123".to_string()));
    }

    #[test]
    fn test_image_cache_contains() {
        let mut cache = ImageCache::new();
        assert!(!cache.contains("https://example.com/image.png"));
        cache.start_loading("https://example.com/image.png");
        assert!(cache.contains("https://example.com/image.png"));
    }
}
