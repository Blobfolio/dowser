/*!
# Dowser: Obligatory `DirEntry` Replacement.
*/

use std::{
	fs::DirEntry,
	io::Result,
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



/// # File Entry.
///
/// This holds a pre-computed hash, whether or not the path points to a
/// directory, and the canonicalized path itself.
pub(super) struct Entry {
	/// # Path.
	pub(super) path: PathBuf,

	/// # Is Directory?
	pub(super) is_dir: bool,

	/// # Hash.
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
	#[inline]
	/// # Hash Path.
	///
	/// Since all paths are canonical, we can test for uniqueness by simply
	/// hashing them.
	pub(super) fn hash_path(path: &Path) -> u64 {
		use std::os::unix::ffi::OsStrExt;
		AHASHER.hash_one(path.as_os_str().as_bytes())
	}

	#[cfg(not(unix))]
	#[must_use]
	#[inline]
	/// # Hash Path.
	///
	/// Since all paths are canonical, we can test for uniqueness by simply
	/// hashing them.
	pub(super) fn hash_path(path: &Path) -> u64 { AHASHER.hash_one(path) }
}
