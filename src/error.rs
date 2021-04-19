/*!
# `CheckSame` - Error
*/

use argyle::ArgyleError;
use std::error::Error;
use std::fmt;



#[derive(Debug, Copy, Clone)]
/// # Error.
pub(super) enum CheckSameError {
	Argue(ArgyleError),
	NoFiles,
	Noop,
	Reset,
	Tmp,
	Write,
}

impl AsRef<str> for CheckSameError {
	#[inline]
	fn as_ref(&self) -> &str { self.as_str() }
}

impl fmt::Display for CheckSameError {
	#[inline]
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.write_str(self.as_str())
	}
}

impl Error for CheckSameError {}

impl From<ArgyleError> for CheckSameError {
	#[inline]
	fn from(err: ArgyleError) -> Self { Self::Argue(err) }
}

impl CheckSameError {
	#[must_use]
	/// # As Str.
	pub(crate) const fn as_str(self) -> &'static str {
		match self {
			Self::Argue(e) => e.as_str(),
			Self::NoFiles => "At least one file path is required.",
			Self::Noop => "",
			Self::Reset => "Unable to reset the cache.",
			Self::Tmp => "Unable to create a temporary cache directory.",
			Self::Write => "Unable to update the cache.",
		}
	}
}
