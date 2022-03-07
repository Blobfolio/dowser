/*!
# Dowser: Utility Methods.
*/

use std::path::Path;



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
	use std::os::unix::ffi::OsStrExt;
	p.as_os_str().as_bytes()
}
