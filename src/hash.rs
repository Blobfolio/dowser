/*!
# Dowser: Hashing
*/

use std::hash::{
	BuildHasherDefault,
	Hasher,
};



/// # No-Hash Hash State.
pub(super) type NoHashState = BuildHasherDefault<NoHashU64>;



#[derive(Copy, Clone, Default)]
/// # No-Hash Hash.
///
/// This is a non-hashing hasher for `u64` values, i.e. the value is also the
/// hash.
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
