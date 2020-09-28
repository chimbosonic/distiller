# Distiller
![crates.io](https://img.shields.io/crates/v/distiller.svg)
`distiller` is a command line tool written in rust, used for extracting comment from source code and storing the results into a queryable sqlite database.

## Installation

`distiller` is written in [Rust](https://www.rust-lang.org/). You can clone the repo and run to compile and install the binary to `${HOME}/.cargo/bin/distiller`

```bash
cargo install --path . --profile release -Z unstable-options   
```
### cargo install
You can also run 

```bash
cargo install distiller
```
This will install it from [crates.io](https://crates.io) to `${HOME}/.cargo/bin/distiller`

## Usage

```bash
distiller --help
distiller 1.0
Extracts all comments in source code to sqlite db

USAGE:
    distiller [OPTIONS] --input <DIRECTORY>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -i, --input <DIRECTORY>    Sets the source directory to parse
    -o, --output <FILE>        Sets the output db file defaults to results.db
```

## Contributing
Pull requests are welcome. For major changes, please open an issue first to discuss what you would like to change.

Please make sure to update tests as appropriate.

## License
[MIT](https://choosealicense.com/licenses/mit/)
