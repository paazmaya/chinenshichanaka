use color_quant::NeuQuant;
use image::codecs::ico::IcoEncoder;
use image::{DynamicImage, GenericImageView, ImageEncoder};
use resvg::tiny_skia::Pixmap;
use resvg::usvg::{Options, Tree};

/// Converts a `DynamicImage` to ICO format and returns the encoded bytes.
///
/// # Panics
/// Panics if the image cannot be converted to RGB8 or if encoding fails.
///
/// # Examples
/// ```
/// use image::DynamicImage;
/// let img = DynamicImage::new_rgb8(32, 32);
/// let ico_bytes = chinenshichanaka::convert(img);
/// assert!(!ico_bytes.is_empty());
/// ```
pub fn convert(img: DynamicImage) -> Vec<u8> {
    let mut output: Vec<u8> = Vec::new();
    let rgb8 = img.as_rgb8().expect("Failed to convert image to RGB8");
    let raw = rgb8.as_raw();
    IcoEncoder::new(&mut output)
        .write_image(
            raw,
            img.width(),
            img.height(),
            image::ExtendedColorType::Rgb8,
        )
        .expect("Failed to encode output image");
    output
}

/// Reduces the number of colors in a `DynamicImage` using the NeuQuant algorithm.
///
/// # Arguments
/// * `img` - Reference to the input image.
/// * `colors` - Number of colors to reduce to.
///
/// # Returns
/// A new `DynamicImage` with reduced colors.
///
/// # Examples
/// ```
/// use image::{DynamicImage, GenericImageView, Rgba};
/// let img = DynamicImage::new_rgba8(10, 10);
/// let reduced = chinenshichanaka::reduce_colors(&img, 4);
/// assert_eq!(reduced.dimensions(), (10, 10));
/// ```
pub fn reduce_colors(img: &DynamicImage, colors: usize) -> DynamicImage {
    let (width, height) = img.dimensions();
    let pixels = img.to_rgba8().into_raw();
    let quantizer = NeuQuant::new(1, colors, &pixels);
    let mut indices = vec![0; pixels.len() / 4];
    let palette = quantizer.color_map_rgb();
    for (i, chunk) in pixels.chunks(4).enumerate() {
        indices[i] = quantizer.index_of(chunk);
    }
    let mut quantized_pixels = Vec::with_capacity(pixels.len());
    for &index in &indices {
        quantized_pixels.extend_from_slice(&palette[index * 3..index * 3 + 3]);
    }
    DynamicImage::ImageRgb8(image::RgbImage::from_raw(width, height, quantized_pixels).unwrap())
}

/// Renders SVG data to a 32x32 `DynamicImage` using resvg.
///
/// # Arguments
/// * `input` - SVG data as a byte slice.
///
/// # Returns
/// A `DynamicImage` containing the rendered SVG.
///
/// # Panics
/// Panics if SVG parsing or image creation fails.
///
/// # Examples
/// ```
/// use image::GenericImageView;
/// let svg = br#"<svg width='32' height='32' xmlns='http://www.w3.org/2000/svg'><rect width='32' height='32' style='fill:rgb(255,0,0);'/></svg>"#;
/// let img = chinenshichanaka::render_svg_to_image(svg);
/// assert_eq!(img.dimensions(), (32, 32));
/// ```
pub fn render_svg_to_image(input: &[u8]) -> DynamicImage {
    let opt = Options::default();
    let rtree = Tree::from_data(input, &opt).expect("Failed to parse SVG");
    let mut pixmap = Pixmap::new(32, 32).expect("Failed to create pixmap");
    resvg::render(
        &rtree,
        resvg::tiny_skia::Transform::default(),
        &mut pixmap.as_mut(),
    );
    DynamicImage::ImageRgba8(
        image::RgbaImage::from_raw(32, 32, pixmap.data().to_vec())
            .expect("Failed to create image from pixmap"),
    )
}
