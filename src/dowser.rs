/*!
# Dowser: Dowser
*/

use crate::{
	mutex_ptr,
	NoHashState,
	utility::{
		resolve_dir_entry,
		resolve_path,
	},
};
use rayon::iter::{
	ParallelBridge,
	ParallelDrainRange,
	ParallelIterator,
};
use std::{
	collections::HashSet,
	convert::TryFrom,
	fmt,
	fs::{
		self,
		ReadDir,
	},
	path::{
		Path,
		PathBuf,
	},
	sync::{
		Arc,
		Mutex,
	},
};



#[derive(Debug, Copy, Clone)]
/// # Error.
pub enum DowserError {
	/// # No files.
	NoFiles,
}

impl fmt::Display for DowserError {
	#[inline]
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.write_str(self.as_str())
	}
}

impl std::error::Error for DowserError {}

impl DowserError {
	#[must_use]
	/// # As Str.
	pub const fn as_str(self) -> &'static str {
		match self {
			Self::NoFiles => "No matching files were found.",
		}
	}
}



#[allow(missing_debug_implementations)]
/// `Dowser` is a very simple recursive file finder. Directories are read in
/// parallel. Symlinks are followed. Hidden files and directories are read like
/// any other. Matching files are canonicalized, deduped, and returned.
///
/// ## Usage
///
/// An instance is initialized using one of the following three methods:
///
/// * [`Dowser::default`]: Return all files without prejudice.
/// * [`Dowser::filtered`]: Filter file paths via the provided callback.
/// * [`Dowser::regex`]: Filter file paths via regular express. (This requires enabling the `regexp` crate feature.)
///
/// From there, add one or more file or directory paths using the [`Dowser::with_path`]
/// and [`Dowser::with_paths`] methods.
///
/// Finally, collect the results with `Vec::<PathBuf>::try_from()`. If no files
/// are found, an error is returned, otherwise the matching file paths are
/// collected into a vector.
///
/// ## Examples
///
/// ```no_run
/// use dowser::Dowser;
/// use std::convert::TryFrom;
/// use std::os::unix::ffi::OsStrExt;
/// use std::path::{Path, PathBuf};
///
/// // Return all files under "/usr/share/man".
/// let files = Vec::<PathBuf>::try_from(
///    Dowser::default().with_path("/usr/share/man")
/// ).expect("No files were found.");
///
/// // Return only Gzipped files using regular expression.
/// let files = Vec::<PathBuf>::try_from(
///     Dowser::regex(r"(?i)[^/]+\.gz$").with_path("/usr/share/man")
/// ).expect("No files were found.");
///
/// // Return only Gzipped files using callback filter.
/// let files = Vec::<PathBuf>::try_from(
///     Dowser::filtered(|p: &Path| p.extension()
///         .map_or(
///             false,
///             |e| e.as_bytes().eq_ignore_ascii_case(b"gz")
///         )
///     )
///     .with_path("/usr/share/man")
/// ).expect("No files were found.");
/// ```
pub struct Dowser {
	/// Directories to scan.
	dirs: Vec<ReadDir>,
	/// Files found.
	files: Vec<PathBuf>,
	/// Unique path hashes (to prevent duplicate scans, results).
	seen: HashSet<u64, NoHashState>,
	/// Filter callback.
	cb: Box<dyn Fn(&Path) -> bool + 'static + Send + Sync>,
}

impl Default for Dowser {
	fn default() -> Self {
		Self {
			dirs: Vec::new(),
			files: Vec::with_capacity(2048),
			seen: HashSet::with_capacity_and_hasher(2048, NoHashState),
			cb: Box::new(|_: &Path| true),
		}
	}
}

impl TryFrom<Dowser> for Vec<PathBuf> {
	type Error = DowserError;

	/// # Build!.
	///
	/// Once everything is set up, call this method to consume the queue and
	/// collect the files into a `Vec<PathBuf>`.
	///
	/// ## Errors
	///
	/// This will return an error if no files are found.
	///
	/// ## Examples
	///
	/// ```no_run
	/// use dowser::Dowser;
	/// use std::convert::TryFrom;
	/// use std::path::PathBuf;
	///
	/// let files = Vec::<PathBuf>::try_from(
	///     Dowser::default().with_path("/my/dir")
	/// ).expect("No files were found.");
	/// ```
	fn try_from(src: Dowser) -> Result<Self, Self::Error> {
		// We don't have to do anything!
		if src.dirs.is_empty() {
			if src.files.is_empty() { return Err(DowserError::NoFiles); }
			return Ok(src.files);
		}

		// Break up the data.
		let Dowser { mut dirs, files, seen, cb } = src;
		let seen = Arc::from(Mutex::new(seen));
		let files = Arc::from(Mutex::new(files));

		// Process until we're our of directories.
		loop {
			dirs = dirs.par_drain(..)
				.flat_map(ParallelBridge::par_bridge)
				.filter_map(resolve_dir_entry)
				.filter_map(|(h, is_dir, p)|
					if mutex_ptr!(seen).insert(h) {
						if is_dir { fs::read_dir(p).ok() }
						else {
							if cb(&p) { mutex_ptr!(files).push(p); }
							None
						}
					}
					else { None }
				)
				.collect();

			if dirs.is_empty() { break; }
		}

		// Unwrap and return.
		Arc::<Mutex<Self>>::try_unwrap(files)
			.ok()
			.and_then(|x| x.into_inner().ok())
			.filter(|x| ! x.is_empty())
			.ok_or(DowserError::NoFiles)
	}
}

/// # Instantiation.
impl Dowser {
	#[inline]
	/// # Filtered via Callback.
	///
	/// Define a custom filter callback to determine whether or not a given
	/// file path should be yielded. Return `true` to keep it, `false` to
	/// reject it.
	///
	/// ## Examples
	///
	/// ```ignore
	/// use dowser::Dowser;
	/// use std::convert::TryFrom;
	/// use std::path::PathBuf;
	///
	/// let files = Vec::<PathBuf>::try_from(
	///     Dowser::filtered(|p: &Path| { ... }).with_path("/my/dir")
	/// ).expect("No files were found.");
	/// ```
	pub fn filtered<F>(cb: F) -> Self
	where F: Fn(&Path) -> bool + 'static + Send + Sync {
		Self {
			cb: Box::new(cb),
			..Self::default()
		}
	}

	#[cfg(feature = "regexp")]
	#[inline]
	/// # Filtered via Regex.
	///
	/// This is a convenience method for filtering files by regular expression.
	/// You supply only the expression, and [`Dowser`] will test it against
	/// the (full) path as a byte string, keeping any matches, rejecting the
	/// rest.
	///
	/// This method is only available when the `regexp` crate feature is
	/// enabled. This pulls down the [`regex`](https://crates.io/crates/regex) crate to handle the details.
	///
	/// Speaking of, see [here](https://docs.rs/regex/1.4.3/regex/#syntax) for syntax reference and other details.
	///
	/// ## Panics
	///
	/// This method will panic if the regular expression is malformed.
	///
	/// ## Examples
	///
	/// ```no_run
	/// use dowser::Dowser;
	/// use std::convert::TryFrom;
	/// use std::path::PathBuf;
	///
	/// let files = Vec::<PathBuf>::try_from(
	///     Dowser::regex(r"(?i)[^/]+\.jpe?g$").with_path("/my/dir")
	/// ).expect("No files were found.");
	/// ```
	pub fn regex<R>(reg: R) -> Self
	where R: std::borrow::Borrow<str> {
		use regex::bytes::Regex;

		let pat: Regex = Regex::new(reg.borrow()).expect("Invalid Regex.");
		Self {
			cb: Box::new(move|p: &Path| pat.is_match(crate::utility::path_as_bytes(p))),
			..Self::default()
		}
	}
}

/// # Adding Path(s).
impl Dowser {
	/// # With Paths.
	///
	/// Append files and/or directories to the finder. File paths will be
	/// checked against the filter callback (if any) and added straight to the
	/// results if they pass (i.e. immediately). Directories will be queued for
	/// later scanning (i.e. during collection).
	///
	/// ## Examples
	///
	/// ```no_run
	/// use dowser::Dowser;
	/// use std::convert::TryFrom;
	/// use std::path::PathBuf;
	///
	/// let files = Vec::<PathBuf>::try_from(
	///     Dowser::default().with_paths(&["/my/dir"])
	/// ).expect("No files were found.");
	/// ```
	pub fn with_paths<P, I>(self, paths: I) -> Self
	where
		P: AsRef<Path>,
		I: IntoIterator<Item=P> {
		paths.into_iter().fold(self, Self::with_path)
	}

	/// # With Path.
	///
	/// Add a path to the finder. If the path is a file, it will be checked
	/// against the filter callback (if any) before being added to the results.
	/// If it is a directory, it will be queued for later scanning (i.e. during
	/// collection).
	///
	/// ## Examples
	///
	/// ```no_run
	/// use dowser::Dowser;
	/// use std::convert::TryFrom;
	/// use std::path::PathBuf;
	///
	/// let files = Vec::<PathBuf>::try_from(
	///     Dowser::default().with_path("/my/dir")
	/// ).expect("No files were found.");
	/// ```
	pub fn with_path<P>(mut self, path: P) -> Self
	where P: AsRef<Path> {
		if let Some((h, is_dir, p)) = resolve_path(PathBuf::from(path.as_ref()), false) {
			if self.seen.insert(h) {
				if is_dir {
					if let Ok(rd) = fs::read_dir(p) {
						self.dirs.push(rd);
					}
				}
				else if (self.cb)(&p) {
					self.files.push(p);
				}
			}
		}

		self
	}
}



#[cfg(test)]
mod tests {
	use super::*;
	use brunch as _;
	use std::os::unix::ffi::OsStrExt;

	#[test]
	fn t_new() {
		let mut abs_dir = fs::canonicalize("tests/assets/").unwrap();
		abs_dir.push("_.txt");
		let abs_p1 = abs_dir.with_file_name("file.txt");
		let abs_p2 = abs_dir.with_file_name("is-executable.sh");
		let abs_perr = abs_dir.with_file_name("foo.bar");

		// Do a non-search search.
		let mut w1 = Vec::<PathBuf>::try_from(
			Dowser::default().with_path(PathBuf::from("tests/"))
		).expect("Missing tests/ directory.");
		assert!(! w1.is_empty());
		assert_eq!(w1.len(), 9);
		assert!(w1.contains(&abs_p1));
		assert!(w1.contains(&abs_p2));
		assert!(! w1.contains(&abs_perr));

		#[cfg(feature = "regexp")]
		{
			// Look only for .txt files.
			w1 = Vec::<PathBuf>::try_from(
				Dowser::regex(r"(?i)\.txt$").with_path(PathBuf::from("tests/"))
			).expect("Missing tests/ directory.");
			assert!(! w1.is_empty());
			assert_eq!(w1.len(), 1);
			assert!(w1.contains(&abs_p1));
			assert!(! w1.contains(&abs_p2));
			assert!(! w1.contains(&abs_perr));

			// Look for something that doesn't exist.
			assert!(
				Vec::<PathBuf>::try_from(
					Dowser::regex(r"(?i)\.exe$").with_path(PathBuf::from("tests/"))
				).is_err()
			);
		}

		// Filtered search.
		w1 = Vec::<PathBuf>::try_from(
			Dowser::filtered(|p: &Path| p.extension()
				.map_or(
					false,
					|e| e.as_bytes().eq_ignore_ascii_case(b"txt")
				)
			)
			.with_path(PathBuf::from("tests/"))
		).expect("Missing /tests directory.");
		assert!(! w1.is_empty());
		assert_eq!(w1.len(), 1);
	}
}
