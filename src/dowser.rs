/*!
# Dowser: Dowser
*/

use crate::{
	NoHashState,
	NoHashU64,
};

#[cfg(feature = "parking_lot_mutex")]
use parking_lot::Mutex;

use rayon::iter::{
	IntoParallelIterator,
	ParallelBridge,
	ParallelDrainRange,
	ParallelExtend,
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
		DirEntry,
		ReadDir,
	},
	os::unix::fs::MetadataExt,
	path::{
		Path,
		PathBuf,
	},
	sync::Arc,
};

#[cfg(not(feature = "parking_lot_mutex"))]
use std::sync::Mutex;



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
/// ## Ulimit
///
/// Multi-threaded filesystem crawling is great and all, but can run into
/// problems when there are a ton of files and the user executing the search
/// has a low `ulimit` configured for their account.
///
/// If you are seeing inconsistent search results — different totals from run
/// to run — you are likely hitting just such a limit.
///
/// In such cases, you should either increase your `ulimit` — refer to your OS
/// instructions for that — or use [`Dowser::into_vec_serial`], which executes
/// a single-threaded crawl, mooting the issue entirely.
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
///
/// If compiled with the `regexp` flag, you can additionally filter by regular
/// expression:
///
/// ```no_run,ignore
/// use dowser::Dowser;
/// use std::path::PathBuf;
///
/// // Return only Gzipped files using regular expression.
/// let files = Vec::<PathBuf>::try_from(
///     Dowser::regex(r"(?i)[^/]+\.gz$").with_path("/usr/share/man")
/// ).expect("No files were found.");
/// ```
///
/// Note: If you want a vector back no matter what — even if empty — you can
/// use [`Dowser::into_vec`] instead of `TryFrom::<PathBuf>`.
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
								if let Ok(rd) = fs::read_dir(p) { Some(rd) }
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

/// # Helper: Impl From `AsRef<Path>` Types.
macro_rules! impl_from_path {
	($($cast:ident $ty:ty),+) => ($(
		impl From<$ty> for Dowser {
			#[inline]
			fn from(src: $ty) -> Self { Self::from(src.$cast()) }
		}
	)+);

	($($ty:ty),+) => ($(
		impl From<$ty> for Dowser {
			#[inline]
			fn from(src: $ty) -> Self { Self::from(PathBuf::from(src)) }
		}
	)+);
}

impl_from_path!(clone &PathBuf, to_path_buf &Path);
impl_from_path!(&str, String, &String, &OsStr, OsString, &OsString);

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
	#[must_use]
	/// # With Capacity.
	///
	/// Create a default [`Dowser`] using the estimated node counts. The closer
	/// this value is to the total number of files and directories the scan
	/// will turn up, the fewer resize allocations there should be.
	///
	/// For reference, the default value is `2048`.
	///
	/// ## Examples
	///
	/// ```no_run
	/// use dowser::Dowser;
	///
	/// let files = Dowser::with_capacity(128)
	///     .with_path("/my/dir")
	///     .into_vec();
	/// ```
	pub fn with_capacity(len: usize) -> Self {
		Self {
			dirs: Vec::new(),
			files: Vec::with_capacity(len),
			seen: HashSet::with_capacity_and_hasher(len, NoHashState),
			cb: Box::new(|_: &Path| true),
		}
	}

	#[must_use]
	/// # With Capacity and Filter.
	///
	/// Create a filtered [`Dowser`] using the estimated node counts. The
	/// closer this value is to the total number of files and directories the
	/// scan will turn up, the fewer resize allocations there should be.
	///
	/// For reference, the default value is `2048`.
	///
	/// ## Examples
	///
	/// ```no_run
	/// use dowser::Dowser;
	/// use std::path::Path;
	///
	/// let files = Dowser::with_capacity_and_filter(128, |p: &Path| { true })
	///     .with_path("/my/dir")
	///     .into_vec();
	/// ```
	pub fn with_capacity_and_filter<F>(len: usize, cb: F) -> Self
	where F: Fn(&Path) -> bool + 'static + Send + Sync {
		Self {
			dirs: Vec::new(),
			files: Vec::with_capacity(len),
			seen: HashSet::with_capacity_and_hasher(len, NoHashState),
			cb: Box::new(cb),
		}
	}

	#[inline]
	#[must_use]
	/// # Filtered via Callback.
	///
	/// Define a custom filter callback to determine whether or not a given
	/// file path should be yielded. Return `true` to keep it, `false` to
	/// reject it.
	///
	/// ## Examples
	///
	/// ```no_run
	/// use dowser::Dowser;
	/// use std::path::{Path, PathBuf};
	///
	/// let files = Vec::<PathBuf>::try_from(
	///     Dowser::filtered(|p: &Path| { true }).with_path("/my/dir")
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
	#[must_use]
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
	#[must_use]
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

	#[must_use]
	/// # Without Paths.
	///
	/// This will prevent the provided directories or files from being crawled
	/// or included in the output.
	///
	/// Note: without-paths should be specified before with-paths, just in case
	/// the sets overlap.
	///
	/// ## Examples
	///
	/// ```no_run
	/// use dowser::Dowser;
	///
	/// let files = Dowser::default()
	///     .without_paths(&["/my/dir/ignore"])
	///     .with_path("/my/dir")
	///     .into_vec();
	/// ```
	pub fn without_paths<P, I>(mut self, paths: I) -> Self
	where
		P: AsRef<Path>,
		I: IntoIterator<Item=P> {
		self.seen.extend(
			paths.into_iter()
				.filter_map(|p| resolve_path_hash(p.as_ref()))
		);

		self
	}

	#[must_use]
	/// # Without Paths (Parallel).
	///
	/// This is a multi-threaded version of [`Dowser::without_paths`]. If you
	/// know your list is small, the sequential version will probably be
	/// faster.
	pub fn par_without_paths<P, I>(mut self, paths: I) -> Self
	where
		P: AsRef<Path>,
		I: IntoParallelIterator<Item=P> {
		self.seen.par_extend(
			paths.into_par_iter()
				.filter_map(|p| resolve_path_hash(p.as_ref()))
		);

		self
	}

	#[must_use]
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

	#[must_use]
	/// # Without Path.
	///
	/// This will prevent the provided directory or file from being crawled or
	/// included in the output.
	///
	/// Note: without-paths should be specified before with-paths, just in case
	/// the sets overlap.
	///
	/// ## Examples
	///
	/// ```no_run
	/// use dowser::Dowser;
	///
	/// let files = Dowser::default()
	///     .without_path("/my/dir/ignore")
	///     .with_path("/my/dir")
	///     .into_vec();
	/// ```
	pub fn without_path<P>(mut self, path: P) -> Self
	where P: AsRef<Path> {
		if let Some(h) = resolve_path_hash(path.as_ref()) {
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
	///
	/// let files = Dowser::default().with_path("/my/dir").into_vec();
	/// ```
	pub fn into_vec(self) -> Vec<PathBuf> {
		// Break up the data.
		let Dowser { mut dirs, mut files, seen, cb } = self;
		let seen = Arc::from(Mutex::new(seen));

		// Process until we're our of directories.
		while ! dirs.is_empty() {
			let (tx, rx) = crossbeam_channel::unbounded();

			files.par_extend(
				dirs.par_drain(..)
					.flat_map(ParallelBridge::par_bridge)
					.filter_map(|e| resolve_dir_entry(e, &seen))
					.filter_map(|(is_dir, p)|
						if is_dir {
							if let Ok(rd) = fs::read_dir(p) {
								// No need to panic; if for some reason this
								// fails we just won't read the directory.
								let _res = tx.send(rd);
							}
							None
						}
						else if cb(&p) { Some(p) }
						else { None }
					)
			);

			drop(tx);
			dirs.extend(rx);
		}

		// Unwrap and return.
		files
	}

	#[must_use]
	/// # Into Vec (serial).
	///
	/// Multi-threaded crawling is great, but can run into issues with `ulimit`
	/// caps and similar, leading to inconsistent results when there are a lot
	/// of them.
	///
	/// This version processes everything in serial, hopefully negating such
	/// issues.
	pub fn into_vec_serial(self) -> Vec<PathBuf> {
		let Dowser { mut dirs, mut files, mut seen, cb } = self;
		while let Some(rd) = dirs.pop() {
			for e in rd.filter_map(Result::ok) {
				if let Some((h, is_dir, path)) = resolve_path(e.path(), true) {
					if seen.insert(h) {
						if is_dir {
							if let Ok(rd) = std::fs::read_dir(path) {
								dirs.push(rd);
							}
						}
						else if cb(&path) {
							files.push(path);
						}
					}
				}
			}
		}

		files
	}


	#[must_use]
	/// # Shallow Search.
	///
	/// This works like [`Dowser::into_vec`] but without the recursion; only
	/// the top-level files within the specified roots will be tested and
	/// returned.
	///
	/// If `include_dirs` is true, sub-directory paths will also be tested and
	/// returned.
	///
	/// ## Examples
	///
	/// ```no_run
	/// use dowser::Dowser;
	///
	/// let files = Dowser::default()
	///     .with_path("/my/dir")
	///     .shallow(false);
	///
	/// let files_and_dirs = Dowser::default()
	///     .with_path("/my/dir")
	///     .shallow(true);
	/// ```
	pub fn shallow(self, include_dirs: bool) -> Vec<PathBuf> {
		// Easy abort!
		if self.dirs.is_empty() {
			return self.files;
		}

		// Break up the data.
		let Dowser { mut dirs, mut files, seen, cb } = self;
		let seen = Arc::from(Mutex::new(seen));

		// Process until we're our of directories.
		files.par_extend(
			dirs.par_drain(..)
				.flat_map(ParallelBridge::par_bridge)
				.filter_map(|e| resolve_dir_entry(e, &seen))
				.filter_map(|(is_dir, p)|
					if (include_dirs || ! is_dir) && cb(&p) { Some(p) }
					else { None }
				)
		);

		// Unwrap and return.
		files
	}
}



#[inline]
/// # Resolve `DirEntry`.
///
/// This is a convenience callback for [`Dowser`] used during `ReadDir`
/// traversal.
///
/// See [`resolve_path`] for more information.
fn resolve_dir_entry(
	entry: Result<DirEntry, std::io::Error>,
	seen: &Arc<Mutex<HashSet<u64, NoHashState>>>
) -> Option<(bool, PathBuf)> {
	let entry = entry.ok()?;
	let (h, is_dir, path) = resolve_path(entry.path(), true)?;

	#[cfg(feature = "parking_lot_mutex")]
	let mut ptr = seen.lock();

	#[cfg(not(feature = "parking_lot_mutex"))]
	let mut ptr = seen.lock().unwrap_or_else(std::sync::PoisonError::into_inner);

	if ptr.insert(h) { Some((is_dir, path)) }
	else { None }
}

/// # Resolve Path.
///
/// This attempts to cheaply resolve a given path, returning:
/// * A unique hash derived from the path's device and inode.
/// * A bool indicating whether or not the path is a directory.
/// * The canonicalized path.
///
/// As [`std::fs::canonicalize`] is an expensive operation, this method allows
/// a "trusted" bypass, which will only canonicalize the path if it is a
/// symlink.
///
/// The trusted mode is only appropriate in cases like `ReadDir` where the
/// directory seed was canonicalized. The idea is that since `DirEntry` paths
/// are joined to the seed, they'll be canonical so long as the seed was,
/// except in cases of symlinks.
fn resolve_path(path: PathBuf, trusted: bool) -> Option<(u64, bool, PathBuf)> {
	if trusted {
		let meta = std::fs::symlink_metadata(&path).ok()?;
		if ! meta.file_type().is_symlink() {
			let hash: u64 = NoHashU64::hash_path(meta.dev(), meta.ino());
			return Some((hash, meta.is_dir(), path));
		}
	}

	let path = std::fs::canonicalize(path).ok()?;
	let meta = std::fs::metadata(&path).ok()?;
	let hash: u64 = NoHashU64::hash_path(meta.dev(), meta.ino());
	Some((hash, meta.is_dir(), path))
}

/// # Resolve Path Hash.
///
/// This is identical to `resolve_path`, except it only returns the hash. It
/// is used by [`Dowser::without_paths`] and [`Dowser::without_path`], which
/// don't actually need anything more.
fn resolve_path_hash(path: &Path) -> Option<u64> {
	let path = std::fs::canonicalize(path).ok()?;
	let meta = std::fs::metadata(&path).ok()?;
	let hash: u64 = NoHashU64::hash_path(meta.dev(), meta.ino());
	Some(hash)
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

		// Make sure parallel and serial pull the same results. This is a small
		// set and should work consistently even with heavy caps...
		let mut par = Dowser::default()
			.with_path(PathBuf::from("tests/"))
			.into_vec();
		par.sort();

		let mut serial = Dowser::default()
			.with_path(PathBuf::from("tests/"))
			.into_vec_serial();
		serial.sort();

		assert_eq!(par, serial, "Parallel and serial results differed; is ulimit a problem?");
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

	#[test]
	fn t_resolve_path() {
		let test_dir = std::fs::canonicalize("./tests/links").expect("Missing dowser link directory.");

		let raw = vec![
			test_dir.join("01"),
			test_dir.join("02"),
			test_dir.join("03"),
			test_dir.join("04"),
			test_dir.join("05"), // Directory.
			test_dir.join("06"), // Directory.
			test_dir.join("07"), // Sym to six.
			test_dir.join("06/08"),
			test_dir.join("06/09"),
			test_dir.join("06/10"), // Sym to one.
		];

		let canon = {
			let mut tmp: Vec<PathBuf> = raw.iter()
				.filter_map(|x| std::fs::canonicalize(x).ok())
				.collect();
			tmp.sort();
			tmp.dedup();
			tmp
		};

		// There should be two fewer entries as two are symlinks.
		assert_eq!(raw.len(), 10);
		assert_eq!(canon.len(), 8, "{:?}", canon);
		assert!(! canon.contains(&raw[6]));
		assert!(! canon.contains(&raw[9]));

		let trusting = {
			let mut tmp: Vec<PathBuf> = raw.iter()
				.filter_map(|x| resolve_path(x.clone(), true).map(|(_, _, p)| p))
				.collect();
			tmp.sort();
			tmp.dedup();
			tmp
		};

		assert_eq!(trusting, canon);
	}

	#[test]
	fn t_shallow() {
		let test_dir = std::fs::canonicalize("./tests/links").expect("Missing dowser link directory.");

		let mut raw = vec![
			test_dir.join("01"),
			test_dir.join("02"),
			test_dir.join("03"),
			test_dir.join("04"),
			test_dir.join("05"), // Directory.
			test_dir.join("06"), // Directory.
		];

		let mut found = Dowser::from(&test_dir).shallow(true);
		found.sort();

		assert_eq!(raw, found);

		// Let's test the filter.
		raw.pop();
		found = Dowser::filtered(|p: &Path| ! p.ends_with("06"))
			.with_path(&test_dir)
			.shallow(true);
		found.sort();

		assert_eq!(raw, found);

		// One last time without any directories.
		raw.pop();

		found = Dowser::from(&test_dir).shallow(false);
		found.sort();

		assert_eq!(raw, found);
	}
}
