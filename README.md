# bflyt-rs

A library/command line tool for working with BFLYT files, used for layout editing with ui2d.

## Usage
```
Usage: bflyt-tool [OPTIONS] <COMMAND>

Commands:
  unpack  Convert from bflyt to json
  pack    Convert from json to bflyt
  help    Print this message or the help of the given subcommand(s)

Options:
  -o, --out <OUT>      
  -p, --print <PRINT>  [possible values: true, false]
  -h, --help           Print help
```

Examples:
- Unpack BFLYT to file 
  - `bflyt-tool unpack info_training.bflyt -o info_training.json`
- Pack BFLYT from JSON
  - `bflyt-tool pack info_training.json -o out.bflyt`
- Unpack BFLYT to console 
  - `bflyt-tool unpack info_training.bflyt --print`

## Build
Install cargo, and build with nightly.
```bash
cargo +nightly build --release
```
