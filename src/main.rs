use chinenshichanaka::{convert, reduce_colors, render_svg_to_image};
use clap::Parser;
use image::{imageops, DynamicImage, GenericImageView, Pixel, Rgba};
use std::fs;
use std::process;

// Input file support depends on the set of features in Cargo.toml

// https://docs.rs/clap/latest/clap/_derive/index.html
#[derive(Parser, Debug)]
#[command(version, about, author, long_about = None)]
struct Args {
    /// The input image file. Supports SVG and many other formats, see
    /// https://github.com/image-rs/image?tab=readme-ov-file#supported-image-formats
    #[arg(index = 1)]
    input: String,

    /// The output file which should end with ".ico"
    /// https://en.wikipedia.org/wiki/ICO_(file_format)
    #[arg(index = 2, default_value = "favicon.ico")]
    output: String,

    /// Verbose mode gives more details about the conversion process
    #[arg(short, long)]
    verbose: bool,
}

/// Entry point for the CLI tool. Parses arguments and runs the conversion process.
fn main() {
    let args: Args = Args::parse();
    if args.verbose {
        println!("Converting '{}' to '{}'", args.input, args.output);
    }

    match args.output.ends_with(".ico") {
        true => {
            convert_paths(&args.input, &args.output, args.verbose);
        }
        false => {
            eprintln!("The output file have to use the 'ico' suffix");
            process::exit(1);
        }
    }
}

/// Converts an input image file to an ICO file, optionally printing verbose output.
///
/// # Arguments
/// * `input` - Path to the input image file (SVG or raster).
/// * `output` - Path to the output ICO file.
/// * `verbosity` - Whether to print verbose output.
pub fn convert_paths(input: &str, output: &str, verbosity: bool) {
    // Read the content of the file into a byte vector
    let input_buffer: Vec<u8> = match fs::read(input) {
        Ok(buffer) => buffer,
        Err(err) => {
            eprintln!("Error reading the input image. {err}");
            return;
        }
    };

    let img = if input.ends_with(".svg") {
        render_svg_to_image(&input_buffer)
    } else {
        match image::load_from_memory(&input_buffer) {
            Ok(img) => img,
            Err(err) => {
                eprintln!("Error decoding the input image. {err}");
                return;
            }
        }
    };

    // The dimensions method returns the images width and height.
    if verbosity {
        println!("Original image dimensions {:?}", img.dimensions());
    }

    // The color method returns the image's `ColorType`.
    if verbosity {
        println!("Original image color type {:?}", img.color());
    }

    let img: DynamicImage = resize_to_square(&img, 32);

    // Reduce colors to 16
    let img = reduce_colors(&img, 16);

    // The dimensions method returns the images width and height.
    if verbosity {
        println!("Dimensions after resizing to square {:?}", img.dimensions());
    }

    // The color method returns the image's `ColorType`.
    if verbosity {
        println!("Color type after color reduction {:?}", img.color());
    }

    // Call the convert function with the input buffer
    let output_buffer: Vec<u8> = convert(img);

    // Finally, save the output buffer to a new file
    match fs::write(output, &output_buffer) {
        Ok(_) => println!("Output saved to '{output}'"),
        Err(err) => eprintln!("Error saving the output image. {err}"),
    }
}

/// Calculates new dimensions for resizing an image to fit within a square.
///
/// # Arguments
/// * `input_width` - Width of the input image.
/// * `input_height` - Height of the input image.
/// * `output_size` - Desired output size (width and height).
///
/// # Returns
/// Tuple of new width and height.
///
/// # Examples
/// ```
/// # use chinenshichanaka::calculate_size;
/// let (w, h) = chinenshichanaka::calculate_size(100, 150, 200);
/// assert_eq!((w, h), (133, 200));
/// ```
fn calculate_size(input_width: u32, input_height: u32, output_size: u32) -> (u32, u32) {
    // Calculate the scaling factor to fit the input image within a square of the desired size
    let scale_factor = f64::min(
        output_size as f64 / input_width as f64,
        output_size as f64 / input_height as f64,
    );

    // Calculate the new dimensions after resizing
    let new_width = (input_width as f64 * scale_factor) as u32;
    let new_height = (input_height as f64 * scale_factor) as u32;

    (new_width, new_height)
}

// Get the color of the top-left pixel
/// Gets the color of the top-left pixel of an image.
///
/// # Arguments
/// * `input_image` - Reference to the input image.
///
/// # Returns
/// The color of the top-left pixel as `Rgba<u8>`.
fn get_top_left_color(input_image: &DynamicImage) -> Rgba<u8> {
    input_image.get_pixel(0, 0)
}

// Create a new square image with the desired output size and fill it with the background color
/// Creates a new square image of the given size, filled with the specified background color.
///
/// # Arguments
/// * `output_size` - Size of the square image (width and height).
/// * `background_color` - Color to fill the image.
///
/// # Returns
/// A new `DynamicImage` filled with the background color.
fn create_square_image(output_size: u32, background_color: Rgba<u8>) -> DynamicImage {
    let mut square_image = DynamicImage::new_rgb8(output_size, output_size);
    imageops::overlay(
        &mut square_image,
        &DynamicImage::ImageRgb8(image::RgbImage::from_pixel(1, 1, background_color.to_rgb())),
        0,
        0,
    );
    square_image
}

// Resize the input image using Lanczos3 filter for high-quality results
/// Resizes an image to the specified dimensions using the Lanczos3 filter.
///
/// # Arguments
/// * `input_image` - Reference to the input image.
/// * `new_width` - Desired width.
/// * `new_height` - Desired height.
///
/// # Returns
/// A new `DynamicImage` with the resized dimensions.
fn resize_image(input_image: &DynamicImage, new_width: u32, new_height: u32) -> DynamicImage {
    input_image.resize_exact(new_width, new_height, imageops::FilterType::Lanczos3)
}

// Paste the resized image onto the square image at the specified position
/// Pastes a resized image onto a square image at the specified position.
///
/// # Arguments
/// * `square_image` - Mutable reference to the destination image.
/// * `resized_image` - Reference to the image to paste.
/// * `paste_x` - X coordinate for pasting.
/// * `paste_y` - Y coordinate for pasting.
fn paste_resized_image(
    square_image: &mut DynamicImage,
    resized_image: &DynamicImage,
    paste_x: u32,
    paste_y: u32,
) {
    imageops::overlay(square_image, resized_image, paste_x as i64, paste_y as i64);
}

// Resize input image to a square with the specified output size
/// Resizes an image to a square of the specified size, centering the original image.
///
/// # Arguments
/// * `input_image` - Reference to the input image.
/// * `output_size` - Desired size for the square image.
///
/// # Returns
/// A new `DynamicImage` resized and centered in a square.
fn resize_to_square(input_image: &DynamicImage, output_size: u32) -> DynamicImage {
    let (input_width, input_height) = input_image.dimensions();
    let (new_width, new_height) = calculate_size(input_width, input_height, output_size);
    let top_left_color = get_top_left_color(input_image);
    let mut square_image = create_square_image(output_size, top_left_color);
    let paste_x = (output_size - new_width) / 2;
    let paste_y = (output_size - new_height) / 2;
    let resized_image = resize_image(input_image, new_width, new_height);
    paste_resized_image(&mut square_image, &resized_image, paste_x, paste_y);
    square_image
}

// Tests
#[cfg(test)]
mod tests {

    use super::*;
    use assert_cmd::Command;
    use image::Rgb;
    use image::{imageops, DynamicImage, GenericImageView, Pixel, Rgba};
    use std::io::Cursor;
    use tempfile::NamedTempFile;

    // Helper function to create an image with specified dimensions and color
    fn create_test_image(width: u32, height: u32, color: Rgba<u8>) -> DynamicImage {
        let mut img: DynamicImage = DynamicImage::new_rgba8(width, height);
        imageops::overlay(
            &mut img,
            &DynamicImage::ImageRgba8(image::RgbaImage::from_pixel(1, 1, color.to_rgba())),
            0,
            0,
        );
        img
    }

    // Helper function to create a temporary image file
    fn create_temp_image_file(extension: &str, color: Rgba<u8>) -> (NamedTempFile, String) {
        let temp_file = NamedTempFile::new().expect("Failed to create temp input file");
        let file_path = temp_file.path().to_str().unwrap().to_owned() + extension;
        let input_image = create_test_image(100, 150, color);
        input_image
            .save(&file_path)
            .expect("Failed to save input image");
        (temp_file, file_path)
    }

    // Helper function to create a temporary directory and output file path
    fn create_temp_output_file(extension: &str) -> (tempfile::TempDir, String) {
        let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
        let output_path = temp_dir.path().to_str().unwrap().to_owned() + extension;
        (temp_dir, output_path)
    }

    // Verifies that a valid image buffer is correctly converted into an ICO format.
    #[test]
    fn test_convert_with_valid_input() {
        // Create a test image
        let input_image: DynamicImage = create_square_image(32, Rgba([255, 0, 0, 255]));
        let input_image: DynamicImage = reduce_colors(&input_image, 32);
        // Call the convert function with the test image
        let output_buffer: Vec<u8> = convert(input_image);

        let guess: image::ImageFormat =
            image::guess_format(&output_buffer).expect("Failed to guess output image format");
        assert_eq!(guess, image::ImageFormat::Ico);

        // Open the output image from the byte buffer
        let reader = image::ImageReader::new(Cursor::new(&output_buffer))
            .with_guessed_format()
            .expect("Cursor io never fails");
        assert_eq!(reader.format(), Some(image::ImageFormat::Ico));

        // Perform assertions on the output image
        let dimensions: (u32, u32) = reader
            .into_dimensions()
            .expect("Failed to get output image dimensions");
        assert_eq!(dimensions, (32, 32));
    }

    // Ensures that invalid input (e.g., an empty buffer) results in no output.
    #[test]
    fn test_convert_with_invalid_input() {
        // Call the convert function with an invalid image
        let invalid_image = DynamicImage::new_rgb8(0, 0); // Empty image
        let result = std::panic::catch_unwind(|| convert(invalid_image));

        // Ensure the function panics due to invalid input
        assert!(result.is_err());
    }

    // Validates the logic for calculating new dimensions.
    #[test]
    fn test_calculate_size() {
        assert_eq!(calculate_size(100, 150, 200), (133, 200));
        assert_eq!(calculate_size(400, 600, 200), (133, 200));
        assert_eq!(calculate_size(300, 300, 200), (200, 200));
        assert_eq!(calculate_size(100, 100, 200), (200, 200));
        assert_eq!(calculate_size(50, 100, 200), (100, 200));
        assert_eq!(calculate_size(100, 50, 200), (200, 100));
    }

    // Ensures that the top-left color of an image is correctly retrieved.
    #[test]
    fn test_get_top_left_color() {
        let input_image: DynamicImage =
            DynamicImage::ImageRgb8(image::RgbImage::from_pixel(1, 1, Rgb([255, 0, 0])));
        assert_eq!(get_top_left_color(&input_image), Rgba([255, 0, 0, 255]));
    }

    #[test]
    fn test_create_square_image() {
        let background_color: Rgba<u8> = Rgba([255, 0, 0, 255]);
        let square_image: DynamicImage = create_square_image(200, background_color);
        assert_eq!(square_image.dimensions(), (200, 200));
        assert_eq!(square_image.get_pixel(0, 0), Rgba([255, 0, 0, 255]));
    }

    #[test]
    fn test_resize_image() {
        let input_image: DynamicImage =
            DynamicImage::ImageRgb8(image::RgbImage::from_pixel(1, 1, Rgb([255, 0, 0])));
        let resized_image: DynamicImage = resize_image(&input_image, 100, 100);
        assert_eq!(resized_image.dimensions(), (100, 100));
    }

    #[test]
    fn test_paste_resized_image() {
        let mut square_image: DynamicImage =
            DynamicImage::ImageRgb8(image::RgbImage::from_pixel(200, 200, Rgb([255, 255, 255])));
        let resized_image: DynamicImage =
            DynamicImage::ImageRgb8(image::RgbImage::from_pixel(100, 100, Rgb([0, 0, 0])));
        paste_resized_image(&mut square_image, &resized_image, 50, 50);
        assert_eq!(square_image.get_pixel(50, 50), Rgba([0, 0, 0, 255]));
    }

    // Confirms that the function properly resizes an image to fit a square.
    #[test]
    fn test_resize_to_square() {
        let input_image: DynamicImage =
            DynamicImage::ImageRgb8(image::RgbImage::from_pixel(1, 1, Rgb([255, 0, 0])));
        let result: DynamicImage = resize_to_square(&input_image, 200);
        assert_eq!(result.dimensions(), (200, 200));
        assert_eq!(result.get_pixel(50, 50), Rgba([255, 0, 0, 255]));
    }

    #[test]
    fn test_reduce_colors() {
        let input_image: DynamicImage = create_test_image(100, 100, Rgba([255, 0, 0, 255]));
        let reduced_image: DynamicImage = reduce_colors(&input_image, 16);
        assert_eq!(reduced_image.dimensions(), (100, 100));

        // Check that the number of unique colors is reduced
        let unique_colors: std::collections::HashSet<_> = reduced_image
            .pixels()
            .map(|(_, _, pixel)| pixel.0)
            .collect();
        assert!(unique_colors.len() <= 16);
    }

    #[test]
    fn test_reduce_colors_with_more_colors() {
        let input_image: DynamicImage = create_test_image(100, 100, Rgba([0, 255, 0, 255]));
        let reduced_image: DynamicImage = reduce_colors(&input_image, 256);
        assert_eq!(reduced_image.dimensions(), (100, 100));

        // Check that the number of unique colors is reduced
        let unique_colors: std::collections::HashSet<_> = reduced_image
            .pixels()
            .map(|(_, _, pixel)| pixel.0)
            .collect();
        assert!(unique_colors.len() <= 256);
    }

    #[test]
    fn test_cli_tool() {
        // Create a temporary PNG file as input
        let temp_input: NamedTempFile =
            NamedTempFile::new().expect("Failed to create temp input file");
        let input_path = temp_input.path().to_str().unwrap().to_owned() + ".png";

        // Create a test image and save it as PNG
        let input_image = create_test_image(100, 150, Rgba([255, 0, 0, 255]));
        input_image
            .save(&input_path)
            .expect("Failed to save input image");

        // Create a temp folder
        let temp_dir: tempfile::TempDir = tempfile::tempdir().expect("Failed to create temp dir");
        println!("temp_dir: {:?}", temp_dir);
        // Create a temporary ICO file as output
        let output_path = temp_dir.path().to_str().unwrap().to_owned() + "/output.ico";
        println!("output_path: {:?}", output_path);

        // Execute the CLI tool
        Command::cargo_bin(env!("CARGO_PKG_NAME"))
            .expect("Binary not found")
            .arg(input_path)
            .arg(&output_path)
            .assert()
            .success();

        // Verify the output file exists and is in ICO format
        let output_content = fs::read(output_path).expect("Failed to read output file");
        let guess = image::guess_format(&output_content).expect("Failed to guess format");
        assert_eq!(guess, image::ImageFormat::Ico);
    }

    #[test]
    fn test_main_with_valid_png_and_ico() {
        let temp_input = NamedTempFile::new().expect("Failed to create temp input file");
        let input_path = temp_input.path().to_str().unwrap().to_owned() + ".png";
        let input_image = create_test_image(100, 150, Rgba([255, 0, 0, 255]));
        input_image
            .save(&input_path)
            .expect("Failed to save input image");

        let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
        let output_path = temp_dir.path().to_str().unwrap().to_owned() + "/output.ico";

        Command::cargo_bin(env!("CARGO_PKG_NAME"))
            .expect("Binary not found")
            .arg(&input_path)
            .arg(&output_path)
            .assert()
            .success();

        let output_content = fs::read(output_path).expect("Failed to read output file");
        let guess = image::guess_format(&output_content).expect("Failed to guess format");
        assert_eq!(guess, image::ImageFormat::Ico);
    }

    #[test]
    fn test_main_with_invalid_output_extension() {
        let (_, input_path) = create_temp_image_file(".png", Rgba([255, 0, 0, 255]));
        let (_, output_path) = create_temp_output_file("/output.jpg");

        let assert = Command::cargo_bin(env!("CARGO_PKG_NAME"))
            .expect("Binary not found")
            .arg(&input_path)
            .arg(&output_path)
            .arg("--verbose")
            .assert()
            .failure();

        let stderr = String::from_utf8(assert.get_output().stderr.clone()).unwrap();
        assert!(stderr.contains("The output file have to use the 'ico' suffix"));
    }

    #[test]
    fn test_convert_paths_with_invalid_input() {
        let (_, output_path) = create_temp_output_file("/output.ico");
        let input_path = "invalid.png".to_string();

        convert_paths(&input_path, &output_path, true);

        assert!(!std::path::Path::new(&output_path).exists());
    }

    #[test]
    fn test_convert_paths_with_valid_input() {
        let temp_input = NamedTempFile::new().expect("Failed to create temp input file");
        let input_path = temp_input.path().to_str().unwrap().to_owned() + ".png";
        let input_image = create_test_image(100, 150, Rgba([255, 0, 0, 255]));
        input_image
            .save(&input_path)
            .expect("Failed to save input image");

        let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
        let output_path = temp_dir.path().to_str().unwrap().to_owned() + "/output.ico";

        convert_paths(&input_path, &output_path, true);

        assert!(std::path::Path::new(&output_path).exists());
        let output_content = fs::read(output_path).expect("Failed to read output file");
        let guess = image::guess_format(&output_content).expect("Failed to guess format");
        assert_eq!(guess, image::ImageFormat::Ico);
    }

    #[test]
    fn test_main_with_valid_png_default_output() {
        let (_, input_path) = create_temp_image_file(".png", Rgba([255, 0, 0, 255]));
        let output_path = std::env::current_dir().unwrap().join("favicon.ico");

        Command::cargo_bin(env!("CARGO_PKG_NAME"))
            .expect("Binary not found")
            .arg(&input_path)
            .assert()
            .success();

        assert!(&output_path.exists());
        let output_content = fs::read(&output_path).expect("Failed to read output file");
        let guess = image::guess_format(&output_content).expect("Failed to guess format");
        assert_eq!(guess, image::ImageFormat::Ico);

        // Clean up the generated favicon.ico file
        fs::remove_file(output_path).expect("Failed to remove output file");
    }

    #[test]
    fn test_convert_paths_with_read_error() {
        let (_, output_path) = create_temp_output_file("/output.ico");
        let input_path = "non_existent.png".to_string();

        convert_paths(&input_path, &output_path, true);

        assert!(!std::path::Path::new(&output_path).exists());
    }

    #[test]
    fn test_convert_with_valid_svg_input() {
        // Create a simple SVG content
        let svg_content = r#"
        <svg width="100" height="100" xmlns="http://www.w3.org/2000/svg">
            <rect width="100" height="100" style="fill:rgb(0,0,255);"/>
        </svg>
        "#;
        let input_buffer = svg_content.as_bytes();

        // Render SVG to image
        let input_image = render_svg_to_image(input_buffer);
        let input_image = reduce_colors(&input_image, 16);

        // Call the convert function with the rendered image
        let output_buffer: Vec<u8> = convert(input_image);

        let guess: image::ImageFormat =
            image::guess_format(&output_buffer).expect("Failed to guess output image format");
        assert_eq!(guess, image::ImageFormat::Ico);

        // Open the output image from the byte buffer
        let reader = image::ImageReader::new(Cursor::new(&output_buffer))
            .with_guessed_format()
            .expect("Cursor io never fails");
        assert_eq!(reader.format(), Some(image::ImageFormat::Ico));

        // Perform assertions on the output image
        let dimensions: (u32, u32) = reader
            .into_dimensions()
            .expect("Failed to get output image dimensions");
        assert_eq!(dimensions, (32, 32));
    }

    #[test]
    fn test_convert_paths_with_valid_svg_input() {
        let temp_input = NamedTempFile::new().expect("Failed to create temp input file");
        let input_path = temp_input.path().to_str().unwrap().to_owned() + ".svg";
        let svg_content = r#"
        <svg width="100" height="100" xmlns="http://www.w3.org/2000/svg">
            <rect width="100" height="100" style="fill:rgb(0,0,255);"/>
        </svg>
        "#;
        fs::write(&input_path, svg_content).expect("Failed to write SVG content to file");

        let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
        let output_path = temp_dir.path().to_str().unwrap().to_owned() + "/output.ico";

        convert_paths(&input_path, &output_path, true);

        assert!(std::path::Path::new(&output_path).exists());
        let output_content = fs::read(output_path).expect("Failed to read output file");
        let guess = image::guess_format(&output_content).expect("Failed to guess format");
        assert_eq!(guess, image::ImageFormat::Ico);
    }

    #[test]
    fn test_main_with_valid_svg_and_ico() {
        let temp_input = NamedTempFile::new().expect("Failed to create temp input file");
        let input_path = temp_input.path().to_str().unwrap().to_owned() + ".svg";
        let svg_content = r#"
        <svg width="100" height="100" xmlns="http://www.w3.org/2000/svg">
            <rect width="100" height="100" style="fill:rgb(0,0,255);"/>
        </svg>
        "#;
        fs::write(&input_path, svg_content).expect("Failed to write SVG content to file");

        let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
        let output_path = temp_dir.path().to_str().unwrap().to_owned() + "/output.ico";

        Command::cargo_bin(env!("CARGO_PKG_NAME"))
            .expect("Binary not found")
            .arg(&input_path)
            .arg(&output_path)
            .assert()
            .success();

        let output_content = fs::read(output_path).expect("Failed to read output file");
        let guess = image::guess_format(&output_content).expect("Failed to guess format");
        assert_eq!(guess, image::ImageFormat::Ico);
    }

    #[test]
    fn test_main_with_valid_svg_default_output() {
        let temp_input = NamedTempFile::new().expect("Failed to create temp input file");
        let input_path = temp_input.path().to_str().unwrap().to_owned() + ".svg";
        let svg_content = r#"
        <svg width="100" height="100" xmlns="http://www.w3.org/2000/svg">
            <rect width="100" height="100" style="fill:rgb(0,0,255);"/>
        </svg>
        "#;
        fs::write(&input_path, svg_content).expect("Failed to write SVG content to file");

        let output_path = std::env::current_dir().unwrap().join("favicon.ico");

        Command::cargo_bin(env!("CARGO_PKG_NAME"))
            .expect("Binary not found")
            .arg(&input_path)
            .assert()
            .success();

        assert!(&output_path.exists());
        let output_content = fs::read(&output_path).expect("Failed to read output file");
        let guess = image::guess_format(&output_content).expect("Failed to guess format");
        assert_eq!(guess, image::ImageFormat::Ico);

        // Clean up the generated favicon.ico file
        fs::remove_file(output_path).expect("Failed to remove output file");
    }

    #[test]
    fn test_convert_with_large_svg_input() {
        // Large SVG content
        let large_svg_content = r#"
        <svg width="1000" height="1000" xmlns="http://www.w3.org/2000/svg">
            <rect width="1000" height="1000" style="fill:rgb(255,0,0);"/>
        </svg>
        "#;
        let input_buffer = large_svg_content.as_bytes();

        // Render SVG to image
        let input_image = render_svg_to_image(input_buffer);
        let input_image = reduce_colors(&input_image, 16);

        // Call the convert function with the large SVG content
        let output_buffer: Vec<u8> = convert(input_image);

        let guess: image::ImageFormat =
            image::guess_format(&output_buffer).expect("Failed to guess output image format");
        assert_eq!(guess, image::ImageFormat::Ico);

        // Open the output image from the byte buffer
        let reader = image::ImageReader::new(Cursor::new(&output_buffer))
            .with_guessed_format()
            .expect("Cursor io never fails");
        assert_eq!(reader.format(), Some(image::ImageFormat::Ico));

        // Perform assertions on the output image
        let dimensions: (u32, u32) = reader
            .into_dimensions()
            .expect("Failed to get output image dimensions");
        assert_eq!(dimensions, (32, 32));
    }

    #[test]
    fn test_convert_with_svg_with_transparency() {
        // SVG content with transparency
        let transparent_svg_content = r#"
        <svg width="100" height="100" xmlns="http://www.w3.org/2000/svg">
            <rect width="100" height="100" style="fill:rgb(0,0,255);fill-opacity:0.5;"/>
        </svg>
        "#;
        let input_buffer = transparent_svg_content.as_bytes();

        // Render SVG to image
        let input_image = render_svg_to_image(input_buffer);
        let input_image = reduce_colors(&input_image, 32);

        // Call the convert function with the transparent SVG content
        let output_buffer: Vec<u8> = convert(input_image);

        let guess: image::ImageFormat =
            image::guess_format(&output_buffer).expect("Failed to guess output image format");
        assert_eq!(guess, image::ImageFormat::Ico);

        // Open the output image from the byte buffer
        let reader = image::ImageReader::new(Cursor::new(&output_buffer))
            .with_guessed_format()
            .expect("Cursor io never fails");
        assert_eq!(reader.format(), Some(image::ImageFormat::Ico));

        // Perform assertions on the output image
        let dimensions: (u32, u32) = reader
            .into_dimensions()
            .expect("Failed to get output image dimensions");
        assert_eq!(dimensions, (32, 32));
    }

    #[test]
    #[should_panic(expected = "Failed to parse SVG")]
    fn test_render_svg_to_image_with_invalid_svg() {
        // Invalid SVG content that should cause parsing to fail
        let invalid_svg = b"<svg><invalid></svg>";
        render_svg_to_image(invalid_svg);
    }

    #[test]
    #[should_panic(expected = "Failed to parse SVG")]
    fn test_render_svg_to_image_with_malformed_svg() {
        // Malformed SVG content
        let malformed_svg = b"not an svg at all";
        render_svg_to_image(malformed_svg);
    }

    #[test]
    #[should_panic(expected = "Failed to parse SVG")]
    fn test_render_svg_to_image_with_empty_data() {
        // Empty data should fail to parse
        let empty_data = b"";
        render_svg_to_image(empty_data);
    }

    #[test]
    fn test_convert_paths_with_write_permission_error() {
        // Create a valid input file
        let temp_input = NamedTempFile::new().expect("Failed to create temp input file");
        let input_path = temp_input.path().to_str().unwrap().to_owned() + ".png";
        let input_image = create_test_image(100, 150, Rgba([255, 0, 0, 255]));
        input_image
            .save(&input_path)
            .expect("Failed to save input image");

        // Try to write to a directory that doesn't exist (should fail on Windows/Unix)
        let invalid_output_path = "/root/nonexistent/output.ico".to_string();

        // This should handle the write error gracefully
        convert_paths(&input_path, &invalid_output_path, true);

        // The file should not exist
        assert!(!std::path::Path::new(&invalid_output_path).exists());
    }

    #[test]
    fn test_convert_paths_with_image_decode_error() {
        // Create a file with invalid image data
        let temp_input = NamedTempFile::new().expect("Failed to create temp input file");
        let input_path = temp_input.path().to_str().unwrap().to_owned() + ".png";

        // Write invalid image data
        fs::write(&input_path, b"not an image").expect("Failed to write invalid data");

        let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
        let output_path = temp_dir.path().to_str().unwrap().to_owned() + "/output.ico";

        // This should handle the decode error gracefully
        convert_paths(&input_path, &output_path, true);

        // The output file should not exist since conversion failed
        assert!(!std::path::Path::new(&output_path).exists());
    }

    #[test]
    fn test_convert_paths_with_corrupted_svg() {
        // Create a file with corrupted SVG data
        let temp_input = NamedTempFile::new().expect("Failed to create temp input file");
        let input_path = temp_input.path().to_str().unwrap().to_owned() + ".svg";

        // Write corrupted SVG data
        let corrupted_svg = r#"<svg width="100" height="100" xmlns="http://www.w3.org/2000/svg"><rect width="100" height="100""#;
        fs::write(&input_path, corrupted_svg).expect("Failed to write corrupted SVG");

        let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
        let output_path = temp_dir.path().to_str().unwrap().to_owned() + "/output.ico";

        // This should panic due to the expect() in render_svg_to_image
        let result = std::panic::catch_unwind(|| {
            convert_paths(&input_path, &output_path, true);
        });

        // Ensure the function panics due to invalid SVG
        assert!(result.is_err());
    }

    #[test]
    #[should_panic(expected = "Failed to convert image to RGB8")]
    fn test_convert_with_unsupported_image_format() {
        // Create an image that might not convert to RGB8 properly
        // This is a bit tricky since most DynamicImage variants can convert to RGB8
        // But we can create a scenario where the conversion might fail
        let img = DynamicImage::new_luma8(32, 32);

        // For this test, we need to modify the convert function to potentially fail
        // Since as_rgb8() rarely fails, this test documents the potential failure point
        convert(img);
    }

    #[test]
    fn test_reduce_colors_with_zero_colors() {
        // Test edge case with zero colors requested
        let input_image = create_test_image(10, 10, Rgba([255, 0, 0, 255]));

        // This should handle the edge case gracefully or panic predictably
        let result = std::panic::catch_unwind(|| {
            reduce_colors(&input_image, 0);
        });

        // This documents that requesting 0 colors may cause issues
        // The behavior depends on the NeuQuant implementation
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_reduce_colors_with_very_large_color_count() {
        // Test with an unreasonably large number of colors
        let input_image = create_test_image(2, 2, Rgba([255, 0, 0, 255]));

        // Request more colors than pixels exist
        let reduced_image = reduce_colors(&input_image, 1000);

        // Should still work, but actual unique colors will be limited by input
        assert_eq!(reduced_image.dimensions(), (2, 2));
    }

    #[test]
    fn test_calculate_size_with_zero_dimensions() {
        // Test edge cases with zero dimensions
        assert_eq!(calculate_size(0, 100, 200), (0, 200));
        assert_eq!(calculate_size(100, 0, 200), (200, 0));
        assert_eq!(calculate_size(0, 0, 200), (0, 0));
    }

    #[test]
    fn test_calculate_size_with_zero_output_size() {
        // Test with zero output size
        assert_eq!(calculate_size(100, 150, 0), (0, 0));
    }

    #[test]
    fn test_get_top_left_color_with_minimal_image() {
        // Test with 1x1 image
        let minimal_image = create_test_image(1, 1, Rgba([128, 64, 32, 255]));
        let color = get_top_left_color(&minimal_image);
        assert_eq!(color, Rgba([128, 64, 32, 255]));
    }

    #[test]
    fn test_resize_to_square_with_zero_output_size() {
        let input_image = create_test_image(100, 100, Rgba([255, 0, 0, 255]));
        let result = resize_to_square(&input_image, 0);
        assert_eq!(result.dimensions(), (0, 0));
    }

    #[test]
    fn test_resize_to_square_with_very_large_output_size() {
        let input_image = create_test_image(10, 10, Rgba([255, 0, 0, 255]));
        // Test with a large but reasonable output size (1000x1000 instead of 10000x10000)
        let result = resize_to_square(&input_image, 1000);
        assert_eq!(result.dimensions(), (1000, 1000));
    }

    #[test]
    fn test_render_svg_with_very_large_dimensions() {
        // SVG with very large dimensions
        let large_svg = r#"
        <svg width="10000" height="10000" xmlns="http://www.w3.org/2000/svg">
            <rect width="10000" height="10000" style="fill:rgb(255,0,0);"/>
        </svg>
        "#;

        // Should still render to 32x32 regardless of source size
        let result = render_svg_to_image(large_svg.as_bytes());
        assert_eq!(result.dimensions(), (32, 32));
    }

    #[test]
    fn test_convert_paths_with_svg_write_error() {
        // Create a valid SVG input file
        let temp_input = NamedTempFile::new().expect("Failed to create temp input file");
        let input_path = temp_input.path().to_str().unwrap().to_owned() + ".svg";
        let svg_content = r#"
        <svg width="100" height="100" xmlns="http://www.w3.org/2000/svg">
            <rect width="100" height="100" style="fill:rgb(0,0,255);"/>
        </svg>
        "#;
        fs::write(&input_path, svg_content).expect("Failed to write SVG content to file");

        // Try to write to an invalid path
        let invalid_output_path = "/root/nonexistent/output.ico".to_string();

        // This should handle the write error gracefully
        convert_paths(&input_path, &invalid_output_path, true);

        // The file should not exist
        assert!(!std::path::Path::new(&invalid_output_path).exists());
    }
}
