use clap::Parser;
use image::{imageops, DynamicImage, GenericImage, GenericImageView, ImageError, Pixel, Rgba};

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


fn convert(input: &str, output: &str) {
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


fn resize_to_square(input_image: &DynamicImage, output_size: u32) -> DynamicImage {
    let (input_width, input_height) = input_image.dimensions();

    // Calculate the scaling factor to fit the input image within a square of the desired size
    let scale_factor = f64::min(output_size as f64 / input_width as f64, output_size as f64 / input_height as f64);

    // Calculate the new dimensions after resizing
    let new_width = (input_width as f64 * scale_factor) as u32;
    let new_height = (input_height as f64 * scale_factor) as u32;
    
    // Get the color of the top-left pixel
    let top_left_color: Rgba<u8> = input_image.get_pixel(0, 0);
    println!("top_left_color {:?}", top_left_color);

    // Resize the input image using Lanczos3 filter for high-quality results
    let resized_image = input_image.resize_exact(new_width, new_height, imageops::FilterType::Lanczos3);

    // Create a new square image with the desired output size and fill it with the background color
    let mut square_image = DynamicImage::new_rgb8(output_size, output_size);
    imageops::overlay(&mut square_image, &DynamicImage::ImageRgb8(image::RgbImage::from_pixel(1, 1, top_left_color.to_rgb())), 0, 0);

    // Calculate the position to paste the resized image at the center of the square image
    let paste_x = (output_size - new_width) / 2;
    let paste_y = (output_size - new_height) / 2;

    // Paste the resized image onto the square image
    imageops::overlay(&mut square_image, &resized_image, paste_x as i64, paste_y as i64);

    square_image
}
