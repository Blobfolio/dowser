/*!
# Dowser: Dowser
*/

use ahash::AHashSet;
use crate::{
	AHASH_STATE,
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
	convert::TryFrom,
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



#[allow(missing_debug_implementations)]
/// `Dowser` is a very simple recursive file finder. Directories are read in
/// parallel. Symlinks are followed. Hidden files and directories are read like
/// any other. Matching files are canonicalized, deduped, and returned.
///
/// ## Filtering
///
/// Results can be filtered prior to being yielded with the use of either
/// [`with_filter()`](Dowser::with_filter) — specifying a custom callback method
/// — or [`with_regex()`](Dowser::with_regex) — to match against a (byte)
/// pattern. (The latter requires the `regexp` crate feature be enabled.)
///
/// It is important to define the filter *before* adding any paths, because if
/// those paths are files, they'll need to be filtered. Right? Right.
///
/// Filter callbacks should accept a `&Path` and return `true` to keep it,
/// `false` to discard it. Ultimately, they get stored in the struct with the
/// following type:
///
/// ```ignore
/// Box<dyn Fn(&Path) -> bool + 'static + Send + Sync>
/// ```
///
/// ## Examples
///
/// ```no_run
/// use dowser::Dowser;
/// use std::os::unix::ffi::OsStrExt;
/// use std::path::PathBuf;
///
/// // Return all files under "/usr/share/man".
/// let res: Vec<PathBuf> = Dowser::default()
///     .with_path("/usr/share/man")
///     .build();
///
/// // Return only Gzipped files.
/// let res: Vec<PathBuf> = Dowser::default()
///     .with_regex(r"(?i)[^/]+\.gz$")
///     .with_path("/usr/share/man")
///     .build();
///
/// // The same thing, done manually.
/// let res: Vec<PathBuf> = Dowser::default()
///     .with_filter(|p: &Path| p.extension()
///         .map_or(
///             false,
///             |e| e.as_bytes().eq_ignore_ascii_case(b"gz")
///         )
///     )
///     .with_path("/usr/share/man")
///     .build();
/// ```
pub struct Dowser {
	/// Directories to scan.
	dirs: Vec<ReadDir>,
	/// Files found.
	files: Vec<PathBuf>,
	/// Unique path hashes (to prevent duplicate scans, results).
	seen: AHashSet<u128>,
	/// Filter callback.
	cb: Box<dyn Fn(&Path) -> bool + 'static + Send + Sync>,
}

impl Default for Dowser {
	fn default() -> Self {
		Self {
			dirs: Vec::new(),
			files: Vec::with_capacity(2048),
			seen: AHashSet::with_capacity_and_hasher(2048, AHASH_STATE),
			cb: Box::new(|_: &Path| true),
		}
	}
}

impl<I, P> From<I> for Dowser
where P: AsRef<Path>, I: IntoIterator<Item=P> {
	/// # From Paths.
	///
	/// This should only be used in cases where all paths are directories, or
	/// no file-filtering is going to take place. Otherwise, you should start
	/// with a [`Dowser::default`], add your filter, *then* add the paths.
	fn from(src: I) -> Self {
		Self::default().with_paths(src)
	}
}

impl TryFrom<Dowser> for Vec<PathBuf> {
	type Error = bool;

	/// # Build Non-Empty.
	///
	/// As an alternative to [`Dowser::build`], you can use this method, which
	/// will produce an error — always `false` — in cases where no files were
	/// found.
	fn try_from(src: Dowser) -> Result<Self, Self::Error> {
		let out = src.build();
		if out.is_empty() { Err(false) }
		else { Ok(out) }
	}
}

impl Dowser {
	/// # With Callback.
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
	/// let files = Dowser::default()
	///     .with_filter(|p: &Path| { ... })
	///     .with_path("/my/dir")
	///     .build();
	/// ```
	pub fn with_filter<F>(mut self, cb: F) -> Self
	where F: Fn(&Path) -> bool + 'static + Send + Sync {
		self.cb = Box::new(cb);
		self
	}

	#[cfg(feature = "regexp")]
	/// # With a Regex Callback.
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
	/// ## Examples
	///
	/// ```no_run
	/// use dowser::Dowser;
	///
	/// let files = Dowser::default()
	///     .with_regex(r"(?i).+\.jpe?g$")
	///     .with_path("/my/dir")
	///     .build();
	/// ```
	pub fn with_regex<R>(mut self, reg: R) -> Self
	where R: std::borrow::Borrow<str> {
		use regex::bytes::Regex;
		let pat: Regex = Regex::new(reg.borrow()).expect("Invalid Regex.");
		self.cb = Box::new(move|p: &Path| pat.is_match(crate::utility::path_as_bytes(p)));
		self
	}

	/// # With Paths.
	///
	/// Append files and/or directories to the finder. File paths will be
	/// checked against the filter callback (if any) and added straight to the
	/// results if they pass (i.e. immediately). Directories will be queued for
	/// later scanning (i.e. when you call [`Dowser::build`]).
	///
	/// ## Examples
	///
	/// ```no_run
	/// use dowser::Dowser;
	///
	/// let files = Dowser::default()
	///     .with_paths(&["/my/dir"])
	///     .build();
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
	/// If it is a directory, it will be queued for later scanning.
	///
	/// ## Examples
	///
	/// ```no_run
	/// use dowser::Dowser;
	///
	/// let files = Dowser::default()
	///     .with_path("/my/dir")
	///     .build();
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

	#[must_use]
	/// # Build!
	///
	/// Once everything is set up, call this method to consume the queue and
	/// collect the files into a `Vec<PathBuf>`.
	///
	/// ## Examples
	///
	/// ```no_run
	/// use dowser::Dowser;
	///
	/// let files = Dowser::default()
	///     .with_path("/my/dir")
	///     .build();
	/// ```
	pub fn build(self) -> Vec<PathBuf> {
		// We don't have to do anything!
		if self.dirs.is_empty() {
			return self.files;
		}

		// Break up the data.
		let Self { mut dirs, files, seen, cb } = self;
		let seen = Arc::from(Mutex::new(seen));
		let files = Arc::from(Mutex::new(files));

		// Process until we're our of directories.
		loop {
			dirs = dirs.par_drain(..)
				.flat_map(ParallelBridge::par_bridge)
				.filter_map(resolve_dir_entry)
				.filter_map(|(h, is_dir, p)|
					if crate::mutex_ptr!(seen).insert(h) {
						if is_dir { fs::read_dir(p).ok() }
						else {
							if cb(&p) { crate::mutex_ptr!(files).push(p); }
							None
						}
					}
					else { None }
				)
				.collect();

			if dirs.is_empty() { break; }
		}

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
		let mut w1 = Dowser::default()
			.with_path(PathBuf::from("tests/"))
			.build();
		assert!(! w1.is_empty());
		assert_eq!(w1.len(), 9);
		assert!(w1.contains(&abs_p1));
		assert!(w1.contains(&abs_p2));
		assert!(! w1.contains(&abs_perr));

		#[cfg(feature = "regexp")]
		{
			// Look only for .txt files.
			w1 = Dowser::default()
				.with_regex(r"(?i)\.txt$")
				.with_paths(&[PathBuf::from("tests/")])
				.build();
			assert!(! w1.is_empty());
			assert_eq!(w1.len(), 1);
			assert!(w1.contains(&abs_p1));
			assert!(! w1.contains(&abs_p2));
			assert!(! w1.contains(&abs_perr));

			// Look for something that doesn't exist.
			w1 = Dowser::default()
				.with_regex(r"(?i)\.exe$")
				.with_path(PathBuf::from("tests/"))
				.build();
			assert!(w1.is_empty());
			assert_eq!(w1.len(), 0);
			assert!(! w1.contains(&abs_p1));
			assert!(! w1.contains(&abs_p2));
			assert!(! w1.contains(&abs_perr));
		}

		// One Extension.
		w1 = Dowser::default()
			.with_path(PathBuf::from("tests/"))
			.with_filter(|p: &Path| p.extension()
				.map_or(
					false,
					|e| e.as_bytes().eq_ignore_ascii_case(b"txt")
				)
			)
			.build();
		assert!(! w1.is_empty());
		assert_eq!(w1.len(), 1);
	}
}
