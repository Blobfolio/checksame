# CheckSame

CheckSame is a recursive, cumulative Blake3 file hasher for x86-64 Linux machines.

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

This application is written in [Rust](https://www.rust-lang.org/) and can be installed using [Cargo](https://github.com/rust-lang/cargo).

For stable Rust (>= `1.47.0`), run:
```bash
RUSTFLAGS="-C link-arg=-s" cargo install \
    --git https://github.com/Blobfolio/checksame.git \
    --bin checksame \
    --target x86_64-unknown-linux-gnu
```

Pre-built `.deb` packages are also added for each [release](https://github.com/Blobfolio/checksame/releases/latest). They should always work for the latest stable Debian and Ubuntu.



## Usage

It's easy. Just run `checksame [FLAGS] [OPTIONS] <PATH(S)>…`.

The following flags and options are available:
```bash
-c, --cache       Cache the hash and output the status.
-h, --help        Prints help information.
-l, --list <list> Read file paths from this list.
    --reset       Reset any previously-saved hash keys before starting.
-V, --version     Prints version information.
```

For example:
```bash
# Generate checksum by passing any number of file and directory paths.
# You can also place paths in a text file — one per line — and add
# that to the mix with the -l option.
checksame -l /path/to/list.txt /path/to/app.js /path/to/folder

# Avoid doing something expensive if nothing changed.
[ "$( checksame -c -l /path/list.txt )" = "0" ] || ./expensive-task
```



## Credits

| Library | License | Author |
| ---- | ---- | ---- |
| [blake3](https://crates.io/crates/blake3) | CC0-1.0 OR Apache-2.0 | Jack O'Connor |



## License

Copyright © 2020 [Blobfolio, LLC](https://blobfolio.com) &lt;hello@blobfolio.com&gt;

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
