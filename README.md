# CheckSame

[![ci](https://img.shields.io/github/workflow/status/Blobfolio/checksame/Build.svg?style=flat-square&label=ci)](https://github.com/Blobfolio/checksame/actions)
[![deps.rs](https://deps.rs/repo/github/blobfolio/checksame/status.svg?style=flat-square&label=deps.rs)](https://deps.rs/repo/github/blobfolio/checksame)<br>
[![license](https://img.shields.io/badge/license-wtfpl-ff1493?style=flat-square)](https://en.wikipedia.org/wiki/WTFPL)
[![contributions welcome](https://img.shields.io/badge/PRs-welcome-brightgreen.svg?style=flat-square&label=contributions)](https://github.com/Blobfolio/checksame/issues)

CheckSame is a recursive, cumulative [Blake3](https://en.wikipedia.org/wiki/BLAKE_(hash_function)#BLAKE3) file hasher for x86-64 Linux machines.

It is "cumulative" in the sense that it computes a _single_ hash representing all of the files passed to it, rather than individual hashes for each file.

By default, this hash is simply printed to STDOUT.

However, when run with `-c` or `--cache`, the resulting hash will be stored and compared against the previous run. In this mode, the program will output one of:

| Value | Meaning |
| ----- | ------- |
| -1 | No hash was previously stored. |
| 0 | No change detected. |
| 1 | Something changed. |

The cache mode is primarily intended to provide an efficient bypass for expensive build routines, or as a way to quickly see if a directory's contents have changed (beyond mere timestamp updates).

The cache lives in `/tmp/checksame` and can be cleared by running the program with the `--reset` flag, or simply deleting the files in that directory. On most systems, that directory should disappear automatically on reboot.



## Installation

Debian and Ubuntu users can just grab the pre-built `.deb` package from the [latest release](https://github.com/Blobfolio/checksame/releases/latest).

This application is written in [Rust](https://www.rust-lang.org/) and can alternatively be built from source using [Cargo](https://github.com/rust-lang/cargo):

```bash
# Clone the source.
git clone https://github.com/Blobfolio/checksame.git

# Go to it.
cd checksame

# Build as usual. Specify additional flags as desired.
cargo build \
    --bin checksame \
    --release
```

(This should work under other 64-bit Unix environments too, like MacOS.)



## Usage

It's easy. Just run `checksame [FLAGS] [OPTIONS] <PATH(S)>…`.

The following flags and options are available:

| Short | Long | Value | Description |
| ----- | ---- | ----- | ----------- |
| `-c` | `--cache` | | Cache the hash and output the status. |
| `-h` | `--help` | | Print help information and exit. |
| `-l` | `--list` | `<FILE>` | Read (absolute) file and/or directory paths from this text file, one entry per line. |
| | `--reset` | | Reset any previously-saved hash keys before starting. |
| `-V` | `--version` | | Print program version and exit. |

For example:
```bash
# Generate checksum by passing any number of file and directory paths.
# You can also place paths in a text file — one per line — and add
# that to the mix with the -l option.
checksame -l /path/to/list.txt /path/to/app.js /path/to/folder

# Avoid doing something expensive if nothing changed.
[ "$( checksame -c -l /path/list.txt )" = "0" ] || ./expensive-task
```


## License

See also: [CREDITS.md](CREDITS.md)

Copyright © 2022 [Blobfolio, LLC](https://blobfolio.com) &lt;hello@blobfolio.com&gt;

This work is free. You can redistribute it and/or modify it under the terms of the Do What The Fuck You Want To Public License, Version 2.

    DO WHAT THE FUCK YOU WANT TO PUBLIC LICENSE
    Version 2, December 2004
    
    Copyright (C) 2004 Sam Hocevar <sam@hocevar.net>
    
    Everyone is permitted to copy and distribute verbatim or modified
    copies of this license document, and changing it is allowed as long
    as the name is changed.
    
    DO WHAT THE FUCK YOU WANT TO PUBLIC LICENSE
    TERMS AND CONDITIONS FOR COPYING, DISTRIBUTION AND MODIFICATION
    
    0. You just DO WHAT THE FUCK YOU WANT TO.
