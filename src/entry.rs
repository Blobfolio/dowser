/*!
# Dowser: Obligatory `DirEntry` Replacement.
*/

use ahash::AHasher;
use std::{
	fs::DirEntry,
	hash::Hasher,
	io::Result,
	path::{
		Path,
		PathBuf,
	},
};



#[allow(clippy::redundant_pub_crate)] // Fix this shit already. Haha.
/// # File Entry.
///
/// This holds a pre-computed hash, whether or not the path points to a
/// directory, and the canonicalized path itself.
pub(super) struct Entry {
	pub(super) path: PathBuf,
	pub(super) is_dir: bool,
	pub(super) hash: u64,
}

impl Entry {
	#[must_use]
	/// # From Entry (Result).
	///
	/// Because [`Dowser`] canonicalizes all seed paths, we can assume that
	/// any non-symlinked `DirEntry` is also canonical, thus avoiding expensive
	/// syscalls. (If it is, we'll canonicalize it first.)
	pub(super) fn from_entry(e: Result<DirEntry>) -> Option<Self> {
		// If this is a symlink, we have to follow it.
		let e = e.ok()?;
		let ft = e.file_type().ok()?;
		if ft.is_symlink() { Self::from_path(e.path()) }
		else {
			let path = e.path();
			let hash = Self::hash_path(&path);
			Some(Self {
				path,
				is_dir: ft.is_dir(),
				hash,
			})
		}
	}

	#[must_use]
	/// # From Path.
	///
	/// Paths sent to this method are untrusted and forced through
	/// canonicalization before any metadata is worked out.
	pub(super) fn from_path<P>(path: P) -> Option<Self>
	where P: AsRef<Path> {
		let path = std::fs::canonicalize(path).ok()?;
		let hash = Self::hash_path(&path);
		let is_dir = path.is_dir();

		Some(Self { path, is_dir, hash })
	}

	#[cfg(unix)]
	#[must_use]
	/// # Hash Path.
	///
	/// Since all paths are canonical, we can test for uniqueness by simply
	/// hashing them.
	pub(super) fn hash_path(path: &Path) -> u64 {
		use std::os::unix::ffi::OsStrExt;
		let mut hasher = AHasher::default();
		hasher.write(path.as_os_str().as_bytes());
		hasher.finish()
	}

	#[cfg(not(unix))]
	#[must_use]
	/// # Hash Path.
	///
	/// Since all paths are canonical, we can test for uniqueness by simply
	/// hashing them.
	pub(super) fn hash_path(path: &Path) -> u64 {
		use std::hash::Hash;
		let mut hasher = AHasher::default();
		path.hash(&mut hasher);
		hasher.finish()
	}
}
