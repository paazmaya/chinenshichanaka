use clap::Parser;
use image::{imageops, DynamicImage, GenericImageView, ImageError, Pixel, Rgba};

// Input file support depends on the set of features in Cargo.toml
// https://github.com/image-rs/image?tab=readme-ov-file#supported-image-formats

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// The input image file
    #[arg(index = 1)]
    input: String,

    /// The output file which should end with ".ico"
    #[arg(index = 2)]
    output: String,
}

fn main() {
    let args: Args = Args::parse();

    println!("input file {}", args.input);
    println!("output file {}", args.output);

    match args.input.ends_with(".png") {
        true => {
            println!("input file is png");
            match args.output.ends_with(".ico") {
                true => {
                    println!("output file is ico");
                    convert(&args.input, &args.output);
                },
                false => {
                    println!("output file is not ico");
                }
            }
        },
        false => {
            println!("input file is not png");
        }
        
    }
}


pub fn convert(input: &str, output: &str) {
    // Open the input image but exit if it fails
    let img: Result<DynamicImage, ImageError> = image::open(input);
    let img: DynamicImage = match img {
        Ok(img) => img,
        Err(err) => {
            println!("Error: {}", err);
            return;
        }
    };

    // The dimensions method returns the images width and height.
    println!("Dimensions {:?}", img.dimensions());

    // The color method returns the image's `ColorType`.
    println!("ColorType {:?}", img.color());

    let img = resize_to_square(&img, 64);

    // The dimensions method returns the images width and height.
    println!("Dimensions {:?}", img.dimensions());

    // The color method returns the image's `ColorType`.
    println!("ColorType {:?}", img.color());

    img.save(output).expect("Failed to save output image");
}

fn calculate_new_size(input_width: u32, input_height: u32, output_size: u32) -> (u32, u32) {
    // Calculate the scaling factor to fit the input image within a square of the desired size
    let scale_factor = f64::min(output_size as f64 / input_width as f64, output_size as f64 / input_height as f64);

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
    imageops::overlay(&mut square_image, &DynamicImage::ImageRgb8(image::RgbImage::from_pixel(1, 1, background_color.to_rgb())), 0, 0);
    square_image
}

// Resize the input image using Lanczos3 filter for high-quality results
fn resize_image(input_image: &DynamicImage, new_width: u32, new_height: u32) -> DynamicImage {
    input_image.resize_exact(new_width, new_height, imageops::FilterType::Lanczos3)
}

// Paste the resized image onto the square image at the specified position
fn paste_resized_image(square_image: &mut DynamicImage, resized_image: &DynamicImage, paste_x: u32, paste_y: u32) {
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
    use std::fmt::Write;

    use super::*;
    use image::Rgb;
    use tempfile::NamedTempFile;
    use image::{imageops, DynamicImage, GenericImageView, Pixel, Rgba};

    // Helper function to create an image with specified dimensions and color
    fn create_test_image(width: u32, height: u32, color: Rgba<u8>) -> DynamicImage {
        let mut img: DynamicImage = DynamicImage::new_rgb8(width, height);
        imageops::overlay(&mut img, &DynamicImage::ImageRgb8(image::RgbImage::from_pixel(1, 1, color.to_rgb())), 0, 0);
        img
    }

    #[test]
    fn test_convert() {
        // Create a temporary input file with a known image
        let mut input_file = NamedTempFile::new().expect("Failed to create temporary input file");
        let input_image = create_test_image(100, 150, Rgba([255, 0, 0, 255]));
        
        // Save the input image in a specific format (e.g., PNG)
        input_image.save_with_format(&mut input_file, image::ImageFormat::Png).expect("Failed to save temporary input file");

        // Create a temporary output file
        let output_file: NamedTempFile = NamedTempFile::new().expect("Failed to create temporary output file");
        let output_path: String = format!("{}.ico", output_file.path().to_str().expect("Failed to convert output path to string"));

        // Call the convert function with the temporary files
        convert(input_file.path().to_str().expect("Failed to convert input path to string"), &output_path);

        // Open the output image and assert its properties
        let output_image: DynamicImage = image::open(output_path).expect("Failed to open output image");


        assert_eq!(output_image.dimensions(), (64, 64));
        assert_eq!(output_image.color(), image::ColorType::Rgb8);

        // Additional assertions based on your requirements
        // ...

        // Clean up temporary files
        input_file.close().expect("Failed to close temporary input file");
        output_file.close().expect("Failed to close temporary output file");
    }

    #[test]
    fn test_calculate_new_size() {
        assert_eq!(calculate_new_size(100, 150, 200), (133, 200));
        assert_eq!(calculate_new_size(400, 600, 200), (133, 200));
        assert_eq!(calculate_new_size(300, 300, 200), (200, 200));
        assert_eq!(calculate_new_size(100, 100, 200), (200, 200));
        assert_eq!(calculate_new_size(50, 100, 200), (100, 200));
        assert_eq!(calculate_new_size(100, 50, 200), (200, 100));
    }

    #[test]
    fn test_get_top_left_color() {
        let input_image = DynamicImage::ImageRgb8(image::RgbImage::from_pixel(1, 1, Rgb([255, 0, 0])));
        assert_eq!(get_top_left_color(&input_image), Rgba([255, 0, 0, 255]));
    }

    #[test]
    fn test_create_square_image() {
        let background_color = Rgba([255, 0, 0, 255]);
        let square_image = create_square_image(200, background_color);
        assert_eq!(square_image.dimensions(), (200, 200));
        assert_eq!(square_image.get_pixel(0, 0), Rgba([255, 0, 0, 255]));
    }

    #[test]
    fn test_resize_image() {
        let input_image = DynamicImage::ImageRgb8(image::RgbImage::from_pixel(1, 1, Rgb([255, 0, 0])));
        let resized_image = resize_image(&input_image, 100, 100);
        assert_eq!(resized_image.dimensions(), (100, 100));
    }

    #[test]
    fn test_paste_resized_image() {
        let mut square_image = DynamicImage::ImageRgb8(image::RgbImage::from_pixel(200, 200, Rgb([255, 255, 255])));
        let resized_image = DynamicImage::ImageRgb8(image::RgbImage::from_pixel(100, 100, Rgb([0, 0, 0])));
        paste_resized_image(&mut square_image, &resized_image, 50, 50);
        assert_eq!(square_image.get_pixel(50, 50), Rgba([0, 0, 0, 255]));
    }

    #[test]
    fn test_resize_to_square() {
        let input_image = DynamicImage::ImageRgb8(image::RgbImage::from_pixel(1, 1, Rgb([255, 0, 0])));
        let result = resize_to_square(&input_image, 200);
        assert_eq!(result.dimensions(), (200, 200));
        assert_eq!(result.get_pixel(50, 50), Rgba([255, 0, 0, 255]));
    }
}
