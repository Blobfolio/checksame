/*!
# `CheckSame`

`CheckSame` is a recursive, cumulative Blake3 file hasher for x86-64 Linux machines.

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

For stable Rust (>= `1.51.0`), run:
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
*/

#![warn(clippy::filetype_is_file)]
#![warn(clippy::integer_division)]
#![warn(clippy::needless_borrow)]
#![warn(clippy::nursery)]
#![warn(clippy::pedantic)]
#![warn(clippy::perf)]
#![warn(clippy::suboptimal_flops)]
#![warn(clippy::unneeded_field_pattern)]
#![warn(macro_use_extern_crate)]
#![warn(missing_copy_implementations)]
#![warn(missing_debug_implementations)]
#![warn(missing_docs)]
#![warn(non_ascii_idents)]
#![warn(trivial_casts)]
#![warn(trivial_numeric_casts)]
#![warn(unreachable_pub)]
#![warn(unused_crate_dependencies)]
#![warn(unused_extern_crates)]
#![warn(unused_import_braces)]

#![allow(clippy::module_name_repetitions)]



mod error;
mod hash;

use argyle::{
	Argue,
	ArgyleError,
	FLAG_HELP,
	FLAG_REQUIRED,
	FLAG_VERSION,
};
use error::CheckSameError;
use fyi_msg::Msg;
use hash::{
	CheckSame,
	FLAG_CACHE,
	FLAG_RESET,
};
use std::{
	ffi::OsStr,
	os::unix::ffi::OsStrExt,
};



/// # Main.
fn main() {
	match _main() {
		Ok(_) | Err(CheckSameError::Noop) => {},
		Err(CheckSameError::Argue(ArgyleError::WantsVersion)) => {
			println!(concat!("CheckSame v", env!("CARGO_PKG_VERSION")));
		},
		Err(CheckSameError::Argue(ArgyleError::WantsHelp)) => {
			helper();
		},
		Err(e) => {
			Msg::error(e).die(1);
		},
	}
}

#[inline]
/// # Actual main.
fn _main() -> Result<(), CheckSameError> {
	// Parse CLI arguments.
	let args = Argue::new(FLAG_HELP | FLAG_REQUIRED | FLAG_VERSION)?
		.with_list();

	// Reset before we begin?
	let mut flags: u8 = 0;
	if args.switch(b"--reset") {
		flags |= FLAG_RESET;
	}
	if args.switch2(b"-c", b"--cache") {
		flags |= FLAG_CACHE;
	}

	// Build it.
	let hash = CheckSame::new(
		dowser::dowse(args.args().iter().map(|x| OsStr::from_bytes(x.as_ref()))),
		flags
	)?;

	// Print it.
	println!("{}", hash);

	// Done!
	Ok(())
}

#[cold]
/// Print Help.
fn helper() {
	println!(concat!(
		r"
          ______
      .-'` .    `'-.
    .'  '    .---.  '.
   /  '    .'     `'. \
  ;  '    /          \|
 :  '  _ ;            `
;  :  /(\ \
|  .       '.
|  ' /     --'
|  .   '.__\
;  :       /
 ;  .     |            ,   ", "\x1b[38;5;199mCheckSame\x1b[0;38;5;69m v", env!("CARGO_PKG_VERSION"), "\x1b[0m", r"
  ;  .    \           /|   Cumulative file hashing
   \  .    '.       .'/    and change detection.
    '.  '  . `'---'`.'
      `'-..._____.-`


USAGE:
    checksame [FLAGS] [OPTIONS] <PATH(S)>...

FLAGS:
    -c, --cache       Cache the hash and output the status.
    -h, --help        Prints help information.
        --reset       Reset any previously-saved hash keys before starting.
    -V, --version     Prints version information.

OPTIONS:
    -l, --list <list>    Read file paths from this list.

ARGS:
    <PATH(S)>...    One or more files or directories to compress.

By default, this will print a single 64-character Blake3 hash for the file(s)
to STDOUT.

In --cache mode, the hash will be cached and compared against the previous run.
A value of -1, 0, or 1 will be printed instead, indicating NEW, UNCHANGED, or
CHANGED, respectively.
",
	));
}
