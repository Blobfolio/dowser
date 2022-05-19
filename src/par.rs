/*!
# Dowser: Parallelism
*/

use std::num::NonZeroUsize;



#[derive(Debug, Clone, Copy)]
/// # Directory Concurrency.
///
/// This enum determines if and how many directories [`Dowser`](crate::Dowser) should try to
/// read in parallel, which is configured via [`Dowser::with_dir_concurrency`](crate::Dowser::with_dir_concurrency).
///
/// The default is [`DirConcurrency::Sane`], which caps the maximum number of
/// concurrent directory reads to `rayon threads / 2` or `8`, whichever's
/// smaller. This is a good middle ground between [`DirConcurrency::Single`] and
/// [`DirConcurrency::Max`], but may not always be the best choice.
///
/// Any degree of parallelization runs the risk of tripping the user's `ulimit`
/// system restrictions (by e.g. opening too many concurrent path handles). If
/// this happens, paths might be intermittently skipped due to "unreadability"
/// or the search may fail entirely.
///
/// If you anticipate a large number of sub-paths or runtime environments with
/// low `ulimit` caps, [`DirConcurrency::Single`] should be used instead.
pub enum DirConcurrency {
	/// # One at a Time.
	///
	/// When using this option, the paths within a given directory will also be
	/// processed in serial.
	Single,

	/// # A Probably Okay Default.
	Sane,

	/// # CONSUME ALL DIRS EN MASSE.
	Max,

	/// # A Custom Value.
	Custom(NonZeroUsize),

	#[deprecated(since = "0.4.4", note = "use DirConcurrency::Custom instead")]
	/// # A Custom Value.
	Other(NonZeroUsize),
}

impl Default for DirConcurrency {
	fn default() -> Self { Self::Sane }
}

impl From<usize> for DirConcurrency {
	#[allow(unsafe_code)]
	fn from(src: usize) -> Self {
		match src {
			0 => Self::Max,
			1 => Self::Single,
			// Safety: zero is checked above.
			n => Self::Custom(unsafe { NonZeroUsize::new_unchecked(n) }),
		}
	}
}

impl From<NonZeroUsize> for DirConcurrency {
	#[inline]
	fn from(src: NonZeroUsize) -> Self { Self::Custom(src) }
}

impl From<DirConcurrency> for usize {
	#[allow(deprecated)] // We deprecated it!
	fn from(src: DirConcurrency) -> Self {
		match src {
			DirConcurrency::Sane => match rayon::current_num_threads() {
				0..=3 => 1,
				n => Self::min(n.wrapping_div(2), 8),
			},
			DirConcurrency::Single => 1,
			DirConcurrency::Max => Self::MAX,
			DirConcurrency::Custom(n) | DirConcurrency::Other(n) => n.get(),
		}
	}
}
