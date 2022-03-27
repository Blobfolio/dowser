/*!
# Dowser: Obligatory `DirEntry` Replacement.
*/

use ahash::AHasher;
use std::{
	fs::{
		DirEntry,
		Metadata,
	},
	hash::Hasher,
	io::Result,
	os::unix::fs::MetadataExt,
	path::{
		Path,
		PathBuf,
	},
};



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
	pub(super) fn from_path<P>(path: P) -> Option<Self>
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
		let mut hasher = AHasher::new_with_keys(1319, 2371);
		hasher.write_u64(meta.dev());
		hasher.write_u64(meta.ino());
		hasher.finish()
	}

	#[must_use]
	/// # Hash Path.
	///
	/// This returns an appropriate hash for a given path. It is primarily used
	/// in cases where the rest of the `Entry` data is not needed.
	pub(super) fn hash_path<P>(path: P) -> Option<u64>
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
