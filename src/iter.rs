/*!
# Dowser: Dowser
*/

use crate::NoHashState;

#[cfg(feature = "parking_lot_mutex")]
use parking_lot::Mutex;

use rayon::iter::{
	IntoParallelIterator,
	ParallelBridge,
	ParallelIterator,
};
use std::{
	collections::HashSet,
	fs::DirEntry,
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



/// # Resolved Path.
///
/// This tuple represents the hash, whether or not it is a directory, and the
/// canonical path.
type Resolved = (u64, bool, PathBuf);



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
	threads: usize,
	seen: HashSet<u64, NoHashState>,
}

impl Default for Dowser {
	fn default() -> Self {
		Self {
			files: Vec::with_capacity(8),
			dirs: Vec::with_capacity(8),
			threads: dowser_threads(),
			seen: HashSet::with_capacity_and_hasher(4096, NoHashState::default()),
		}
	}
}

impl From<PathBuf> for Dowser {
	fn from(src: PathBuf) -> Self {
		if let Some((h, is_dir, p)) = resolve_path(src) {
			let mut seen = HashSet::with_capacity_and_hasher(4096, NoHashState::default());
			seen.insert(h);

			let mut dirs = Vec::with_capacity(8);
			let mut files = Vec::with_capacity(8);

			if is_dir { dirs.push(p); }
			else { files.push(p); }

			Self {
				files,
				dirs,
				threads: dowser_threads(),
				seen,
			}
		}
		else { Self::default() }
	}
}

impl From<&Path> for Dowser {
	fn from(src: &Path) -> Self { Self::from(src.to_path_buf()) }
}

impl From<&PathBuf> for Dowser {
	fn from(src: &PathBuf) -> Self { Self::from(src.clone()) }
}

impl From<&[PathBuf]> for Dowser {
	fn from(src: &[PathBuf]) -> Self {
		let mut seen = HashSet::with_capacity_and_hasher(4096, NoHashState::default());
		let mut dirs = Vec::with_capacity(8);
		let mut files = Vec::with_capacity(8);

		for (h, is_dir, p) in src.iter().filter_map(|p| resolve_path(p.clone())) {
			if seen.insert(h) {
				if is_dir { dirs.push(p); }
				else { files.push(p); }
			}
		}

		Self {
			files,
			dirs,
			threads: dowser_threads(),
			seen,
		}
	}
}

impl From<Vec<PathBuf>> for Dowser {
	fn from(src: Vec<PathBuf>) -> Self {
		let mut seen = HashSet::with_capacity_and_hasher(4096, NoHashState::default());
		let mut dirs = Vec::with_capacity(8);
		let mut files = Vec::with_capacity(8);

		for (h, is_dir, p) in src.into_iter().filter_map(resolve_path) {
			if seen.insert(h) {
				if is_dir { dirs.push(p); }
				else { files.push(p); }
			}
		}

		Self {
			files,
			dirs,
			threads: dowser_threads(),
			seen,
		}
	}
}

impl Iterator for Dowser {
	type Item = PathBuf;

	fn next(&mut self) -> Option<Self::Item> {
		loop {
			// We have a file ready to go!
			if let Some(p) = self.files.pop() {
				return Some(p);
			}
			// Read some directories.
			match self.dirs.len().min(self.threads) {
				0 => break,
				1 => { self.crawl(); },
				n => { self.crawl_n(n); },
			}
		}

		None
	}

	fn size_hint(&self) -> (usize, Option<usize>) {
		let dirs = self.dirs.len();
		let files = self.files.len();

		if 0 == dirs { (files, Some(files)) }
		else { (files + dirs, None) }
	}
}

impl Dowser {
	/// # Crunch One.
	///
	/// This reads the final directory in the queue when there is only one.
	fn crawl(&mut self) {
		if let Some(p) = self.dirs.pop() {
			if let Ok(rd) = std::fs::read_dir(p) {
				let seen = Mutex::new(&mut self.seen);
				let f = Mutex::new(&mut self.files);
				let d = Mutex::new(&mut self.dirs);

				rd.par_bridge()
					.for_each(|e| if let Some((h, is_dir, p)) = resolve_entry(e) {
						if mutex!(seen).insert(h) {
							if is_dir { mutex!(d).push(p); }
							else { mutex!(f).push(p); }
						}
					});
			}
		}
	}

	/// # Crunch Many.
	///
	/// This reads multiple directories — but maybe not _all_ — in the queue,
	/// using multiple threads to spread the effort out a bit.
	fn crawl_n(&mut self, n: usize) {
		// Split off so we can write right back to self.dirs during iteration.
		let new = self.dirs.split_off(self.dirs.len() - n);

		let seen = Mutex::new(&mut self.seen);
		let f = Mutex::new(&mut self.files);
		let d = Mutex::new(&mut self.dirs);

		new.into_par_iter()
			.filter_map(|p| std::fs::read_dir(p).ok())
			.flat_map(ParallelBridge::par_bridge)
			.for_each(|e| if let Some((h, is_dir, p)) = resolve_entry(e) {
				if mutex!(seen).insert(h) {
					if is_dir { mutex!(d).push(p); }
					else { mutex!(f).push(p); }
				}
			});
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
	where
		P: AsRef<Path>,
		I: IntoIterator<Item=P> {
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
		if let Some((h, is_dir, p)) = resolve_path(path.as_ref().to_path_buf()) {
			if self.seen.insert(h) {
				if is_dir { self.dirs.push(p); }
				else { self.files.push(p); }
			}
		}

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
		if let Some(h) = resolve_path_hash(path.as_ref()) {
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
	where
		P: AsRef<Path>,
		I: IntoIterator<Item=P> {
		self.seen.extend(
			paths.into_iter()
				.filter_map(|p| resolve_path_hash(p.as_ref()))
		);

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
	/// along the way and applies your filter callback in parallel.
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
		let Self { mut files, mut dirs, threads, mut seen } = self;

		// We wouldn't have had a chance to filter these yet.
		if ! files.is_empty() {
			files.retain(|p| cb(p));
		}

		// Consume!
		{
			let s = Mutex::new(&mut seen);
			let f = Mutex::new(&mut files);

			while ! dirs.is_empty() {
				let new = dirs.split_off(dirs.len().saturating_sub(threads));
				let d = Mutex::new(&mut dirs);

				new.into_par_iter()
					.filter_map(|p| std::fs::read_dir(p).ok())
					.flat_map(ParallelBridge::par_bridge)
					.for_each(|e| if let Some((h, is_dir, p)) = resolve_entry(e) {
						if mutex!(s).insert(h) {
							if is_dir { mutex!(d).push(p); }
							else if cb(&p) { mutex!(f).push(p); }
						}
					});
			}
		}

		// Done!
		files
	}
}



#[inline]
/// # Thread Count.
///
/// This returns the ideal number of threads to use when crawling directories.
/// To help with `ulimit` difficulties, this is either one less than what Rayon
/// would normally use, or 8, whichever is lower.
fn dowser_threads() -> usize {
	match rayon::current_num_threads() {
		0..=2 => 1,
		n => usize::min(n - 1, 8),
	}
}

/// # Hash Path.
fn hash_path(dev: u64, ino: u64) -> u64 {
	let mut hasher = ahash::AHasher::new_with_keys(1319, 2371);
	hasher.write_u64(dev);
	hasher.write_u64(ino);
	hasher.finish()
}

/// # Resolve Entry Result.
///
/// This is like `resolve_path`, except it assumes that any non-symlink entry
/// path is canonical, because its parent path was canonical.
///
/// Symlinks get passed to `resolve_path`, ensuring they're fully
/// canonicalized.
fn resolve_entry(e: Result<DirEntry, std::io::Error>) -> Option<Resolved> {
	let e = e.ok()?;

	// If this is a symlink, we have to follow it.
	if e.file_type().map_or(true, |ft| ft.is_symlink()) {
		return resolve_path(e.path());
	}

	let meta = e.metadata().ok()?;
	let path = e.path();
	let hash: u64 = hash_path(meta.dev(), meta.ino());
	Some((hash, meta.is_dir(), path))
}

/// # Resolve Path.
///
/// This attempts to cheaply resolve a given path, returning:
/// * A unique hash derived from the path's device and inode.
/// * A bool indicating whether or not the path is a directory.
/// * The canonicalized path.
fn resolve_path(path: PathBuf) -> Option<Resolved> {
	let path = std::fs::canonicalize(path).ok()?;
	let meta = std::fs::metadata(&path).ok()?;
	let hash: u64 = hash_path(meta.dev(), meta.ino());
	Some((hash, meta.is_dir(), path))
}

/// # Resolve Path Hash.
///
/// This is identical to `resolve_path`, except it only returns the hash. It
/// is used by [`Dowser::without_paths`] and [`Dowser::without_path`], which
/// don't actually need anything more.
fn resolve_path_hash(path: &Path) -> Option<u64> {
	if let Ok(meta) = std::fs::symlink_metadata(&path) {
		if ! meta.file_type().is_symlink() {
			return Some(hash_path(meta.dev(), meta.ino()));
		}
	}

	let path = std::fs::canonicalize(path).ok()?;
	let meta = std::fs::metadata(&path).ok()?;
	Some(hash_path(meta.dev(), meta.ino()))
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
		let mut w2: Vec<PathBuf> = Dowser::from(PathBuf::from("tests/")).collect();
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
				.filter_map(|x| resolve_path(x.clone()).map(|(_, _, p)| p))
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
