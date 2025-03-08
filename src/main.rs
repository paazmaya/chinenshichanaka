use clap::Parser;
use color_quant::NeuQuant;
use image::{imageops, DynamicImage, GenericImageView, ImageEncoder, Pixel, Rgba};
use resvg::tiny_skia::Pixmap;
use resvg::usvg::{Options, Tree};
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
}

fn main() {
    let args: Args = Args::parse();

    println!("input file {}", args.input);
    println!("output file {}", args.output);

    match args.output.ends_with(".ico") {
        true => {
            println!("output file is ico");
            convert_paths(&args.input, &args.output);
        }
        false => {
            eprintln!("output file is not ico");
            process::exit(1);
        }
    }
}

pub fn convert_paths(input: &str, output: &str) {
    // Read the content of the file into a byte vector
    let input_buffer: Vec<u8> = match fs::read(input) {
        Ok(buffer) => buffer,
        Err(err) => {
            eprintln!("Error reading image: {}", err);
            return;
        }
    };

    // Call the convert function with the input buffer
    let output_buffer: Vec<u8> = convert(&input_buffer, input.ends_with(".svg"));
    // Finally, save the output buffer to a new file
    match fs::write(output, &output_buffer) {
        Ok(_) => println!("Output saved to: {}", output),
        Err(err) => eprintln!("Error saving output image: {}", err),
    }
}

fn reduce_colors(img: &DynamicImage, colors: usize) -> DynamicImage {
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

pub fn convert(input: &[u8], is_svg: bool) -> Vec<u8> {
    let img = if is_svg {
        // Render SVG to a raster image
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
    } else {
        // Open the input image from a byte slice
        // Decode the input buffer into a DynamicImage
        match image::load_from_memory(input) {
            Ok(img) => img,
            Err(err) => {
                eprintln!("Error decoding input image: {}", err);
                return Vec::new();
            }
        }
    };

    // The dimensions method returns the images width and height.
    println!("Dimensions {:?}", img.dimensions());

    // The color method returns the image's `ColorType`.
    println!("ColorType {:?}", img.color());

    let img: DynamicImage = resize_to_square(&img, 32);

    // Reduce colors to 16
    let img = reduce_colors(&img, 16);

    // The dimensions method returns the images width and height.
    println!("Dimensions {:?}", img.dimensions());

    // The color method returns the image's `ColorType`.
    println!("ColorType {:?}", img.color());

    // Save the output image to a byte vector
    let mut output: Vec<u8> = Vec::new();
    let rgb8 = img.as_rgb8().expect("Failed to convert image to RGB8");
    let raw = rgb8.as_raw();

    image::codecs::ico::IcoEncoder::new(&mut output)
        .write_image(
            raw,
            img.width(),
            img.height(),
            image::ExtendedColorType::Rgb8,
        )
        .expect("Failed to encode output image");

    output
}

fn calculate_new_size(input_width: u32, input_height: u32, output_size: u32) -> (u32, u32) {
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
fn get_top_left_color(input_image: &DynamicImage) -> Rgba<u8> {
    input_image.get_pixel(0, 0)
}

// Create a new square image with the desired output size and fill it with the background color
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
fn resize_image(input_image: &DynamicImage, new_width: u32, new_height: u32) -> DynamicImage {
    input_image.resize_exact(new_width, new_height, imageops::FilterType::Lanczos3)
}

// Paste the resized image onto the square image at the specified position
fn paste_resized_image(
    square_image: &mut DynamicImage,
    resized_image: &DynamicImage,
    paste_x: u32,
    paste_y: u32,
) {
    imageops::overlay(square_image, resized_image, paste_x as i64, paste_y as i64);
}

// Resize input image to a square with the specified output size
fn resize_to_square(input_image: &DynamicImage, output_size: u32) -> DynamicImage {
    let (input_width, input_height) = input_image.dimensions();
    let (new_width, new_height) = calculate_new_size(input_width, input_height, output_size);
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
        input_image.save(&file_path).expect("Failed to save input image");
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
        let input_image: DynamicImage = create_test_image(100, 150, Rgba([255, 0, 0, 255]));
        let mut input_buffer: Vec<u8> = Vec::new();

        image::codecs::png::PngEncoder::new(&mut input_buffer)
            .write_image(
                input_image.as_rgba8().unwrap().as_raw(),
                input_image.width(),
                input_image.height(),
                image::ExtendedColorType::Rgba8,
            )
            .expect("Failed to encode input image");

        // Call the convert function with the test image bytes
        let output_buffer: Vec<u8> = convert(&input_buffer, false);

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
        // Call the convert function with invalid input (empty buffer)
        let output_buffer: Vec<u8> = convert(&[], false);

        // The output buffer should be empty since the input is invalid
        assert!(output_buffer.is_empty());
    }

    // Validates the logic for calculating new dimensions.
    #[test]
    fn test_calculate_new_size() {
        assert_eq!(calculate_new_size(100, 150, 200), (133, 200));
        assert_eq!(calculate_new_size(400, 600, 200), (133, 200));
        assert_eq!(calculate_new_size(300, 300, 200), (200, 200));
        assert_eq!(calculate_new_size(100, 100, 200), (200, 200));
        assert_eq!(calculate_new_size(50, 100, 200), (100, 200));
        assert_eq!(calculate_new_size(100, 50, 200), (200, 100));
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
            .assert()
            .failure();

        let stderr = String::from_utf8(assert.get_output().stderr.clone()).unwrap();
        assert_eq!(stderr.contains("output file is not ico"), true);
    }

    #[test]
    fn test_convert_paths_with_invalid_input() {
        let (_, output_path) = create_temp_output_file("/output.ico");
        let input_path = "invalid.png".to_string();

        convert_paths(&input_path, &output_path);

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

        convert_paths(&input_path, &output_path);

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

        convert_paths(&input_path, &output_path);

        assert!(!std::path::Path::new(&output_path).exists());
    }

    #[test]
    fn test_convert_with_decode_error() {
        let invalid_input = vec![0, 1, 2, 3, 4, 5];
        let output_buffer = convert(&invalid_input, false);

        assert!(output_buffer.is_empty());
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

        // Call the convert function with the SVG content
        let output_buffer: Vec<u8> = convert(input_buffer, true);

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

        convert_paths(&input_path, &output_path);

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

}
