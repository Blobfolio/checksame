/*!
# `CheckSame`

`CheckSame` generates a single Blake3 hash from any number of files or directories.



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

#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::map_err_ignore)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::module_name_repetitions)]



use fyi_menu::{
	Argue,
	FLAG_REQUIRED,
};
use fyi_msg::{
	Msg,
	MsgKind,
};
use fyi_witcher::Witcher;
use std::{
	fmt,
	io,
	path::PathBuf,
};



#[derive(Debug, Copy, Clone, Eq, Hash, PartialEq)]
enum CheckSameKind {
	New,
	Changed,
	Same,
}

impl fmt::Display for CheckSameKind {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(
			f,
			"{}",
			match self {
				Self::New => "-1",
				Self::Changed => "1",
				Self::Same => "0",
			}
		)
	}
}



/// Main.
fn main() {
	// Parse CLI arguments.
	let mut args = Argue::new(FLAG_REQUIRED)
		.with_version(b"CheckSame", env!("CARGO_PKG_VERSION").as_bytes())
		.with_help(helper)
		.with_list();

	// Reset before we begin?
	if args.switch("--reset") { reset(); }

	// Hold the key for later.
	let key: String = args.option2("-k", "--key").map(String::from).unwrap_or_default();

	// Pull the file list everything.
	let mut files: Vec<PathBuf> = Witcher::default()
		.with_paths(args.args())
		.build();

	if files.is_empty() {
		MsgKind::Error.into_msg("At least one valid file path is required.")
			.eprintln();
		std::process::exit(1);
	}

	// Sort to keep results consistent.
	files.sort();

	// Add up all the hashes, then hash that for the final checksum.
	let chk = files.iter()
		.fold(
			blake3::Hasher::new(),
			|mut h, p| {
				if let Some(res) = hash_file(p) { h.update(&res); }
				h
			}
		)
		.finalize()
		.to_hex();

	// Just print the hash.
	if key.is_empty() { println!("{:?}", chk); }
	// Compare the old and new hash, save it, and print the state.
	else {
		println!("{}", save_compare(&chk, key));
	}
}

/// Hash File.
fn hash_file(path: &PathBuf) -> Option<[u8; 32]> {
	let mut file = std::fs::File::open(&path).ok()?;
	let mut hasher = blake3::Hasher::new();
	io::copy(&mut file, &mut hasher).ok()?;
	let hash = hasher.finalize();
	Some(*hash.as_bytes())
}

/// Reset.
fn reset() {
	if let Ok(entries) = std::fs::read_dir(tmp_dir()) {
		entries
			.filter_map(std::result::Result::ok)
			.for_each(|x| {
				let path = x.path();
				if path.is_file() {
					let _ = std::fs::remove_file(path);
				}
			});
	}
}

/// Save/Compare.
fn save_compare(chk: &str, key: String) -> CheckSameKind {
	use std::io::Write;

	let mut file = tmp_dir();
	file.push(key);

	let mut changed: CheckSameKind = CheckSameKind::New;

	// Did it already exist? Compare the new and old values.
	if file.is_file() {
		if std::fs::read_to_string(&file).unwrap_or_default() == chk {
			changed = CheckSameKind::Same;
		}
		else {
			changed = CheckSameKind::Changed;
		}
	}

	// Save it.
	if std::fs::File::create(&file)
		.and_then(
			|mut out|
			out.write_all(chk.as_bytes()).and_then(|_| out.flush())
		)
		.is_err()
	{
		MsgKind::Error.into_msg("Unable to store checksum.")
			.eprintln();
		std::process::exit(1);
	}

	changed
}

/// Get/Make Temporary Directory.
fn tmp_dir() -> PathBuf {
	let mut dir = std::env::temp_dir();
	dir.push("checksame");

	if ! dir.is_dir() && (dir.is_file() || std::fs::create_dir(&dir).is_err()) {
		MsgKind::Error.into_msg(&format!(
			"Unable to create temporary directory {:?}.",
			&dir
		))
			.eprintln();
		std::process::exit(1);
	}

	dir
}

#[cold]
/// Print Help.
fn helper(_: Option<&str>) {
	Msg::from(format!(
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
 ;  .     |            ,   {}{}{}
  ;  .    \           /|   Cumulative file hashing
   \  .    '.       .'/    and change detection.
    '.  '  . `'---'`.'
      `'-..._____.-`


USAGE:
    checksame [FLAGS] [OPTIONS] <PATH(S)>...

FLAGS:
    -h, --help        Prints help information.
        --reset       Reset any previously-saved hash keys before starting.
    -V, --version     Prints version information.

OPTIONS:
    -k, --key <list>     Store checksum under this keyname for change detection.
    -l, --list <list>    Read file paths from this list.

ARGS:
    <PATH(S)>...    One or more files or directories to compress.

When no key is provided, the hash will be printed. Otherwise a value of -1, 0,
or 1 will be printed, indicating NEW, UNCHANGED, or CHANGED, respectively.

",
		"\x1b[38;5;199mCheckSame\x1b[0;38;5;69m v",
		env!("CARGO_PKG_VERSION"),
		"\x1b[0m",
	)).print()
}
