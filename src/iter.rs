/*!
# Dowser: Dowser
*/

use crate::Entry;
use dactyl::NoHash;
use std::{
	collections::HashSet,
	ffi::OsStr,
	path::{
		Path,
		PathBuf,
	},
};



#[derive(Debug, Clone)]
/// # Dowser.
///
/// `Dowser` is a very simple recursive file iterator. Symlinks and hidden
/// nodes are followed like any other, and all results are canonicalized and
/// deduped prior to yielding.
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
/// From there, you can do your normal [`Iterator`](std::iter::Iterator) business, or if you just want to
/// collect the results into a vector, call [`Dowser::into_vec`] or [`Dowser::into_vec_filtered`].
///
/// ## Examples
///
/// ```no_run
/// use dowser::Dowser;
/// use std::path::PathBuf;
///
/// let files: Vec<PathBuf> = Dowser::default()
///     .with_path("/usr/share")
///     // You could filter_map(), etc., here, with the understanding that
///     // every path you get will belong to a valid, canonical file.
///     .collect();
/// ```
pub struct Dowser {
	files: Vec<PathBuf>,
	dirs: Vec<PathBuf>,
	seen: HashSet<u64, NoHash>,
}

impl Default for Dowser {
	#[inline]
	fn default() -> Self {
		Self {
			files: Vec::with_capacity(8),
			dirs: Vec::with_capacity(8),
			seen: HashSet::with_capacity_and_hasher(4096, NoHash::default()),
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
	/// Note: item ordering is arbitrary and likely to change from run-to-run.
	fn next(&mut self) -> Option<Self::Item> {
		loop {
			// We have a file ready to go!
			if let Some(p) = self.files.pop() {
				return Some(p);
			}

			if let Some(p) = self.dirs.pop() {
				if let Ok(rd) = std::fs::read_dir(p) {
					for e in rd {
						if let Some(e) = Entry::from_entry(e) {
							if self.seen.insert(e.hash) {
								if e.is_dir { self.dirs.push(e.path); }
								else { self.files.push(e.path); }
							}
						}
					}
				}
			}
			// We're out of things to do!
			else { break; }
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

	#[inline]
	#[must_use]
	/// # With Paths.
	///
	/// Queue up multiple file and/or directory paths.
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
	///
	/// ## Panics
	///
	/// This will panic if you try to pass a single `Path` or `PathBuf` object
	/// directly to this method (instead of a collection of same). Use
	/// [`Dowser::with_path`] to add such an object directly.
	pub fn with_paths<P, I>(self, paths: I) -> Self
	where P: AsRef<Path>, I: IntoIterator<Item=P> {
		assert!(! is_singular_path(&paths), "Dowser::with_paths requires an Iterator of paths, not a direct Path/PathBuf object.");
		paths.into_iter().fold(self, Self::with_path)
	}
}

impl Dowser {
	#[must_use]
	#[inline]
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
		if let Ok(p) = std::fs::canonicalize(path) {
			let hash = Entry::hash_path(&p);
			self.seen.insert(hash);
		}

		self
	}

	#[must_use]
	#[inline]
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
	///
	/// ## Panics
	///
	/// This will panic if you try to pass a single `Path` or `PathBuf` object
	/// directly to this method (instead of a collection of same). Use
	/// [`Dowser::without_path`] to add such an object directly.
	pub fn without_paths<P, I>(mut self, paths: I) -> Self
	where P: AsRef<Path>, I: IntoIterator<Item=P> {
		assert!(! is_singular_path(&paths), "Dowser::without_paths requires an Iterator of paths, not a direct Path/PathBuf object.");

		self.seen.extend(paths.into_iter().filter_map(|p|
			std::fs::canonicalize(p).ok().map(|p| Entry::hash_path(&p))
		));
		self
	}
}

impl Dowser {
	#[must_use]
	/// # Consume Into Vec.
	///
	/// This method is an optimized alternative to running
	/// `Dowser.iter().collect::<Vec<PathBuf>>()`.
	///
	/// It yields the same results as the above, but makes fewer allocations
	/// along the way.
	///
	/// ## Examples
	///
	/// ```no_run
	/// use dowser::Dowser;
	/// use std::path::PathBuf;
	///
	/// // The iterator way.
	/// let files: Vec<PathBuf> = Dowser::default()
	///     .with_path("/usr/share")
	///     .collect();
	///
	/// // The optimized way.
	/// let files: Vec<PathBuf> = Dowser::default()
	///     .with_path("/usr/share")
	///     .into_vec();
	/// ```
	pub fn into_vec(self) -> Vec<PathBuf> {
		let Self { mut files, mut dirs, mut seen } = self;

		if ! dirs.is_empty() {
			loop {
				for p in std::mem::take(&mut dirs) {
					if let Ok(rd) = std::fs::read_dir(p) {
						for e in rd {
							if let Some(e) = Entry::from_entry(e) {
								if seen.insert(e.hash) {
									if e.is_dir { dirs.push(e.path); }
									else { files.push(e.path); }
								}
							}
						}
					}
				}

				if dirs.is_empty() { break; }
			}
		}

		// Done!
		files
	}

	#[must_use]
	/// # Consume Into Vec (Filtered).
	///
	/// This method is an optimized alternative to running
	/// `Dowser.iter().filter(…).collect::<Vec<PathBuf>>()`.
	///
	/// It yields the same results as the above, but makes fewer allocations
	/// along the way.
	///
	/// Note: every entry passed to your callback will be a valid, canonical
	/// file path. (You don't have to explicitly test for [`is_file`](std::path::Path::is_file) or
	/// anything like that.)
	///
	/// ## Examples
	///
	/// ```no_run
	/// use dowser::Dowser;
	/// use std::path::PathBuf;
	///
	/// // The iterator way.
	/// let files: Vec<PathBuf> = Dowser::default()
	///     .with_path("/usr/share")
	///     .filter(|p|
	///         p.extension().map_or(
    ///             false,
    ///             |e| e.eq_ignore_ascii_case("jpg")
    ///         )
	///     )
	///     .collect();
	///
	/// // The optimized way.
	/// let files: Vec<PathBuf> = Dowser::default()
	///     .with_path("/usr/share")
	///     .into_vec_filtered(|p|
	///         p.extension().map_or(
    ///             false,
    ///             |e| e.eq_ignore_ascii_case("jpg")
    ///         )
	///     );
	/// ```
	pub fn into_vec_filtered<F>(self, cb: F) -> Vec<PathBuf>
	where F: Fn(&Path) -> bool + Sync + Send {
		let Self { mut files, mut dirs, mut seen } = self;

		// We wouldn't have had a chance to filter these yet.
		if ! files.is_empty() { files.retain(|p| cb(p)); }

		if ! dirs.is_empty() {
			loop {
				for p in std::mem::take(&mut dirs) {
					if let Ok(rd) = std::fs::read_dir(p) {
						for e in rd {
							if let Some(e) = Entry::from_entry(e) {
								if seen.insert(e.hash) {
									if e.is_dir { dirs.push(e.path); }
									else if cb(&e.path) { files.push(e.path); }
								}
							}
						}
					}
				}

				if dirs.is_empty() { break; }
			}
		}

		// Done!
		files
	}
}



/// # Is Singular Path?
///
/// Returns true if the type seems to be a singular `Path`/`PathBuf` object.
/// This is necessary to differentiate them from proper collections
/// implementing `IntoIterator<AsRef<Path>>`, at least until negative trait
/// bounds are stabilized.
fn is_singular_path<T>(raw: T) -> bool {
	fn type_of<T>(_: T) -> &'static str { std::any::type_name::<T>() }

	let kind = type_of(raw).trim_start_matches('&');
	kind == "std::path::Path" || kind == "std::path::PathBuf"
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

		itered = Dowser::from(test_dir.as_path()).into_vec_filtered(|_| true);
		itered.sort();
		assert_eq!(canon, itered);
	}

	#[test]
	#[should_panic]
	fn t_with_paths1() {
		let path: &Path = "/usr/bin".as_ref();
		let _res = Dowser::default().with_paths(path);
	}

	#[test]
	#[should_panic]
	fn t_with_paths2() {
		let path: &Path = "/usr/bin".as_ref();
		let _res = Dowser::default().with_paths(&path.to_path_buf());
	}

	#[test]
	fn t_with_paths3() {
		let path: &Path = "/usr/bin".as_ref();
		// These shouldn't panic.
		let _res = Dowser::default().with_paths(&[path]);
		let _res = Dowser::default().with_paths(&[path.to_path_buf()]);
	}

	#[test]
	#[should_panic]
	fn t_without_paths1() {
		let path: &Path = "/usr/bin".as_ref();
		let _res = Dowser::default().without_paths(path);
	}

	#[test]
	#[should_panic]
	fn t_without_paths2() {
		let path: &Path = "/usr/bin".as_ref();
		let _res = Dowser::default().without_paths(&path.to_path_buf());
	}

	#[test]
	fn t_without_paths3() {
		let path: &Path = "/usr/bin".as_ref();
		// These shouldn't panic.
		let _res = Dowser::default().without_paths(&[path]);
		let _res = Dowser::default().without_paths(&[path.to_path_buf()]);
	}
}
