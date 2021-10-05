# bcdl
A command line bandcamp downloader.

## Installation
From crates.io
```shell
$ cargo install bcdl
```

From release page:
```shell
$ curl https://github.com/WORKINPROGRESS
```

## Example
```
$ bcdl --url https://citiesaviv.bandcamp.com/album/gum-2
Downloading album page...
Downloading Track 1 - (I SEEN YOU) SHINE     5M [#################################################] 100%
Downloading Track 2 - OVER                   2M [#################################################] 100%
Downloading Track 3 - WORLD MADE OF MARBLE   5M [#################################################] 100%
Downloading Track 4 - SUDDENLY EVAPORATE     2M [#################################################] 100%
Downloading Track 5 - CALL TOWER             1M [#################################################] 100%
Downloading Track 6 - MOBO                   1M [#################################################] 100%
Downloading Track 7 - STANDING BY THE 260    6M [#################################################] 100%
Downloading Track 8 - GESTURES               8M [#################################################] 100%
Downloading Track 9 - TAMIKA                 2M [#################################################] 100%
Downloading Track 10 - POWER APPROACHES     43M [#################################################] 100%
Downloaded 10 Tracks and 77.62M in 11 Seconds.
```

## Command Line Options
```
bcdl 0.1.0
Grant Handy <grantshandy@gmail.com>
A command line bandcamp downloader

USAGE:
    bcdl [FLAGS] [OPTIONS] --url <url>

FLAGS:
    -d, --debug      Don't actually save the songs for debugging purposes
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -p, --path <path>    Path to save songs to
    -u, --url <url>      Download from bandcamp URL
```