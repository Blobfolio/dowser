/*!
# Dowser: Obligatory `DirEntry` Replacement.
*/

use ahash::AHasher;
use std::{
	fs::DirEntry,
	hash::Hasher,
	io::Result,
	os::unix::fs::{
		DirEntryExt,
		MetadataExt,
	},
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
	pub(super) dev: u64,
	pub(super) hash: u64,
}

impl Entry {
	#[must_use]
	/// # From Entry (Result).
	///
	/// Because [`Dowser`] canonicalizes all seed paths, we can assume that
	/// any non-symlinked `DirEntry` is also canonical, thus avoiding expensive
	/// syscalls. (If it is, we'll canonicalize it first.)
	pub(super) fn from_entry(e: Result<DirEntry>, dev: u64) -> Option<Self> {
		// If this is a symlink, we have to follow it.
		let e = e.ok()?;
		let ft = e.file_type().ok()?;
		if ft.is_symlink() {
			return Self::from_path(e.path());
		}

		Some(Self {
			path: e.path(),
			is_dir: ft.is_dir(),
			dev, // Assume the device is unchanged.
			hash: Self::hash_two(dev, e.ino()),
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
		let dev = meta.dev();

		Some(Self {
			path,
			is_dir: meta.is_dir(),
			dev,
			hash: Self::hash_two(dev, meta.ino()),
		})
	}

	#[must_use]
	/// # Hash Meta.
	///
	/// On Unix systems, file uniqueness means a unique device/inode
	/// combination.
	fn hash_two(dev: u64, ino: u64) -> u64 {
		let mut hasher = AHasher::new_with_keys(1319, 2371);
		hasher.write_u64(dev);
		hasher.write_u64(ino);
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

		// If this isn't a symlink, the metadata device/inode can be trusted.
		if let Ok(meta) = std::fs::symlink_metadata(path) {
			if ! meta.is_symlink() {
				return Some(Self::hash_two(meta.dev(), meta.ino()));
			}
		}

		// Otherwise we should canonicalize first, then refetch the metadata.
		std::fs::canonicalize(path)
			.and_then(std::fs::metadata)
			.ok()
			.map(|m| Self::hash_two(m.dev(), m.ino()))
	}
}
