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



use argyle::{
	Argue,
	ArgyleError,
	FLAG_HELP,
	FLAG_REQUIRED,
	FLAG_VERSION,
};
use dowser::{
	dowse,
	utility::path_as_bytes,
};
use fyi_msg::Msg;
use std::{
	ffi::OsStr,
	fmt,
	io,
	os::unix::ffi::OsStrExt,
	path::{
		Path,
		PathBuf,
	},
};



#[derive(Debug, Copy, Clone, Eq, Hash, PartialEq)]
enum CheckSameKind {
	New,
	Changed,
	Same,
}

impl fmt::Display for CheckSameKind {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.write_str(
			match self {
				Self::New => "-1",
				Self::Changed => "1",
				Self::Same => "0",
			}
		)
	}
}



/// # Main.
fn main() {
	match _main() {
		Ok(_) => {},
		Err(ArgyleError::WantsVersion) => {
			println!(concat!("CheckSame v", env!("CARGO_PKG_VERSION")));
		},
		Err(ArgyleError::WantsHelp) => {
			helper();
		},
		Err(e) => {
			Msg::error(e).die(1);
		},
	}
}

#[inline]
/// # Actual main.
fn _main() -> Result<(), ArgyleError> {
	// Parse CLI arguments.
	let args = Argue::new(FLAG_HELP | FLAG_REQUIRED | FLAG_VERSION)?
		.with_list();

	// Reset before we begin?
	if args.switch(b"--reset") { reset()?; }

	// Are we in check mode?
	let cache = args.switch2(b"-c", b"--cache");

	// Pull the file list.
	let mut files: Vec<PathBuf> = dowse(
		args.args().iter().map(|x| OsStr::from_bytes(x.as_ref()))
	);

	if files.is_empty() {
		// We don't need to require new files when resetting.
		if args.switch(b"--reset") {
			return Ok(());
		}

		return Err(ArgyleError::Custom("At least one valid file path is required."));
	}

	// Sort paths to keep results consistent.
	files.sort();

	// It is faster to hash each file separately and then hash the hashes
	// rather than feeding the byte content of each file into a single hasher.
	let chk = files.iter()
		.fold(
			blake3::Hasher::new(),
			|mut h, p| {
				if let Some(res) = hash_file(p) { h.update(&res); }
				h
			}
		)
		.finalize();

	// Compare the old and new hash, save it, and print the state.
	if cache {
		println!("{}", save_compare(
			chk.as_bytes(),
			&files.iter()
				.fold(
					blake3::Hasher::new(),
					|mut h, p| {
						h.update(path_as_bytes(p));
						h
					}
				)
				.finalize()
				.to_hex()
		)?);
	}
	// Just print the hash.
	else { println!("{}", chk.to_hex()); }

	Ok(())
}

/// # Hash File.
fn hash_file(path: &Path) -> Option<[u8; 32]> {
	let mut file = std::fs::File::open(&path).ok()?;
	let mut hasher = blake3::Hasher::new();
	io::copy(&mut file, &mut hasher).ok()?;
	let hash = hasher.finalize();
	Some(*hash.as_bytes())
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

/// # Reset.
fn reset() -> Result<(), ArgyleError> {
	let entries = std::fs::read_dir(tmp_dir()?)
		.map_err(|_| ArgyleError::Custom("Unable to reset cache."))?;

	entries
		.filter_map(std::result::Result::ok)
		.for_each(|x| {
			let path = x.path();
			if path.is_file() {
				let _ = std::fs::remove_file(path);
			}
		});

	Ok(())
}

/// # Save/Compare.
fn save_compare(chk: &[u8; 32], key: &str) -> Result<CheckSameKind, ArgyleError> {
	use std::io::Write;

	let mut file = tmp_dir()?;
	file.push(key);

	// Did it already exist? Compare the new and old values.
	let mut changed: CheckSameKind = CheckSameKind::New;
	if file.is_file() {
		if std::fs::read(&file).unwrap_or_default() == chk {
			changed = CheckSameKind::Same;
		}
		else {
			changed = CheckSameKind::Changed;
		}
	}

	// Save it.
	std::fs::File::create(&file)
		.and_then(
			|mut out|
			out.write_all(chk).and_then(|_| out.flush())
		)
		.map_err(|_| ArgyleError::Custom("Unable to save cache."))?;

	Ok(changed)
}

/// # Get/Make Temporary Directory.
fn tmp_dir() -> Result<PathBuf, ArgyleError> {
	let mut dir = std::env::temp_dir();
	dir.push("checksame");

	if ! dir.is_dir() && (dir.exists() || std::fs::create_dir(&dir).is_err()) {
		Err(ArgyleError::Custom("Unable to create temporary directory."))
	}
	else { Ok(dir) }
}