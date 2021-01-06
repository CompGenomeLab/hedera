# hedera

This tool creates a profile plot for scores over sets of genomic regions. Typically, these regions are genes, but any other regions defined in BED will work.

## Usage

**NOTE: Currently `hedera` requires `bedtools` to be available at `$PATH`**

```
hedera 0.1.0
Ümit Akköse <umieat@gmail.com>
This tool creates a profile plot for scores over sets of genomic regions. Typically, these regions
are genes, but any other regions defined in BED will work.

USAGE:
    hedera [SUBCOMMAND]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

SUBCOMMANDS:
    help               Prints this message or the help of the given subcommand(s)
    reference-point    Reference-point refers to a position within a BED region (e.g., the
                       starting point). In this mode, only those genomic positions before
                       (upstream) and/or after (downstream) of the reference point will be
                       plotted.

```

#### reference-point

```
hedera-reference-point
Reference-point refers to a position within a BED region (e.g., the starting point). In this mode,
only those genomic positions before (upstream) and/or after (downstream) of the reference point will
be plotted.

USAGE:
    hedera reference-point [FLAGS] [OPTIONS] --regions <FILE> --reads <FILE>... --outFileName <FILE>

FLAGS:
        --relative    Plot scores of first reads relative to second reads (For each bin value of
                      first reads will be divided by value of second reads)

    -h, --help        Prints help information
    -V, --version     Prints version information

OPTIONS:
    -R, --regions <FILE>
            bed file containing regions to plot.
    -S, --reads <FILE>...
            bed file(s) containing the reads to be plotted. If multiple bed files are given, each
            one will be plotted as seperate line. Multiple files should be separated by space.
    -a, --downstream <INT>
            Distance downstream of the reference-point selected. [default: 500]
    -b, --upstream <INT>
            Distance upstream of the reference-point selected. [default: 500]
        --binSize <INT>
            Length, in bases, of the non-overlapping bins for averaging the score over the regions
            length. [default: 10]

        --referencePoint <referencePoint>
            The reference point for the plotting could be either the region start, the region end or
            the center of the region. [default: center] [possible values: start, end, center]

    -O, --outFileName <FILE>                 File name to save the plot.

        --plotTitle <plotTitle>              Title of the plot  [default: name of the regions file]
        --plotHeight <INT>                   Height of the plot [default: 720]
        --plotWidth <INT>                    Width of the plot  [default: 1280]

```

#### Basic Usage

```
hedera reference-point \
    -R regions.bed \
    -S reads.bed \
    -O profile.png

```

## Installation

Prebuilt binaries for macOS and Linux can be downloaded from the [GitHub releases page](https://github.com/CompGenomeLab/hedera/releases).

hedera is written in Rust, so you'll need to grab a [Rust installation](https://www.rust-lang.org/) in order to install or compile it with `cargo`.

Use `cargo` to install

```
$ cargo install --branch main --git https://github.com/CompGenomeLab/hedera.git
```

Or build from source

```
$ git clone https://github.com/CompGenomeLab/hedera.git
$ cd hedera
$ cargo build --release
$ ./target/release/hedera --version
0.1.0
```