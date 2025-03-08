# chinenshichanaka (知念志喜屋仲)

> Convert an image to a favicon with the size 32x32

Quality checks:

* [![Rust CI](https://github.com/paazmaya/chinenshichanaka/actions/workflows/rust-ci.yml/badge.svg)](https://github.com/paazmaya/chinenshichanaka/actions/workflows/rust-ci.yml) Testing and build installers at GitHub Actions
* [![codecov](https://codecov.io/gh/paazmaya/chinenshichanaka/graph/badge.svg?token=MCCGGycixe)](https://codecov.io/gh/paazmaya/chinenshichanaka) Unit test code coverage at Codecov
* [![Codacy Badge](https://app.codacy.com/project/badge/Grade/4175020c06ba4f2097a9b40a77de0003)](https://app.codacy.com/gh/paazmaya/chinenshichanaka/dashboard?utm_source=gh&utm_medium=referral&utm_content=&utm_campaign=Badge_grade) Code quality measurement at Coday
* [![Code Smells](https://sonarcloud.io/api/project_badges/measure?project=paazmaya_chinenshichanaka&metric=code_smells)](https://sonarcloud.io/summary/new_code?id=paazmaya_chinenshichanaka) Code smells at Sonarcloud (which does not support Rust)

![Okapi smiling](./icon-128x128.png)

It was sometimes challenging to get the favicon size right, so I made this. 
The generated `favicon.ico` (or any other `.ico` output file name you choose) is a square, 32 pixels in both width and height.

The input image file support depends on the set of features set in `Cargo.toml` and thus some additional libraries need to be available when compiling the application.
More details at https://github.com/image-rs/image?tab=readme-ov-file#supported-image-formats

## Background for the project name

The name of the project (chinenshichanaka, 知念志喜屋仲) is for honoring the legacy of a certain master from the Ryukyu 
archipelago, Japan, who contributed to the martial arts that we today know as **karate** and **ryukyu kobujutsu**.

[Read more about why these martial arts are important to me at `karatejukka.fi`.](https://karatejukka.fi)

## Installation

```sh
cargo install chinenshichanaka
```

## Usage

After installation and having the executable available in the `PATH` variable, the input image file is 
specified as the first argument, and the output icon file optionally as the second argument:

```sh
chinenshichanaka "image of okapi.png" "favicon.ico"
```

Now there should be the resulting `favicon.ico` file in the current folder.

The file can be checked, for example, with [GraphicsMagick](http://www.graphicsmagick.org/):

```sh
gm identify favicon.ico
# favicon.ico=> ICO 32x32+0+0 DirectClass 8-bit 1.7Ki 0.000u 0m:0.000001s
```

## License

[Licensed under the MIT license.](./LICENSE)
