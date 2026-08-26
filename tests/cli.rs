use assert_cmd::Command;
use image::{imageops, DynamicImage, Pixel, Rgba};
use std::fs;
use std::path::Path;
use tempfile::NamedTempFile;

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

fn create_temp_image_file(extension: &str, color: Rgba<u8>) -> (NamedTempFile, String) {
    let temp_file = NamedTempFile::new().expect("Failed to create temp input file");
    let file_path = temp_file.path().to_str().unwrap().to_owned() + extension;
    let input_image = create_test_image(100, 150, color);
    input_image
        .save(&file_path)
        .expect("Failed to save input image");
    (temp_file, file_path)
}

fn create_temp_output_file(extension: &str) -> (tempfile::TempDir, String) {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let output_path = temp_dir.path().to_str().unwrap().to_owned() + extension;
    (temp_dir, output_path)
}

#[test]
fn test_cli_tool() {
    let temp_input: NamedTempFile = NamedTempFile::new().expect("Failed to create temp input file");
    let input_path = temp_input.path().to_str().unwrap().to_owned() + ".png";

    let input_image = create_test_image(100, 150, Rgba([255, 0, 0, 255]));
    input_image
        .save(&input_path)
        .expect("Failed to save input image");

    let temp_dir: tempfile::TempDir = tempfile::tempdir().expect("Failed to create temp dir");
    let output_path = temp_dir.path().to_str().unwrap().to_owned() + "/output.ico";

    Command::cargo_bin("chinenshichanaka")
        .expect("Binary not found")
        .arg(input_path)
        .arg(&output_path)
        .assert()
        .success();

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

    Command::cargo_bin("chinenshichanaka")
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

    let assert = Command::cargo_bin("chinenshichanaka")
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
fn test_main_with_valid_png_default_output() {
    let (_, input_path) = create_temp_image_file(".png", Rgba([255, 0, 0, 255]));
    let output_path = std::env::current_dir().unwrap().join("favicon.ico");

    Command::cargo_bin("chinenshichanaka")
        .expect("Binary not found")
        .arg(&input_path)
        .assert()
        .success();

    assert!(Path::new(&output_path).exists());
    let output_content = fs::read(&output_path).expect("Failed to read output file");
    let guess = image::guess_format(&output_content).expect("Failed to guess format");
    assert_eq!(guess, image::ImageFormat::Ico);

    fs::remove_file(output_path).expect("Failed to remove output file");
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

    Command::cargo_bin("chinenshichanaka")
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

    Command::cargo_bin("chinenshichanaka")
        .expect("Binary not found")
        .arg(&input_path)
        .assert()
        .success();

    assert!(Path::new(&output_path).exists());
    let output_content = fs::read(&output_path).expect("Failed to read output file");
    let guess = image::guess_format(&output_content).expect("Failed to guess format");
    assert_eq!(guess, image::ImageFormat::Ico);

    fs::remove_file(output_path).expect("Failed to remove output file");
}
