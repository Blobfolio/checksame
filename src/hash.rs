/*!
# `CheckSame` - Hasher
*/

use ahash::AHasher;
use blake3::{
	Hash,
	Hasher as BHasher,
};
use rayon::{
	iter::{
		IntoParallelIterator,
		ParallelIterator,
	},
	slice::ParallelSliceMut,
};
use std::{
	fmt,
	fs::File,
	hash::Hasher,
	path::{
		Path,
		PathBuf,
	},
};
use super::CheckSameError;



/// # Reset First.
///
/// When set, all existing `CheckSame` cache files will be deleted. When
/// combined with [`FLAG_CACHE`], the result will always look "new".
pub(super) const FLAG_RESET: u8 = 0b0001;

/// # Cache Mode.
///
/// Print the change status rather than the hash. This is either -1, 1, or 0,
/// indicating no previous cache, something changed, or all's the same,
/// respectively.
pub(super) const FLAG_CACHE: u8 = 0b0010;



#[derive(Debug, Clone, Copy)]
/// # Status.
///
/// This is a list of cache statuses, used internally by [`CheckSame`].
enum CheckedSame {
	/// # We aren't worried about caching.
	Noop,
	/// # No change.
	Same,
	/// # The cache changed.
	Changed,
	/// # No previous cache.
	New,
}



#[derive(Debug)]
/// # `CheckSame`.
///
/// This struct holds the hash data for a set of paths. The only public-facing
/// method is [`CheckSame::new`], which does all the work.
///
/// The resulting object can be sent to any formatted writer accepting
/// `Display`. If [`FLAG_CACHE`] is set, this will print the status; otherwise
/// the hash is printed.
pub(super) struct CheckSame {
	/// # Key Hash.
	///
	/// This hash is used to calculate a unique file path for the set. It is
	/// calculated by hashing all of the file paths in order.
	key: u64,

	/// # Hash.
	///
	/// This is the cumulative `Blake3` hash of all included files. It is
	/// calculated by hashing each file individually, in order, then hashing
	/// those hashes.
	///
	/// This avoids the overhead of having to keep all file contents in memory
	/// long enough to come up with a single hash.
	hash: Hash,

	/// # Cache status.
	///
	/// This holds the cache status of the set. When not in cache mode, this is
	/// always [`CheckedSame::Noop`] and serves no purpose.
	status: CheckedSame,
}

impl fmt::Display for CheckSame {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self.status {
			CheckedSame::Noop => f.write_str(&self.hash.to_hex()),
			CheckedSame::Same => f.write_str("0"),
			CheckedSame::Changed => f.write_str("1"),
			CheckedSame::New => f.write_str("-1"),
		}
	}
}

impl From<Vec<PathBuf>> for CheckSame {
	fn from(paths: Vec<PathBuf>) -> Self {
		// First pass, hash all the files, consuming the original vector.
		let mut raw: Vec<(String, [u8; 32])> = paths.into_par_iter()
			.map(|p| {
				let hash = hash_file(&p);
				(p.to_string_lossy().into_owned(), hash)
			})
			.collect();

		// Resort by path for consistency.
		raw.par_sort_by(|(a, _), (b, _)| a.cmp(b));

		// Second pass, build the cumulative file/key hashes.
		let mut key_h = AHasher::new_with_keys(1319, 2371);
		let mut all_h = BHasher::new();
		for (p, h) in raw {
			key_h.write(p.as_bytes());
			all_h.update(&h);
		}

		// We're done!
		Self {
			key: key_h.finish(),
			hash: all_h.finalize(),
			status: CheckedSame::Noop,
		}
	}
}

impl CheckSame {
	/// # New Instance.
	///
	/// This generates a new instance from a set of paths and flags. This is
	/// the only public method for the struct; after running, make use of its
	/// `Display` implementation to print the results.
	///
	/// If [`FLAG_RESET`] is passed, all existing cache references will be
	/// cleared.
	///
	/// If [`FLAG_CACHE`] is passed, the resulting hash will be cached to a
	/// temporary file, and `Display` will print the status (compared to any
	/// previous cache) rather than the hash itself.
	///
	/// ## Errors
	///
	/// This will return an error if the path list is empty or any reset/cache
	/// operations fail for any reason.
	pub(crate) fn new(paths: Vec<PathBuf>, flags: u8) -> Result<Self, CheckSameError> {
		// If there are no paths, there's (probably) nothing for us to do.
		if paths.is_empty() {
			// We need to reset any other random caches before leaving.
			// Assuming the reset goes all right, this is a no-op rather than
			// a shameful error.
			if 0 != flags & FLAG_RESET {
				reset(&tmp_dir()?)?;
				return Err(CheckSameError::Noop);
			}

			// Otherwise shame!
			return Err(CheckSameError::NoFiles);
		}

		// Consume and build.
		let mut out = Self::from(paths);

		// We need to do something with the cache directory.
		if 0 < flags {
			let cache_dir = tmp_dir()?;

			// Reset the cache?
			if 0 != flags & FLAG_RESET {
				reset(&cache_dir)?;
			}

			// Check the cache?
			if 0 != flags & FLAG_CACHE {
				out.check_same(cache_dir)?;
			}
		}

		Ok(out)
	}

	/// # Check Sameness.
	///
	/// This checks the temporary file cache to see if there is a previous
	/// result for the path set, and if that value was different than the new
	/// hash.
	///
	/// If there is a change, the cache is updated accordingly.
	///
	/// ## Errors
	///
	/// This method returns an error if the cache file cannot be written to.
	fn check_same(&mut self, mut path: PathBuf) -> Result<(), CheckSameError> {
		use std::io::Write;

		// Generate a file path for the cache.
		path.push(self.key.to_string());

		// Get the hash as bytes.
		let bytes: &[u8] = self.hash.as_bytes();

		// This is already cached.
		if path.is_file() {
			// If it is unchanged, we're done!
			if std::fs::read(&path).unwrap_or_default() == bytes {
				self.status = CheckedSame::Same;
				return Ok(());
			}

			self.status = CheckedSame::Changed;
		}
		// This is something new.
		else {
			self.status = CheckedSame::New;
		}

		// Save it for next time.
		File::create(&path)
			.and_then(|mut out| out.write_all(bytes).and_then(|_| out.flush()))
			.map_err(|_| CheckSameError::Write)
	}
}



/// # Hash File.
///
/// Hash the contents of a file path if possible, returning the hash bytes on
/// success.
fn hash_file(path: &Path) -> [u8; 32] {
	if let Ok(mut file) = File::open(path) {
		let mut hasher = BHasher::new();
		if std::io::copy(&mut file, &mut hasher).is_ok() {
			return <[u8; 32]>::from(hasher.finalize());
		}
	}

	// Default to zeroes.
	[b'0'; 32]
}

/// # Reset Cache.
///
/// This will attempt to remove all `CheckSame` caches, which are just stored
/// as files in a temporary directory generated by the program.
///
/// Note: this does not delete the directory itself; that is preserved for
/// future use.
///
/// ## Errors
///
/// This method returns an error in cases where the temporary directory cannot
/// be read, or any files within it cannot be deleted.
fn reset(cache_dir: &Path) -> Result<(), CheckSameError> {
	let entries = std::fs::read_dir(cache_dir).map_err(|_| CheckSameError::Reset)?;
	entries.filter_map(Result::ok).try_for_each(|path| {
		let path = path.path();
		if path.is_file() {
			std::fs::remove_file(path).map_err(|_| CheckSameError::Reset)?;
		}
		Ok(())
	})
}

/// # Get/Make Temporary Directory.
///
/// This retrieves/creates a temporary directory to store `CheckSame` cache
/// files in.
///
/// ## Errors
///
/// This will return an error if the directory path is blocked or cannot be
/// created.
fn tmp_dir() -> Result<PathBuf, CheckSameError> {
	// The directory has to exist.
	let dir = std::env::temp_dir().join("checksame");
	if ! dir.is_dir() && (dir.exists() || std::fs::create_dir(&dir).is_err()) {
		Err(CheckSameError::Tmp)
	}
	else { Ok(dir) }
}
