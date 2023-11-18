# Merge PNA

[GitHub repository](https://github.com/tukinami/merge-pna)

## What is this?

Tool to merge PNG and PNA(grayscale image for alpha-channel).

## Usage
```
Usage: merge-pna.exe [OPTIONS] --input-path <INPUT_PATH>

Options:
  -i, --input-path <INPUT_PATH>    Path to input png file
  -p, --pna-path <PNA_PATH>        Path to pna file [default: PATH_TO_PNG_DIR/PNG_NAME.pna]
  -o, --output-path <OUTPUT_PATH>  Path to output png file [default: PATH_TO_PNG_DIR/PNG_NAME_new.png]
  -f, --force                      Flag of force overwriting output png
  -h, --help                       Print help information
  -V, --version                    Print version information
```

## Using Library

+ [png](https://github.com/image-rs/image-png) / The image-rs Developers
+ [clap](https://github.com/clap-rs/clap) / rust-cli/Maintainers, clap-rs/Admins, Kevin K.

## License

licensed under MIT.

## Author

月波 清火 (tukinami seika)

[GitHub](https://github.com/tukinami)
