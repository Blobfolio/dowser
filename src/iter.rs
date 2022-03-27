/*!
# Dowser: Dowser
*/

use crate::{
	DirConcurrency,
	NoHashState,
};

#[cfg(feature = "parking_lot_mutex")]
use parking_lot::Mutex;

use rayon::iter::{
	IntoParallelIterator,
	ParallelBridge,
	ParallelIterator,
};
use std::{
	collections::HashSet,
	ffi::OsStr,
	fs::{
		DirEntry,
		Metadata,
	},
	hash::Hasher,
	os::unix::fs::MetadataExt,
	path::{
		Path,
		PathBuf,
	},
};

#[cfg(not(feature = "parking_lot_mutex"))]
use std::sync::Mutex;



#[cfg(feature = "parking_lot_mutex")]
/// # Helper: Unlock Mutex.
macro_rules! mutex { ($var:expr) => ($var.lock()); }

#[cfg(not(feature = "parking_lot_mutex"))]
/// # Helper: Unlock Mutex.
macro_rules! mutex { ($var:expr) => ($var.lock().unwrap_or_else(std::sync::PoisonError::into_inner)); }



#[derive(Debug, Clone)]
/// # Dowser.
///
/// `Dowser` is a very simple recursive file iterator with parallelized
/// crawling for performance. Symlinks and hidden nodes are followed like any
/// other, and all results are canonicalized and deduped prior to yielding.
///
/// ## Usage
///
/// All you need to do is chain [`Dowser::default`] with one or more of the
/// following seed methods:
///
/// * [`Dowser::with_path`] / [`Dowser::with_paths`]
/// * [`Dowser::without_path`] / [`Dowser::without_paths`]
///
/// The `with_*` methods add sources to be crawled, while the `without_*`
/// methods shitlist sources, preventing them from being yielded by the
/// iterator.
///
/// If using `without_*`, be sure to chain those _first_, before any `with_*`
/// calls, just in case your withs and withouts overlap. ;)
///
/// From there, just do your normal iterator business.
///
/// ## Gotchas
///
/// Because `Dowser` internally implements multi-threading, you should not try
/// to do something like `Dowser::default().par_bridge()`; that will just make
/// everything slower.
///
/// `Dowser` leaves some threads in reserve to help mitigate system caps like
/// `ulimit`, but if the user running the program has a very low `ulimit` set,
/// the results may be inconsistent from run to run. In such cases, please
/// refer to your operating system's instructions for increasing the limit.
///
/// ## Examples
///
/// ```no_run
/// use dowser::Dowser;
/// use std::path::PathBuf;
///
/// let files: Vec<PathBuf> = Dowser::default()
///     .with_path("/usr/share")
///     // You could filter_map(), etc., here. All paths returned are canonical,
///     // valid files.
///     .collect();
/// ```
pub struct Dowser {
	files: Vec<PathBuf>,
	dirs: Vec<PathBuf>,
	dir_concurrency: usize,
	seen: HashSet<u64, NoHashState>,
}

impl Default for Dowser {
	fn default() -> Self {
		Self {
			files: Vec::with_capacity(8),
			dirs: Vec::with_capacity(8),
			dir_concurrency: usize::from(DirConcurrency::Sane),
			seen: HashSet::with_capacity_and_hasher(4096, NoHashState::default()),
		}
	}
}

macro_rules! from_single {
	($($ty:ty),+ $(,)?) => ($(
		impl From<$ty> for Dowser {
			#[inline]
			fn from(src: $ty) -> Self { Self::default().with_path(src) }
		}
	)+);
}

from_single!(&OsStr, &Path, PathBuf, &PathBuf, &str, String, &String);

impl From<&[PathBuf]> for Dowser {
	fn from(src: &[PathBuf]) -> Self {
		let mut out = Self::default();

		for e in src.iter().filter_map(Entry::from_path) {
			if out.seen.insert(e.hash) {
				if e.is_dir { out.dirs.push(e.path); }
				else { out.files.push(e.path); }
			}
		}

		out
	}
}

impl From<Vec<PathBuf>> for Dowser {
	fn from(src: Vec<PathBuf>) -> Self {
		let mut out = Self::default();

		for e in src.into_iter().filter_map(Entry::from_path) {
			if out.seen.insert(e.hash) {
				if e.is_dir { out.dirs.push(e.path); }
				else { out.files.push(e.path); }
			}
		}

		out
	}
}

impl Iterator for Dowser {
	type Item = PathBuf;

	/// # Next!
	///
	/// This iterator yields canonical, deduplicated _file_ paths. Directories
	/// are recursively traversed, but their paths are not returned.
	///
	/// Item ordering is arbitrary and likely to change from run-to-run, but
	/// unless you hit a `ulimit`-type ceiling (see [`DirConcurrency`]), the
	/// same items should always get returned.
	fn next(&mut self) -> Option<Self::Item> {
		loop {
			// We have a file ready to go!
			if let Some(p) = self.files.pop() {
				return Some(p);
			}

			// Are we out of things to do?
			let len = self.dirs.len();
			if len == 0 { break; }

			// Read one directory in serial.
			if self.dir_concurrency == 1 {
				if let Ok(rd) = std::fs::read_dir(self.dirs.remove(len - 1)) {
					for e in rd.filter_map(Entry::from_entry) {
						if self.seen.insert(e.hash) {
							if e.is_dir { self.dirs.push(e.path); }
							else { self.files.push(e.path); }
						}
					}
				}
			}
			// Read one or more directories in parallel.
			else {
				let new = self.dirs.split_off(len.saturating_sub(self.dir_concurrency));
				let s = Mutex::new(&mut self.seen);
				let f = Mutex::new(&mut self.files);
				let d = Mutex::new(&mut self.dirs);

				new.into_par_iter()
					.filter_map(|p| std::fs::read_dir(p).ok())
					.flat_map(ParallelBridge::par_bridge)
					.for_each(|e| if let Some(e) = Entry::from_entry(e) {
						if mutex!(s).insert(e.hash) {
							if e.is_dir { mutex!(d).push(e.path); }
							else { mutex!(f).push(e.path); }
						}
					});
			}
		}

		None
	}

	/// # Size Hints.
	///
	/// This iterator has an unknown size until the final directory has been
	/// read, after which point it is just a matter of flushing the files it
	/// found there.
	fn size_hint(&self) -> (usize, Option<usize>) {
		let lower = self.files.len();
		let upper =
			if self.dirs.is_empty() { Some(lower) }
			else { None };

		(lower, upper)
	}
}

impl Dowser {
	#[inline]
	#[must_use]
	/// # With Paths.
	///
	/// Queue up multiple file and/or directory paths.
	///
	/// ## Warning
	///
	/// **Do not** pass a single `Path` or `PathBuf` to this method. If you
	/// need to add just one path, use [`Dowser::with_path`] instead.
	///
	/// ## Examples
	///
	/// ```no_run
	/// use dowser::Dowser;
	/// use std::path::PathBuf;
	///
	/// let files: Vec::<PathBuf> = Dowser::default()
	///     .with_paths(&["/my/dir"])
	///     .collect();
	/// ```
	pub fn with_paths<P, I>(self, paths: I) -> Self
	where P: AsRef<Path>, I: IntoIterator<Item=P> {
		paths.into_iter().fold(self, Self::with_path)
	}

	#[must_use]
	/// # With Path.
	///
	/// Queue up a single file or directory path.
	///
	/// This can be called multiple times, but [`Dowser::with_paths`] probably
	/// makes more sense when you want to crawl multiple roots.
	///
	/// ## Examples
	///
	/// ```no_run
	/// use dowser::Dowser;
	/// use std::path::PathBuf;
	///
	/// let files: Vec::<PathBuf> = Dowser::default()
	///     .with_path("/my/dir")
	///     .collect();
	/// ```
	pub fn with_path<P>(mut self, path: P) -> Self
	where P: AsRef<Path> {
		if let Some(e) = Entry::from_path(path) {
			if self.seen.insert(e.hash) {
				if e.is_dir { self.dirs.push(e.path); }
				else { self.files.push(e.path); }
			}
		}

		self
	}

	#[must_use]
	/// # With Directory Concurrency.
	///
	/// By default, [`Dowser`] processes directories in parallel, but only so
	/// many at a time. This tends to provide a small performance boost for
	/// most searches, but may not always be the best strategy.
	///
	/// See [`DirConcurrency`] for more information.
	pub fn with_dir_concurrency(mut self, val: DirConcurrency) -> Self {
		self.dir_concurrency = usize::from(val);
		self
	}
}

impl Dowser {
	#[must_use]
	/// # Without Path.
	///
	/// This will prevent the provided directory or file from being crawled or
	/// included in the output.
	///
	/// Note: without-path(s) should be specified before with-path(s), just in
	/// case the sets overlap.
	///
	/// ## Examples
	///
	/// ```no_run
	/// use dowser::Dowser;
	/// use std::path::PathBuf;
	///
	/// let files: Vec<PathBuf> = Dowser::default()
	///     .without_path("/my/dir/ignore")
	///     .with_path("/my/dir")
	///     .collect();
	/// ```
	pub fn without_path<P>(mut self, path: P) -> Self
	where P: AsRef<Path> {
		if let Some(h) = Entry::hash_path(path) {
			self.seen.insert(h);
		}

		self
	}

	#[must_use]
	/// # Without Paths.
	///
	/// This will prevent the provided directories or files from being crawled
	/// or included in the output.
	///
	/// Note: without-path(s) should be specified before with-path(s), just in
	/// case the sets overlap.
	///
	/// ## Examples
	///
	/// ```no_run
	/// use dowser::Dowser;
	/// use std::path::PathBuf;
	///
	/// let files: Vec<PathBuf> = Dowser::default()
	///     .without_paths(&["/my/dir/ignore"])
	///     .with_path("/my/dir")
	///     .collect();
	/// ```
	pub fn without_paths<P, I>(mut self, paths: I) -> Self
	where P: AsRef<Path>, I: IntoIterator<Item=P> {
		self.seen.extend(paths.into_iter().filter_map(Entry::hash_path));
		self
	}
}

impl Dowser {
	#[must_use]
	/// # Consume Into Vec (Filtered).
	///
	/// This method is an optimized alternative to running
	/// `Dowser.iter().filter(…).collect::<Vec<PathBuf>>()`.
	///
	/// It yields the same results as the above, but makes fewer allocations
	/// along the way and applies your filter callback in parallel (unless
	/// [`DirConcurrency::Single`] was set).
	///
	/// ## Examples
	///
	/// ```no_run
	/// use dowser::{Dowser, Extension};
	/// use std::path::PathBuf;
	///
	/// const GZ: Extension = Extension::new2(*b"gz");
	///
	/// // The iterator way.
	/// let files: Vec<PathBuf> = Dowser::default()
	///     .with_path("/usr/share")
	///     .filter(|p| Some(GZ) == Extension::try_from2(p))
	///     .collect();
	///
	/// // The optimized way.
	/// let files: Vec<PathBuf> = Dowser::default()
	///     .with_path("/usr/share")
	///     .into_vec(|p| Some(GZ) == Extension::try_from2(p));
	/// ```
	pub fn into_vec<F>(self, cb: F) -> Vec<PathBuf>
	where F: Fn(&Path) -> bool + Sync + Send {
		let Self { mut files, mut dirs, dir_concurrency, mut seen } = self;

		// We wouldn't have had a chance to filter these yet.
		if ! files.is_empty() {
			files.retain(|p| cb(p));
		}

		// Consume the queue serially.
		if dir_concurrency == 1 {
			while let Some(p) = dirs.pop() {
				if let Ok(rd) = std::fs::read_dir(p) {
					for e in rd.filter_map(Entry::from_entry) {
						if seen.insert(e.hash) {
							if e.is_dir { dirs.push(e.path); }
							else if cb(&e.path) { files.push(e.path); }
						}
					}
				}
			}
		}
		// Consume the queue in parallel.
		else {
			let s = Mutex::new(&mut seen);
			let f = Mutex::new(&mut files);

			loop {
				let len = dirs.len();
				if len == 0 { break; }

				let new = dirs.split_off(len.saturating_sub(dir_concurrency));
				let d = Mutex::new(&mut dirs);

				new.into_par_iter()
					.filter_map(|p| std::fs::read_dir(p).ok())
					.flat_map(ParallelBridge::par_bridge)
					.for_each(|e| if let Some(e) = Entry::from_entry(e) {
						if mutex!(s).insert(e.hash) {
							if e.is_dir { mutex!(d).push(e.path); }
							else if cb(&e.path) { mutex!(f).push(e.path); }
						}
					});
			}
		}

		// Done!
		files
	}
}



/// # File Entry.
///
/// This holds a pre-computed hash, whether or not the path points to a
/// directory, and the canonicalized path itself.
struct Entry {
	path: PathBuf,
	is_dir: bool,
	hash: u64,
}

impl Entry {
	#[must_use]
	/// # From Entry (Result).
	///
	/// Because [`Dowser`] canonicalizes all seed paths, we can assume that
	/// any non-symlinked `DirEntry` is also canonical, thus avoiding expensive
	/// syscalls. (If it is, we'll canonicalize it first.)
	fn from_entry(e: Result<DirEntry, std::io::Error>) -> Option<Self> {
		// If this is a symlink, we have to follow it.
		let e = e.ok()?;
		if e.file_type().map_or(true, |ft| ft.is_symlink()) {
			return Self::from_path(e.path());
		}

		let meta = e.metadata().ok()?;

		Some(Self {
			path: e.path(),
			is_dir: meta.is_dir(),
			hash: Self::hash_meta(&meta),
		})
	}

	#[must_use]
	/// # From Path.
	///
	/// Paths sent to this method are untrusted and forced through
	/// canonicalization before any metadata is worked out.
	fn from_path<P>(path: P) -> Option<Self>
	where P: AsRef<Path> {
		let path = std::fs::canonicalize(path).ok()?;
		let meta = std::fs::metadata(&path).ok()?;

		Some(Self {
			path,
			is_dir: meta.is_dir(),
			hash: Self::hash_meta(&meta),
		})
	}

	#[must_use]
	/// # Hash Meta.
	///
	/// On Unix systems, file uniqueness means a unique device/inode
	/// combination.
	fn hash_meta(meta: &Metadata) -> u64 {
		let mut hasher = ahash::AHasher::new_with_keys(1319, 2371);
		hasher.write_u64(meta.dev());
		hasher.write_u64(meta.ino());
		hasher.finish()
	}

	#[must_use]
	/// # Hash Path.
	///
	/// This returns an appropriate hash for a given path. It is primarily used
	/// in cases where the rest of the `Entry` data is not needed.
	fn hash_path<P>(path: P) -> Option<u64>
	where P: AsRef<Path> {
		let path = path.as_ref();

		if let Ok(meta) = std::fs::symlink_metadata(path) {
			if ! meta.is_symlink() {
				return Some(Self::hash_meta(&meta));
			}
		}

		std::fs::canonicalize(path)
			.and_then(std::fs::metadata)
			.ok()
			.map(|m| Self::hash_meta(&m))
	}
}



#[cfg(test)]
mod tests {
	use super::*;
	use brunch as _;

	#[test]
	fn t_new() {
		let mut abs_dir = std::fs::canonicalize("tests/assets/").unwrap();
		abs_dir.push("_.txt");
		let abs_p1 = abs_dir.with_file_name("file.txt");
		let abs_p2 = abs_dir.with_file_name("is-executable.sh");
		let abs_perr = abs_dir.with_file_name("foo.bar");

		// Builder init.
		let mut w1: Vec<PathBuf> = Dowser::default()
			.with_path(PathBuf::from("tests/"))
			.collect();
		assert!(! w1.is_empty());
		assert_eq!(w1.len(), 9);
		assert!(w1.contains(&abs_p1));
		assert!(w1.contains(&abs_p2));
		assert!(! w1.contains(&abs_perr));

		// From init.
		let mut w2: Vec<PathBuf> = Dowser::from("tests/").collect();
		w1.sort();
		w2.sort();
		assert_eq!(w1, w2);
	}

	#[test]
	fn t_resolve_path() {
		let test_dir = std::fs::canonicalize("./tests/links")
			.expect("Missing dowser link directory.");

		// Make sure symlinks are detected.
		let links = std::fs::read_dir(&test_dir)
			.expect("Missing dowser link directory.")
			.filter_map(Result::ok)
			.filter_map(|e| e.file_type().ok())
			.filter(|t| t.is_symlink())
			.count();
		assert_eq!(links, 1, "Wrong symlink count!");

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
			test_dir.join("06/11"), // Sym to seven to six.
		];

		let mut canon = {
			let mut tmp: Vec<PathBuf> = raw.iter()
				.filter_map(|x| std::fs::canonicalize(x).ok())
				.collect();
			tmp.sort();
			tmp.dedup();
			tmp
		};

		// There should be two fewer entries as two are symlinks.
		assert_eq!(raw.len(), 11);
		assert_eq!(canon.len(), 8, "{:?}", canon);
		assert!(! canon.contains(&raw[6]));
		assert!(! canon.contains(&raw[9]));
		assert!(! canon.contains(&raw[10]));

		let trusting = {
			let mut tmp: Vec<PathBuf> = raw.iter()
				.filter_map(|x| Entry::from_path(x))
				.map(|e| e.path)
				.collect();
			tmp.sort();
			tmp.dedup();
			tmp
		};

		assert_eq!(trusting, canon);

		// Now let's make sure Dowser does the same thing, albeit just the
		// files.
		canon.retain(|p| p.is_file());

		let mut itered: Vec<PathBuf> = Dowser::from(test_dir.as_path()).collect();
		itered.sort();
		assert_eq!(canon, itered);

		itered = Dowser::from(test_dir.as_path()).into_vec(|_| true);
		itered.sort();
		assert_eq!(canon, itered);
	}
}
