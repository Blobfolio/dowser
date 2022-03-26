/*!
# Dowser: Hashing
*/

use std::hash::{
	BuildHasher,
	Hasher,
};



#[derive(Debug, Copy, Clone, Default)]
/// # Passthrough Hash State.
///
/// This is just a fancy alias for [`NoHashU64::default`].
pub(super) struct NoHashState;

impl BuildHasher for NoHashState {
	type Hasher = NoHashU64;

	#[inline]
	fn build_hasher(&self) -> Self::Hasher { NoHashU64::default() }
}



#[derive(Debug, Copy, Clone, Default)]
/// # Passthrough Hash.
///
/// This is a non-hashing hash for `u64` sets that uses `self` as the hash.
/// It is used by [`Dowser`] to track visited paths, which are stored as
/// pre-calculated `u64` hashes. (The set needs neither the inputs nor the
/// paths; it just needs to know whether or not a new path has already been
/// seen.)
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
