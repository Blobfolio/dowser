/*!
# Dowser: Parallelism
*/

use std::num::NonZeroUsize;



#[derive(Debug, Clone, Copy)]
/// # Directory Concurrency.
///
/// This enum determines if and how many directories [`Dowser`] should try to
/// read in parallel, which is configured via [`Dowser::with_dir_concurrency`].
///
/// The default is [`DirConcurrency::Sane`], which caps the maximum number of
/// concurrent directory reads to `rayon threads - 1` or `8`, whichever's
/// smaller. This is a good middle ground between [`DirConcurrency::Single`] and
/// [`DirConcurrency::Max`].
///
/// If you anticipate there being very few paths of any kind, the serial
/// [`DirConcurrency::Single`] option might actually prove faster.
///
/// Conversely, if you expect _a lot_ of directories, [`DirConcurrency::Max`] is
/// likely the best strategy. **However be careful** with this choice. If the user's
/// `ulimit` is set too low, paths might be silently skipped due to intermittent
/// unreadability.
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
	fn from(src: usize) -> Self {
		match src {
			0 => Self::Max,
			1 => Self::Single,
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
				0..=2 => 1,
				n => Self::min(n - 1, 8),
			},
			DirConcurrency::Single => 1,
			DirConcurrency::Max => Self::MAX,
			DirConcurrency::Custom(n) | DirConcurrency::Other(n) => n.get(),
		}
	}
}
