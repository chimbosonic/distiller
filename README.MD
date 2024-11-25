# Distiller
[![crates.io](https://img.shields.io/crates/v/distiller.svg)](https://crates.io/crates/distiller)
![pipeline status](https://github.com/chimbosonic/distiller/actions/workflows/build.yml/badge.svg?branch=main)

`distiller` is a command line tool written in rust, used for extracting comment from source code and storing the results into a queryable sqlite database.

## Installation
`distiller` is written in [Rust](https://www.rust-lang.org/). You can clone the repo and run to compile and install the binary to `${HOME}/.cargo/bin/distiller`

```bash
cargo install --path .   
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
distiller is a command line tool written in rust, used for extracting comment from source code and storing the results into a queryable sqlite database.

Usage: distiller [OPTIONS] --input <INPUT>

Options:
  -o, --output <OUTPUT>  Sets the output db file defaults to results.db
  -i, --input <INPUT>    Sets the source directory to parse
  -h, --help             Print help
  -V, --version          Print version
```

## Contributing
Pull requests are welcome. For major changes, please open an issue first to discuss what you would like to change.

Please make sure to update tests as appropriate.

## License
[MIT](https://choosealicense.com/licenses/mit/)
