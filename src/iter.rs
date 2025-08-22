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
/// methods do the opposite.
///
/// If using both, the `without_*`s should be chained _first_ to ensure they
/// have priority over any matching paths in the `with_*`s.
///
/// From there, you can do your normal [`Iterator`](std::iter::Iterator) business.
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
	/// # Found Files.
	files: Vec<PathBuf>,

	/// # Found Directories.
	dirs: Vec<PathBuf>,

	/// # Encountered Hashes.
	///
	/// This is used to prevent parsing the same file/directory twice.
	seen: HashSet<u64, NoHash>,

	/// # Symlinks?
	///
	/// If `true`, follow and canonicalize symlinks; if `false`, ignore them.
	symlinks: bool,
}

impl Default for Dowser {
	#[inline]
	fn default() -> Self {
		Self {
			files: Vec::with_capacity(8),
			dirs: Vec::with_capacity(8),
			seen: HashSet::with_capacity_and_hasher(4096, NoHash::default()),
			symlinks: true,
		}
	}
}

/// # Helper: Generate From Impl.
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

		for e in src.iter().filter_map(|p| Entry::from_path(p, true)) {
			if out.seen.insert(e.hash()) {
				match e {
					Entry::Dir(p) =>  { out.dirs.push(p); },
					Entry::File(p) => { out.files.push(p); },
				}
			}
		}

		out
	}
}

impl From<Vec<PathBuf>> for Dowser {
	fn from(src: Vec<PathBuf>) -> Self {
		let mut out = Self::default();

		for e in src.iter().filter_map(|p| Entry::from_path(p, true)) {
			if out.seen.insert(e.hash()) {
				match e {
					Entry::Dir(p) =>  { out.dirs.push(p); },
					Entry::File(p) => { out.files.push(p); },
				}
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
			// If we have a file ready-to-go, return it!
			if let Some(p) = self.files.pop() { return Some(p); }

			// Otherwise crawl the next directory, if any.
			let p = self.dirs.pop()?;
			let Ok(rd) = std::fs::read_dir(p) else { continue; };
			for e in rd {
				if
					let Some(e) = Entry::from_entry(e, self.symlinks) &&
					self.seen.insert(e.hash())
				{
					match e {
						Entry::Dir(p) =>  { self.dirs.push(p); },
						Entry::File(p) => { self.files.push(p); },
					}
				}
			}
		}
	}

	/// # Size Hints.
	///
	/// This iterator has an unknown size until the final directory has been
	/// read, after which point it is just a matter of flushing the files it
	/// found there.
	fn size_hint(&self) -> (usize, Option<usize>) {
		let lower = self.files.len();
		let upper = self.dirs.is_empty().then_some(lower);
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
		self.push_path(path);
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
	/// # Push Path.
	///
	/// Queue up a single file or directory path.
	///
	/// See also [`Dowser::with_path`].
	///
	/// ## Examples
	///
	/// ```no_run
	/// use dowser::Dowser;
	/// use std::path::PathBuf;
	///
	/// let mut crawl = Dowser::default();
	/// crawl.push_path("/my/dir");
	/// let files: Vec::<PathBuf> = crawl.collect();
	/// ```
	pub fn push_path<P>(&mut self, path: P)
	where P: AsRef<Path> {
		if
			let Some(e) = Entry::from_path(path.as_ref(), self.symlinks) &&
			self.seen.insert(e.hash())
		{
			match e {
				Entry::Dir(p) =>  { self.dirs.push(p); },
				Entry::File(p) => { self.files.push(p); },
			}
		}
	}
}

impl Dowser {
	/// # Load Paths From File.
	///
	/// Queue up multiple file and/or directory paths from a text file, one
	/// entry per line.
	///
	/// Lines are trimmed and ignored if empty, but otherwise resolved the
	/// same as if passed directly to methods like [`Dowser::with_path`], i.e.
	/// relative to the current working directory (not the text file itself).
	///
	/// For that reason, it is recommended that all paths stored in text files
	/// be absolute to avoid any ambiguity.
	///
	/// ## Examples
	///
	/// ```no_run
	/// use dowser::Dowser;
	/// use std::path::PathBuf;
	///
	/// // Read the paths from list.txt.
	/// let mut crawler = Dowser::default();
	/// crawler.read_paths_from_file("list.txt").unwrap();
	///
	/// // Crunch into a vec.
	/// let files: Vec::<PathBuf> = crawler.collect();
	/// ```
	///
	/// ## Errors
	///
	/// This method will bubble up any errors encountered while trying to read
	/// the text file.
	pub fn read_paths_from_file<P: AsRef<Path>>(&mut self, src: P)
	-> Result<(), std::io::Error> {
		let raw = std::fs::read_to_string(src)?;
		for line in raw.lines() {
			let line = line.trim();
			if
				! line.is_empty() &&
				let Some(e) = Entry::from_path(line.as_ref(), self.symlinks) &&
				self.seen.insert(e.hash())
			{
				match e {
					Entry::Dir(p) =>  { self.dirs.push(p); },
					Entry::File(p) => { self.files.push(p); },
				}
			}
		}

		Ok(())
	}
}

impl Dowser {
	#[must_use]
	#[inline]
	/// # Without Symlinks.
	///
	/// Ignore any and all symlinks rather than following them, as [`Dowser`]
	/// otherwise does by default.
	///
	/// Note: this setting is not retroactive; call this method before adding
	/// any paths.
	///
	/// ## Examples
	///
	/// ```no_run
	/// use dowser::Dowser;
	/// use std::path::PathBuf;
	///
	/// let files: Vec<PathBuf> = Dowser::default() // Symlinks would be followed.
	///     .without_symlinks()                     // Now they won't be!
	///     .with_path("/my/dir")
	///     .collect();
	/// ```
	pub const fn without_symlinks(mut self) -> Self {
		self.symlinks = false;
		self
	}

	#[must_use]
	#[inline]
	/// # Without Path.
	///
	/// This will prevent the provided directory or file from being (directly)
	/// crawled or included in the output.
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
		if let Some(e) = Entry::from_path(path.as_ref(), self.symlinks) {
			self.seen.insert(e.hash());
		}

		self
	}

	#[must_use]
	#[inline]
	/// # Without Paths.
	///
	/// This will prevent the provided paths from being (directly) crawled or
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
	pub fn without_paths<P, I>(self, paths: I) -> Self
	where P: AsRef<Path>, I: IntoIterator<Item=P> {
		assert!(
			! is_singular_path(&paths),
			"Dowser::without_paths requires an Iterator of paths, not a direct Path/PathBuf object.",
		);
		paths.into_iter().fold(self, Self::without_path)
	}
}



/// # Is Singular Path?
///
/// Returns true if the type seems to be a singular `Path`/`PathBuf` object.
/// This is necessary to differentiate them from proper collections
/// implementing `IntoIterator<AsRef<Path>>`, at least until negative trait
/// bounds are stabilized.
fn is_singular_path<T>(raw: T) -> bool {
	/// # Type to String.
	fn type_of<T>(_: T) -> &'static str { std::any::type_name::<T>() }

	let kind = type_of(raw).trim_start_matches('&');
	kind == "std::path::Path" || kind == "std::path::PathBuf"
}



#[cfg(test)]
mod tests {
	use super::*;
	use brunch as _;
	use std::collections::BTreeSet;

	#[test]
	fn t_new() {
		let mut abs_dir = std::fs::canonicalize("tests/assets/").unwrap();
		abs_dir.push("_.txt");
		let abs_p1 = abs_dir.with_file_name("file.txt");
		let abs_p2 = abs_dir.with_file_name("is-executable.sh");
		let abs_p3 = std::fs::canonicalize("tests/extensions.txt").unwrap();
		let abs_perr = abs_dir.with_file_name("foo.bar");

		// Builder init.
		let mut w1: Vec<PathBuf> = Dowser::default()
			.with_path(PathBuf::from("tests/"))
			.collect();
		assert!(! w1.is_empty());
		assert_eq!(w1.len(), 10);
		assert!(w1.contains(&abs_p1));
		assert!(w1.contains(&abs_p2));
		assert!(w1.contains(&abs_p3));
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
			.filter(std::fs::FileType::is_symlink)
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
		assert_eq!(canon.len(), 8, "{canon:?}");
		assert!(! canon.contains(&raw[6]));
		assert!(! canon.contains(&raw[9]));
		assert!(! canon.contains(&raw[10]));

		let trusting = {
			let mut tmp: Vec<PathBuf> = raw.iter()
				.filter_map(|p| Entry::from_path(p, true))
				.map(|e| match e { Entry::Dir(p) | Entry::File(p) => p })
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

		// Almost done... last thing to check is the symlink logic!
		let six_dir = std::fs::canonicalize(test_dir.join("06")).expect("Missing test dir 06");
		let yay: Vec<_> = Dowser::default().with_path(&six_dir).collect();
		let nay: Vec<_> = Dowser::default().without_symlinks().with_path(&six_dir).collect();
		assert!(! nay.is_empty(), "BUG: Symlinks logic broke totals.");
		assert!(nay.len() < yay.len(), "BUG: Symlinks were followed!");
		assert!(
			nay.iter().all(|p| p.parent().is_some_and(|p| p == six_dir)),
			"Bug: Symlinks were followed!",
		);
	}

	#[test]
	fn t_without() {
		let root = std::fs::canonicalize("./tests/links").expect("Missing test directory.");

		// Exclude 4 (file) and 6 (directory).
		let found: BTreeSet<PathBuf> = Dowser::default()
			.without_paths(&["./tests/links/04", "./tests/links/06"])
			.with_path("./tests/links")
			.collect();

		assert_eq!(
			found.len(),
			4,
			"Unexpected number of files found!"
		);

		for stub in ["01", "02", "03", "06/08"] {
			assert!(
				found.contains(&root.join(stub)),
				"Missing {stub}.",
			);
		}
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
		let _res = Dowser::default().with_paths([path]);
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
		let _res = Dowser::default().without_paths([path]);
		let _res = Dowser::default().without_paths(&[path.to_path_buf()]);
	}

	#[test]
	fn t_read_paths_from_file() {
		use std::fs::File;
		use std::io::Write;

		// Find the temporary directory.
		let tmp = std::env::temp_dir();
		if ! tmp.is_dir() { return; }

		// Declare a few paths to test crawl.
		let asset_dir = std::fs::canonicalize("tests/assets")
			.expect("Missing dowser assets dir");
		let link01 = std::fs::canonicalize("tests/links/01")
			.expect("Missing dowser links/01");

		// Mock up a text file containing those entries.
		let text_file = tmp.join("dowser.test.txt");
		let text = format!(
			"{}\n{}\n",
			asset_dir.as_os_str().to_str().expect("Asset dir cannot be represented as a string."),
			link01.as_os_str().to_str().expect("Link01 cannot be represented as a string."),
		);

		// Try to save the text file.
		let res = File::create(&text_file)
			.and_then(|mut file|
				file.write_all(text.as_bytes()).and_then(|()| file.flush())
			);

		// Not all environments will allow that; only proceed with the testing
		// if it worked.
		if res.is_ok() && text_file.is_file() {
			// Feed the text file to dowser, collect the results.
			let mut crawl = Dowser::default();
			crawl.read_paths_from_file(&text_file)
				.expect("Loading text file failed.");
			let found: BTreeSet<PathBuf> = crawl.collect();

			// We don't need the text file anymore.
			let _res = std::fs::remove_file(text_file);

			// It should have found the following!
			assert!(found.len() == 4);
			assert!(found.contains(&link01));
			assert!(found.contains(&asset_dir.join("file.txt")));
			assert!(found.contains(&asset_dir.join("functioning.JPEG")));
			assert!(found.contains(&asset_dir.join("is-executable.sh")));
		}
	}
}
