# CheckSame

CheckSame is a recursive, cumulative file hasher for x86-64 Linux machines.

By default, it simply prints a Blake3 hash representing all file paths passed to it, but it can also be used for cached change detection by passing `-k` or `--key` — any arbitrary string made up of alphanumeric characters, `-`, and/or `_` — in which case it will output:

| Value | Meaning |
| ----- | ------- |
| -1 | No hash was previously stored. |
| 0 | No change detected. |
| 1 | Something changed. |

The key comparison mode is primarily intended to provide an efficient bypass for expensive build routines, etc.



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
-h, --help        Prints help information.
-k, --key <key>   Store checksum under this keyname for change detection.
-l, --list <list> Read file paths from this list.
    --reset       Reset any previously-saved hash keys before starting.
-V, --version     Prints version information.
```

For example:
```bash
# Generate checksum for one file.
checksame /path/to/app.js

# Generate cumulative checksum for all files in a folder.
checksame /path/to/assets

# Avoid doing something expensive if nothing changed.
[ "$( checksame -k MyTask -l /path/list.txt )" = "0" ] || ./expensive-task
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
