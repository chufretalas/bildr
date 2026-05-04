# bildr

An image convolution program to have fun with convolutions.

## Usage

TODO

## Accepted image formats

TODO

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