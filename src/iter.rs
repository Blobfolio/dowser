/*!
# Dowser: Dowser
*/

use dactyl::NoHash;
use std::{
	collections::HashSet,
	ffi::{
		OsStr,
		OsString,
	},
	fs::DirEntry,
	path::{
		Path,
		PathBuf,
	},
};



/// # Static Hasher.
///
/// This is used for cheap collision detection. No need to get fancy with it.
const AHASHER: ahash::RandomState = ahash::RandomState::with_seeds(
	0x8596_cc44_bef0_1aa0,
	0x98d4_0948_da60_19ae,
	0x49f1_3013_c503_a6aa,
	0xc4d7_82ff_3c9f_7bef,
);



#[derive(Debug, Clone)]
/// # Dowser.
///
/// `Dowser` is a very simple recursive file iterator. Symlinks and hidden
/// nodes are followed like any other, and all results are canonicalized and
/// deduped prior to yielding.
///
/// ## Usage
///
/// Create a new instance using [`Dowser::default`], then specify root paths
/// to ignore and/or include with [`Dowser::without_path`] and
/// [`Dowser::with_path`], respectively.
///
/// (When excluding _and_ including, the order matters. To be safe, apply the
/// withouts first.)
///
/// From there, leverage your favorite [`Iterator`](std::iter::Iterator)
/// trait methods to filter/collect the results.
///
/// ## Examples
///
/// ```
/// use dowser::Dowser;
/// use std::path::PathBuf;
///
/// let files: Vec<PathBuf> = Dowser::default()
///     .with_path("/usr/share/images")
///     .filter(|p|
///         p.extension()
///             .is_some_and(|ext| ext.eq_ignore_ascii_case("bmp"))
///     )
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
	/// # New Instance.
	///
	/// Initialize and return a new [`Dowser`] instance with modest initial
	/// buffers and symlink-following enabled.
	fn default() -> Self {
		Self {
			files: Vec::with_capacity(8),
			dirs: Vec::with_capacity(8),
			seen: HashSet::with_capacity_and_hasher(4096, NoHash::default()),
			symlinks: true,
		}
	}
}

/// # Helper: Generate `From` impls.
macro_rules! from {
	($($ty:ty),+ $(,)?) => ($(
		/// # New Instance w/ Entry Point.
		///
		/// This method is equivalent to calling [`Dowser::default`] and
		/// [`Dowser::with_path`] separately.
		impl From<$ty> for Dowser {
			#[inline]
			fn from(src: $ty) -> Self { Self::default().with_path(src) }
		}

		/// # New Instance w/ Entry Point(s).
		///
		/// This method is equivalent to calling [`Dowser::default`], then
		/// calling [`Dowser::with_path`] in a loop.
		impl From<&[$ty]> for Dowser {
			#[inline]
			fn from(src: &[$ty]) -> Self {
				src.iter().fold(Dowser::default(), Dowser::with_path)
			}
		}

		/// # New Instance w/ Entry Point(s).
		///
		/// This method is equivalent to calling [`Dowser::default`], then
		/// calling [`Dowser::with_path`] in a loop.
		impl From<Vec<$ty>> for Dowser {
			#[inline]
			fn from(src: Vec<$ty>) -> Self { Self::from(src.as_slice()) }
		}
	)+);
}

from!{
	&OsStr,
	&OsString, OsString,
	&Path,
	&PathBuf,  PathBuf,
	&str,
	&String,   String,
}

impl Iterator for Dowser {
	type Item = PathBuf;

	/// # Next!
	///
	/// This iterator yields canonical, deduplicated _file_ paths. Directories
	/// are recursively traversed, but their paths are not shared.
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
					let Ok(e) = e &&
					let Some(e) = Entry::from_dir_entry(&e, self.symlinks)
				{
					self.record_entry(e);
				}
			}

			// Rinse and repeat.
		}
	}

	/// # Size Hints.
	///
	/// Because entry points can technically be added at any point, no upper
	/// bound is specified. If there are files in the buffer that haven't been
	/// returned yet, their count serves as the lower limit.
	fn size_hint(&self) -> (usize, Option<usize>) {
		(self.files.len(), None)
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
	/// ```
	/// use dowser::Dowser;
	/// use std::path::PathBuf;
	///
	/// let mut crawl = Dowser::default();
	/// crawl.push_path("/my/dir");
	/// let files: Vec<PathBuf> = crawl.collect();
	///
	/// // Alternatively, you can just use `From`:
	/// let files: Vec<PathBuf> = Dowser::from("/my/dir").collect();
	/// ```
	pub fn push_path<P>(&mut self, path: P)
	where P: AsRef<Path> {
		if let Some(e) = Entry::from_path(path.as_ref(), self.symlinks) {
			self.record_entry(e);
		}
	}

	/// # Push Path(s) From File.
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
	/// crawler.push_paths_from_file("list.txt").unwrap();
	///
	/// // Crunch into a vec.
	/// let files: Vec<PathBuf> = crawler.collect();
	/// ```
	///
	/// ## Errors
	///
	/// This method will bubble up any errors encountered while trying to read
	/// the text file.
	pub fn push_paths_from_file<P: AsRef<Path>>(&mut self, src: P)
	-> Result<(), std::io::Error> {
		let raw = std::fs::read_to_string(src)?;
		for line in raw.lines() {
			let line = line.trim();
			if
				! line.is_empty() &&
				let Some(e) = Entry::from_path(line.as_ref(), self.symlinks)
			{
				self.record_entry(e);
			}
		}

		Ok(())
	}

	#[must_use]
	/// # With Path.
	///
	/// Queue up a single file or directory path.
	///
	/// This can be called multiple times if you want to crawl multiple roots.
	///
	/// ## Examples
	///
	/// ```
	/// use dowser::Dowser;
	/// use std::path::PathBuf;
	///
	/// let files: Vec<PathBuf> = Dowser::default()
	///     .with_path("/my/dir")
	///     .collect();
	///
	/// // Alternatively, you can just use `From`:
	/// let files: Vec<PathBuf> = Dowser::from("/my/dir").collect();
	/// ```
	pub fn with_path<P>(mut self, path: P) -> Self
	where P: AsRef<Path> {
		self.push_path(path);
		self
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
	/// ```
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
	/// This method can be used to pre-emptively mark a file or directory path
	/// as "seen", causing it to be ignored should it come up during the crawl.
	///
	/// It is recommended you specify "without" paths before "with" paths, just
	/// in case there's any overlap.
	///
	/// Note: [`Dowser`] does not explicitly test for ancestry, so while an
	/// excluded directory will never itself be crawled, select child paths
	/// can still turn up in the results if external links resolve directly to
	/// _them_ (and symlink-following is enabled).
	///
	/// ## Examples
	///
	/// ```
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
}

impl Dowser {
	#[inline]
	/// # Record Path Entry.
	///
	/// Mark a path as "seen" and if new, add it to the type-appropriate
	/// bucket for later.
	fn record_entry(&mut self, e: Entry) {
		if self.seen.insert(e.hash()) {
			match e {
				Entry::Dir(p) =>  { self.dirs.push(p); },
				Entry::File(p) => { self.files.push(p); },
			}
		}
	}
}



#[derive(Debug, Clone, Eq, PartialEq)]
/// # Typed Path Entry.
enum Entry {
	/// # Directory.
	Dir(PathBuf),

	/// # File.
	File(PathBuf),
}

impl Entry {
	/// # From Path (Untrusted).
	///
	/// Canonicalize, qualify, and return an [`Entry`] from an arbitrary system
	/// path.
	///
	/// If the path doesn't exist, its metadata can't be parsed, or it is a
	/// symlink and `follow == false`, `None` is returned instead.
	fn from_path(path: &Path, follow: bool) -> Option<Self> {
		// If symlinks are disabled, we need to confirm this isn't one.
		if ! follow {
			let meta = std::fs::symlink_metadata(path).ok()?;
			if meta.file_type().is_symlink() { return None; }
		}

		// Canonicalize and return!
		if
			let Ok(path) = std::fs::canonicalize(path) &&
			let Ok(meta) = std::fs::symlink_metadata(&path) // Path is canonical so no need to resolve links.
		{
			if meta.is_dir() { Some(Self::Dir(path)) }
			else { Some(Self::File(path)) }
		}
		else { None }
	}

	#[expect(clippy::filetype_is_file, reason = "We're testing all three possibilities.")]
	#[inline]
	/// # From `DirEntry`.
	///
	/// An optimized alternative to [`Entry::from_path`] used when processing
	/// items yielded during [`read_dir`](std::fs::read_dir) operations.
	fn from_dir_entry(e: &DirEntry, follow: bool) -> Option<Self> {
		let ft = e.file_type().ok()?;

		// We can assume the path is canonical if a file or directory because
		// the directory being read was itself canonical.
		if ft.is_dir() { Some(Self::Dir(e.path())) }
		else if ft.is_file() { Some(Self::File(e.path())) }

		// The same cannot be said for symlinks…
		else if
			follow &&
			let Ok(path) = std::fs::canonicalize(e.path()) &&
			let Ok(meta) = std::fs::symlink_metadata(&path) // Path is canonical so no need to resolve links.
		{
			if meta.is_dir() { Some(Self::Dir(path)) }
			else { Some(Self::File(path)) }
		}

		// If we aren't following symlinks, we have our answer.
		else { None }
	}
}

impl Entry {
	#[cfg(unix)]
	#[must_use]
	#[inline]
	/// # Hash Path (Optimized).
	///
	/// Entry paths are always canonical, so hashes can serve as a proxy for
	/// uniqueness.
	pub(super) fn hash(&self) -> u64 {
		use std::os::unix::ffi::OsStrExt;

		// Bytes hash faster than path components.
		AHASHER.hash_one(self.path().as_os_str().as_bytes())
	}

	#[cfg(not(unix))]
	#[must_use]
	#[inline]
	/// # Hash Path (Unoptimized).
	///
	/// Entry paths are always canonical, so hashes can serve as a proxy for
	/// uniqueness.
	pub(super) fn hash(&self) -> u64 { AHASHER.hash_one(self.path()) }

	#[inline]
	/// # Extract the Path.
	fn path(&self) -> &Path {
		match self { Self::Dir(p) | Self::File(p) => p.as_path() }
	}
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
			.without_path("./tests/links/04")
			.without_path("./tests/links/06")
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
	fn t_push_paths_from_file() {
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
			crawl.push_paths_from_file(&text_file)
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
