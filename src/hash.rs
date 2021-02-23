/*!
# Dowser: Hashing
*/

use std::hash::{
	BuildHasher,
	Hasher,
};



#[derive(Debug, Copy, Clone)]
/// # Passthrough Hash State.
///
/// This is just a fancy alias for [`NoHashU64::default`].
pub(super) struct NoHashState;

impl BuildHasher for NoHashState {
	type Hasher = NoHashU64;

	#[inline]
	fn build_hasher(&self) -> Self::Hasher { NoHashU64::default() }
}



#[derive(Debug, Default, Copy, Clone)]
/// # Passthrough Hash.
///
/// This is a non-hashing hash for `u64` sets that uses `self` as the hash.
/// It is used by [`Dowser`] and [`dowse`] to track visited paths, which are
/// stored as pre-calculated `u64` hashes. (The set needs neither the inputs
/// nor the paths; it just needs to know whether or not a new path has already
/// been seen.)
pub(super) struct NoHashU64(u64);

impl Hasher for NoHashU64 {
	#[cold]
	/// # Write.
	///
	/// Unimplemented.
	///
	/// # Panics.
	///
	/// Calling this method will panic. Use [`NoHashU64::write_u64`] instead.
	fn write(&mut self, _bytes: &[u8]) {
		unimplemented!("Only u64 keys are supported.");
	}

	#[inline]
	/// # Write U64.
	///
	/// Write a `u64`. This is the only push method supported by this
	/// passthrough hasher. The last value sent is returned transparently, so
	/// i.e. you should only call it once per [`NoHashU64::finish`].
	fn write_u64(&mut self, i: u64) { self.0 = i; }

	#[inline]
	/// # Finish.
	fn finish(&self) -> u64 { self.0 }
}

impl NoHashU64 {
	#[must_use]
	#[inline]
	/// # Path Hash.
	///
	/// This hashes a device and inode to produce a more or less unique result.
	/// This is the value we grab for each path and use in the `HashSet`.
	pub(crate) fn hash_path(dev: u64, ino: u64) -> u64 {
		let mut hasher = wyhash::WyHash::default();
		hasher.write_u64(dev);
		hasher.write_u64(ino);
		hasher.finish()
	}
}
