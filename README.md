# bildr

An image convolution program to have fun with convolutions.
Built entirely in rust with as few dependencies as possible.

## Features
* **Custom Mathematical Kernels:** Write your own convolution matrices using the custom `.kbildr` format.
* **Real-time Visualization:** Watch the convolution process render line-by-line in a native window.
* **Padding:** Handle image edges with `Zero` or `Clamp` padding strategies.
* **Cross-Platform & Standalone:** Native 24-bit `.bmp` and P3/P6 `.ppm` parsing and encoding built without third-party image libraries.

## Installation

**Option 1: Download Pre-compiled Binaries**
Download linux or windows binaries from the [Releases Page](../../releases). 

**Option 2: Build from Source**
See the [Building](#building) section below.

## Usage

`bildr` operates via the command line. You must provide an input image, an output destination, and a kernel file.

**Basic Usage:**
```bash
bildr input.bmp output.bmp --kernel my_kernel.kbildr
```

**Full Options:**
```bash
bildr <INPUT> <OUTPUT> --kernel <KERNEL> [OPTIONS]
```

* `<INPUT>`: Path to the source image (e.g., `photo.bmp` or `image.ppm`).
* `<OUTPUT>`: Path where the result will be saved (e.g., `result.bmp`).
* `-k, --kernel <KERNEL>`: **(Required)** Path to the `.kbildr` kernel file.
* `-n, --normalize`: **(Optional)** Automatically normalizes the kernel weights.
* `-p, --padding <PADDING>`: **(Optional)** Edge handling strategy. 
  * `zero` (Default): Treats out-of-bounds pixels as black.
  * `clamp`: Extends the edge pixels infinitely outward.
* `-v, --visualize`: **(Optional)** Opens a native window showing the convolution process rendering line-by-line.

**Example with all features enabled:**
```bash
bildr input.bmp output.bmp -k edge_detect.kbildr --padding clamp --normalize --visualize
```

## Accepted Image Formats

`bildr` uses a custom, zero-dependency image I/O engine that strictly supports the following formats as input:

* **BITMAP (.bmp):** Standard **24-bit uncompressed** Bitmap files (Standard Windows/Krita export format).
* **PPM (.ppm):** Netpbm color image format. Both `P3` (ASCII text) and `P6` (Binary) magic numbers are supported. Max color depth must be 255 (8-bit per channel).

As for output files, bildr uses the destination path extension to decide between BITMAP or PPM.
* (.bmp): The output image will be saved as a 24-bit uncompressed bitmap.
* (.ppm): The output image will be saved as a P6 PPM.

## .kbildr

Custom kernels are fed to bildr as .kbildr files, which, unsurprisingly, stands for "bildr kernel".
It's a simple text file with the following structure
```
sf
r c
w11 w12 ... w1c
w21 w22 ... w2c
...
wr1 wr2 ... wrc
```
Where:
- **sf** (scaling factor) is a *float* parameter which serves as a multiplier to each pixel's convolution result.
- **r c** (rows columns) are *integers* which dictate respectively how many rows and columns the kernel matrix will have.
- **wyx** (weights), the rest of the file should consist of lines of float values separated by at least one whitespace.
    - It should follow the exact dimensions the **r** and **c** parameters.

**Comments**: If a line *starts with* '#', the line is considered a comment.

**OBS**: The parser uses Rust's str::parse() for floats, so it accepts numbers with no leading zero (.42 => 0.42), integers (42 => 42.0) and trailing dots (42. => 42.0)

**Example 1: Identity**
```
1
3 3
0.0 0.0 0.0
0.0 1.0 0.0
0.0 0.0 0.0
```

**Example 2: Blur Box**
```
# just a simple 4x4 blur box filter
0.0625
4 4
1  1  1  1
1  1  1  1
1  1  1  1
1  1  1  1 
```

**Example 3: Rectangular**
```
0.42
2 3
1.23  0     4
1.01  0.02  4.2
```

## building

This project is written in Rust. You will need [Rust and Cargo installed](https://rust-lang.org/tools/install/) to compile it.

### From linux to linux

```bash
cargo build --release
```

### From linux to windows

**1. Requirements for cross-compilation**
```bash
sudo apt update
sudo apt install mingw-w64
rustup target add x86_64-pc-windows-gnu
```

**2. Build the Executable**
```bash
cargo build --release --target x86_64-pc-windows-gnu
```