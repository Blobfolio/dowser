/*!
# Dowser: Dowser
*/

use crate::{
	mutex_ptr,
	NoHashState,
	utility::{
		resolve_dir_entry,
		resolve_path,
		resolve_path_hash,
	},
};
use rayon::iter::{
	ParallelBridge,
	ParallelDrainRange,
	ParallelIterator,
};
use std::{
	collections::HashSet,
	ffi::{
		OsStr,
		OsString,
	},
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
	#[inline]
	fn default() -> Self {
		Self {
			dirs: Vec::new(),
			files: Vec::with_capacity(2048),
			seen: HashSet::with_capacity_and_hasher(2048, NoHashState),
			cb: Box::new(|_: &Path| true),
		}
	}
}

/// # Helper: Impl From Owned `PathBuf` Collections.
macro_rules! impl_from_owned {
	($($ty:ty),+) => ($(
		impl From<$ty> for Dowser {
			fn from(src: $ty) -> Self {
				let mut files = Vec::with_capacity(2048);
				let mut seen = HashSet::with_capacity_and_hasher(2048, NoHashState);

				let dirs = src.into_iter()
					.filter_map(|p| resolve_path(p, false))
					.filter_map(|(h, is_dir, p)|
						if seen.insert(h) {
							if is_dir {
								if let Ok(rd) = fs::read_dir(p) {
									Some(rd)
								}
								else { None }
							}
							else {
								files.push(p);
								None
							}
						}
						else { None }
					)
					.collect();

				Self {
					dirs,
					files,
					seen,
					..Self::default()
				}
			}
		}
	)+);

	($($num:literal),+) => ($(
		impl_from_owned!([PathBuf; $num]);
	)+);
}

impl_from_owned!(Vec<PathBuf>, HashSet<PathBuf>);
impl_from_owned!(
	1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16,
	17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32
);

impl From<PathBuf> for Dowser {
	fn from(src: PathBuf) -> Self {
		let mut dirs = Vec::new();
		let mut files = Vec::with_capacity(2048);
		let mut seen = HashSet::with_capacity_and_hasher(2048, NoHashState);

		if let Some((h, is_dir, p)) = resolve_path(src, false) {
			if seen.insert(h) {
				if is_dir {
					if let Ok(rd) = fs::read_dir(p) {
						dirs.push(rd);
					}
				}
				else {
					files.push(p);
				}
			}
		}

		Self {
			dirs,
			files,
			seen,
			..Self::default()
		}
	}
}

impl From<&PathBuf> for Dowser {
	#[inline]
	fn from(src: &PathBuf) -> Self { Self::from(src.clone()) }
}

impl From<&Path> for Dowser {
	#[inline]
	fn from(src: &Path) -> Self { Self::from(src.to_path_buf()) }
}

/// # Helper: Impl From `AsRef<Path>` Types.
macro_rules! impl_from_as_path {
	($($ty:ty),+) => ($(
		impl From<$ty> for Dowser {
			#[inline]
			fn from(src: $ty) -> Self { Self::from(PathBuf::from(src)) }
		}
	)+);
}

impl_from_as_path!(&str, String, &String, &OsStr, OsString, &OsString);

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
	/// use std::path::PathBuf;
	///
	/// let files = Vec::<PathBuf>::try_from(
	///     Dowser::default().with_path("/my/dir")
	/// ).expect("No files were found.");
	/// ```
	fn try_from(src: Dowser) -> Result<Self, Self::Error> {
		let out = src.into_vec();
		if out.is_empty() { Err(DowserError::NoFiles) }
		else { Ok(out) }
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
	#[inline]
	/// # With Paths.
	///
	/// Append files and/or directories to the finder. File paths will be
	/// checked against the filter callback (if any) and added straight to the
	/// results if they pass (i.e. immediately). Directories will be queued for
	/// later scanning (i.e. during collection).
	///
	/// ## Warning
	///
	/// The source paths are meant to be paths, _plural_. Pass an iterator,
	/// slice, or collection to it, _not_ a singular `Path`/`PathBuf`. If you
	/// want to add a single path, use [`Dowser::with_path`] instead, or
	/// enclose the value in a slice, like `with_paths([val])`.
	///
	/// ## Examples
	///
	/// ```no_run
	/// use dowser::Dowser;
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

	/// # Without Paths.
	///
	/// This will prevent the provided directories or files from being crawled
	/// or included in the output.
	///
	/// ## Examples
	///
	/// ```no_run
	/// use dowser::Dowser;
	/// use std::path::PathBuf;
	///
	/// let files = Vec::<PathBuf>::try_from(
	///     Dowser::default()
	///         .with_path("/my/dir")
	///         .without_paths(&["/my/dir/ignore"])
	/// ).expect("No files were found.");
	/// ```
	pub fn without_paths<P, I>(mut self, paths: I) -> Self
	where
		P: AsRef<Path>,
		I: IntoIterator<Item=P> {
		self.seen.extend(
			paths.into_iter()
				.filter_map(|p| resolve_path_hash(PathBuf::from(p.as_ref()), false))
		);

		self
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

	/// # Without Path.
	///
	/// This will prevent the provided directory or file from being crawled or
	/// included in the output.
	///
	/// ## Examples
	///
	/// ```no_run
	/// use dowser::Dowser;
	/// use std::path::PathBuf;
	///
	/// let files = Vec::<PathBuf>::try_from(
	///     Dowser::default()
	///         .with_path("/my/dir")
	///         .without_path("/my/dir/ignore")
	/// ).expect("No files were found.");
	/// ```
	pub fn without_path<P>(mut self, path: P) -> Self
	where P: AsRef<Path> {
		if let Some(h) = resolve_path_hash(PathBuf::from(path.as_ref()), false) {
			self.seen.insert(h);
		}
		self
	}
}

/// # Building.
impl Dowser {
	#[must_use]
	/// # Into Vec.
	///
	/// Run the search and return a vector of file paths, if any.
	///
	/// If you want to ensure there _are_ files found, use
	/// `Vec::<PathBuf>::try_from` instead. That does the same thing, but only
	/// returns a vec if it is non-empty.
	///
	/// ## Examples
	///
	/// ```no_run
	/// use dowser::Dowser;
	/// use std::path::PathBuf;
	///
	/// let files = Dowser::default().with_path("/my/dir").into_vec();
	/// ```
	pub fn into_vec(self) -> Vec<PathBuf> {
		// We don't have to do anything!
		if self.dirs.is_empty() {
			return self.files;
		}

		// Break up the data.
		let Dowser { mut dirs, files, seen, cb } = self;
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
		Arc::<Mutex<Vec<PathBuf>>>::try_unwrap(files)
			.ok()
			.and_then(|x| x.into_inner().ok())
			.unwrap_or_default()
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

	#[test]
	fn t_from() {
		fn compare_vecs(a: &[PathBuf], b: &[PathBuf]) -> bool {
			a.len() == b.len() &&
			a.iter().all(|x| b.contains(x))
		}

		let normal = Dowser::default()
			.with_path(PathBuf::from("tests/"))
			.into_vec();

		assert!(! normal.is_empty());

		let from = Dowser::from("tests").into_vec();
		assert!(compare_vecs(&normal, &from));

		let from = Dowser::from(PathBuf::from("tests/")).into_vec();
		assert!(compare_vecs(&normal, &from));

		let from = Dowser::from(vec![PathBuf::from("tests/")]).into_vec();
		assert!(compare_vecs(&normal, &from));
	}
}
