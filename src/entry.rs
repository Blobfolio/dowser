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
pub(super) enum Entry {
	/// # Directory.
	Dir(PathBuf),

	/// # File.
	File(PathBuf),
}

impl Entry {
	#[expect(clippy::filetype_is_file, reason = "It's what we want.")]
	/// # From `DirEntry` Result.
	pub(super) fn from_entry(e: Result<DirEntry>, symlinks: bool) -> Option<Self> {
		let e = e.ok()?;
		let ft = e.file_type().ok()?;

		// We can assume the path is canonical because the root we crawled to
		// get this record was itself canonical.
		if ft.is_dir() { Some(Self::Dir(e.path())) }
		else if ft.is_file() { Some(Self::File(e.path())) }
		// Except for symlinks, of course, which need to be followed if
		// allowed…
		else if symlinks {
			let path = std::fs::canonicalize(e.path()).ok()?;
			if path.is_dir() { Some(Self::Dir(path)) }
			else { Some(Self::File(path)) }
		}
		// And ignored if not.
		else { None }
	}

	/// # From Path.
	pub(super) fn from_path(path: &Path, symlinks: bool) -> Option<Self> {
		// If symlinks are to be avoided, we need to confirm the type before
		// canonicalizing!
		if ! symlinks {
			let meta = std::fs::symlink_metadata(path).ok()?;
			if meta.file_type().is_symlink() { return None; }
		}

		// Unassociated paths can be anything; we have to canonicalize to make
		// sense of it.
		let path = std::fs::canonicalize(path).ok()?;
		if path.is_dir() { Some(Self::Dir(path)) }
		else { Some(Self::File(path)) }
	}

	#[cfg(unix)]
	#[must_use]
	#[inline]
	/// # Hash Path.
	///
	/// Since all paths are canonical, we can test for uniqueness by simply
	/// hashing them.
	pub(super) fn hash(&self) -> u64 {
		use std::os::unix::ffi::OsStrExt;
		AHASHER.hash_one(self.path().as_os_str().as_bytes())
	}

	#[cfg(not(unix))]
	#[must_use]
	#[inline]
	/// # Hash Path.
	///
	/// Since all paths are canonical, we can test for uniqueness by simply
	/// hashing them.
	pub(super) fn hash(&self) -> u64 { AHASHER.hash_one(self.path()) }

	#[inline]
	/// # Extract the Path.
	fn path(&self) -> &Path {
		match self {
			Self::Dir(p) | Self::File(p) => p.as_path(),
		}
	}
}
