# `bids-subset`

Fast and easy BIDS dataset subsetting.

## Features

- Cross platform (Unix symlinks by default, windows copies)
- Easy (Glob pattern matching for subjects, sessions, datatypes and files)
- Fast

## Usage

```
Usage: bids-subset [OPTIONS] <PATH>

Arguments:
  <PATH>  Path to input BIDS dataset

Options:
  -o, --output <OUTPUT>      Path to output BIDS dataset
  -s, --subject <SUBJECT>    Subject glob pattern
  -e, --session <SESSION>    Session glob pattern
  -d, --datatype <DATATYPE>  BIDS Datatype glob pattern (anat, func, ...)
  -f, --file <FILE>          File filter pattern
  -x, --exclude-top-level    Exclude top level metadata files
  -c, --copy                 Enable copy mode (default on linux is symlink)
  -h, --help                 Print help
  -V, --version              Print version
```

## Examples

Extract session "01" from a dataset:

```bash
bids-subset path/to/bids -s 01
```

Extract files containing "run-1" from a dataset:

```bash
bids-subset path/to/bids -f "*run-1*"
```

Extract session "01" and "02" from a dataset:

```bash
bids-subset path/to/bids -e "{01,02}"
```

All of the above can be combined:

```bash
bids-subset path/to/bids -s 01 -e "{01,02}" -f "*run-1*"
```

