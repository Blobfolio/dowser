/*!
# Dowser: Utility Methods.
*/

use std::{
	os::unix::ffi::OsStrExt,
	path::Path,
};



#[cfg_attr(feature = "docsrs", doc(cfg(unix)))]
#[must_use]
#[inline]
/// # Path to Bytes.
///
/// Like it says; convert an `&Path` to an `&[u8]`.
///
/// ## Examples
///
/// ```no_run
/// use std::path::PathBuf;
/// let path = dowser::utility::path_as_bytes(&PathBuf::from("/path/to/file.jpg"));
/// ```
pub fn path_as_bytes(p: &Path) -> &[u8] {
	p.as_os_str().as_bytes()
}
