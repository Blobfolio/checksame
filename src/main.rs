/*!
# `CheckSame`
*/

#![forbid(unsafe_code)]

#![warn(
	clippy::filetype_is_file,
	clippy::integer_division,
	clippy::needless_borrow,
	clippy::nursery,
	clippy::pedantic,
	clippy::perf,
	clippy::suboptimal_flops,
	clippy::unneeded_field_pattern,
	macro_use_extern_crate,
	missing_copy_implementations,
	missing_debug_implementations,
	missing_docs,
	non_ascii_idents,
	trivial_casts,
	trivial_numeric_casts,
	unreachable_pub,
	unused_crate_dependencies,
	unused_extern_crates,
	unused_import_braces,
)]

#![allow(
	clippy::module_name_repetitions,
	clippy::redundant_pub_crate,
)]



mod error;
mod hash;

use argyle::{
	Argue,
	ArgyleError,
	FLAG_HELP,
	FLAG_REQUIRED,
	FLAG_VERSION,
};
use dowser::Dowser;
use error::CheckSameError;
use fyi_msg::Msg;
use hash::{
	CheckSame,
	FLAG_CACHE,
	FLAG_RESET,
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
		Dowser::default().with_paths(args.args_os()).collect(),
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
    -h, --help        Print help information and exit.
        --reset       Reset any previously-saved hash keys before starting.
    -V, --version     Print version information and exit.

OPTIONS:
    -l, --list <FILE> Read (absolute) file and/or directory paths from this
                      text file, one entry per line.

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
