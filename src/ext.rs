/*!
# Dowser: Extension
*/

use dactyl::{
	NiceU16,
	NiceU32,
};
use std::{
	hash::{
		Hash,
		Hasher,
	},
	path::Path,
};

#[cfg(unix)]
use std::os::unix::ffi::OsStrExt;



#[cfg(unix)]
/// # Path to Bytes.
///
/// Convert a path to a slice.
macro_rules! path_slice {
	($path:ident) => ($path.as_ref().as_os_str().as_bytes());
}

#[cfg(not(unix))]
/// # Path to Bytes.
///
/// Convert a path to a slice. On Windows this may not be strictly correct,
/// but hopefully good enough to match an extension.
macro_rules! path_slice {
	($path:ident) => ($path.as_ref().to_string_lossy().as_bytes());
}



#[derive(Debug, Clone, Copy)]
/// # Extension.
///
/// This enum can be used to efficiently check a file path's extension case-
/// insensitively against a hard-coded reference extension. It is likely
/// overkill in most situations, but if you're looking to optimize the
/// filtering of large path lists, this can turn those painful nanosecond
/// operations into pleasant picosecond ones!
///
/// The magic is largely down to storing values as `u16` or `u32` integers and
/// comparing those (rather than byte slices or `OsStr`), and not messing
/// around with the path `Components` iterator. (Note, this is done using the
/// safe `u*::from_le_bytes()` methods rather than casting chicanery.)
///
/// At the moment, only extensions sized between 2-4 bytes are supported as
/// those sizes are the most common and also translate perfectly to primitives,
/// but larger values may be added in the future.
///
/// ## Reference Constructors.
///
/// A "reference" extension is one known to you ahead of time, i.e. what you're
/// looking for. These can be constructed using the constant [`Extension::new2`],
/// [`Extension::new3`], and [`Extension::new4`] methods.
///
/// Because these are "known" values, no logical validation is performed. If
/// you do something silly like mix case or type them incorrectly, equality
/// tests will fail. You'd only be hurting yourself!
///
/// ```
/// use dowser::Extension;
///
/// const EXT2: Extension = Extension::new2(*b"gz");
/// const EXT3: Extension = Extension::new3(*b"png");
/// const EXT4: Extension = Extension::new4(*b"html");
/// ```
///
/// The main idea is you'll pre-compute these values and compare unknown
/// runtime values against them later.
///
/// ## Runtime Constructors.
///
/// A "runtime" extension, for lack of a better adjective, is a value you
/// don't know ahead of time, e.g. from a user-supplied path. These can be
/// constructed using the [`Extension::try_from2`], [`Extension::try_from3`],
/// and [`Extension::try_from4`] methods, which accept any `AsRef<Path>`
/// argument.
///
/// The method you choose should match the length you're looking for. For
/// example, if you're hoping for a PNG, use [`Extension::try_from3`].
///
/// ```
/// use dowser::Extension;
///
/// const EXT3: Extension = Extension::new3(*b"png");
/// assert_eq!(Extension::try_from3("/path/to/IMAGE.PNG"), Some(EXT3));
/// assert_eq!(Extension::try_from3("/path/to/doc.html"), None);
/// ```
///
/// ## Examples
///
/// To filter a list of image paths with the standard library — say, matching
/// PNGs — you would do something like:
///
/// ```no_run
/// use std::ffi::OsStr;
/// use std::path::PathBuf;
///
/// // Imagine this is much longer…
/// let paths = vec![PathBuf::from("/path/to/image.png")];
///
/// paths.iter()
///     .filter(|p| p.extension()
///         .map_or(false, |e| e.eq_ignore_ascii_case(OsStr::new("png")))
///     )
///     .for_each(|p| todo!());
/// ```
///
/// Using [`Extension`] instead, the same operation looks like:
///
/// ```no_run
/// use dowser::Extension;
/// use std::path::PathBuf;
///
/// // Imagine this is much longer…
/// let paths = vec![PathBuf::from("/path/to/image.png")];
///
/// // The reference extension.
/// const EXT: Extension = Extension::new3(*b"png");
///
/// paths.iter()
///     .filter(|p| Extension::try_from3(p).map_or(false, |e| e == EXT))
///     .for_each(|p| todo!());
/// ```
pub enum Extension {
	/// # 2-char Extension.
	///
	/// Like `.gz`.
	Ext2(u16),

	/// # 3-char Extension.
	///
	/// Like `.png`.
	Ext3(u32),

	/// # 4-char Extension.
	///
	/// Like `.html`.
	Ext4(u32),
}

impl Eq for Extension {}

impl Hash for Extension {
	fn hash<H: Hasher>(&self, state: &mut H) {
		match *self {
			Self::Ext2(n) => { state.write_u16(n); },
			Self::Ext3(n) | Self::Ext4(n) => { state.write_u32(n); },
		}
	}
}

impl PartialEq for Extension {
	#[inline]
	fn eq(&self, other: &Self) -> bool {
		match (*self, *other) {
			(Self::Ext2(e1), Self::Ext2(e2)) => e1 == e2,
			(Self::Ext3(e1), Self::Ext3(e2)) |
			(Self::Ext4(e1), Self::Ext4(e2)) => e1 == e2,
			_ => false,
		}
	}
}

impl<P> PartialEq<P> for Extension
where P: AsRef<Path> {
	/// # Path Equality.
	///
	/// When there's just one extension and one path to check, you can compare
	/// them directly (extension first).
	///
	/// ## Examples
	///
	/// ```
	/// use dowser::Extension;
	///
	/// const MY_EXT: Extension = Extension::new4(*b"html");
	///
	/// assert_eq!(MY_EXT, "/path/to/index.html");
	/// assert_ne!(MY_EXT, "/path/to/image.jpeg");
	/// ```
	fn eq(&self, other: &P) -> bool {
		match self {
			Self::Ext2(_) => Self::try_from2(other),
			Self::Ext3(_) => Self::try_from3(other),
			Self::Ext4(_) => Self::try_from4(other),
		}
			.map_or(false, |e| e.eq(self))
	}
}

/// # Unchecked Instantiation.
impl Extension {
	#[must_use]
	/// # New Unchecked (2).
	///
	/// Create a new [`Extension`], unchecked, from two bytes, e.g. `*b"gz"`.
	/// This should be lowercase and not include a period.
	///
	/// This method is intended for known values that you want to check
	/// unknown values against. Sanity-checking is traded for performance, but
	/// you're only hurting yourself if you misuse it.
	///
	/// For compile-time generation, see [`Extension::codegen`].
	///
	/// ## Examples
	///
	/// ```
	/// use dowser::Extension;
	/// const MY_EXT: Extension = Extension::new2(*b"gz");
	/// ```
	pub const fn new2(src: [u8; 2]) -> Self {
		Self::Ext2(u16::from_le_bytes(src))
	}

	#[must_use]
	/// # New Unchecked (3).
	///
	/// Create a new [`Extension`], unchecked, from three bytes, e.g. `*b"gif"`.
	/// This should be lowercase and not include a period.
	///
	/// This method is intended for known values that you want to check
	/// unknown values against. Sanity-checking is traded for performance, but
	/// you're only hurting yourself if you misuse it.
	///
	/// For compile-time generation, see [`Extension::codegen`].
	///
	/// ## Examples
	///
	/// ```
	/// use dowser::Extension;
	/// const MY_EXT: Extension = Extension::new3(*b"gif");
	/// ```
	pub const fn new3(src: [u8; 3]) -> Self {
		Self::Ext3(u32::from_le_bytes([b'.', src[0], src[1], src[2]]))
	}

	#[must_use]
	/// # New Unchecked (4).
	///
	/// Create a new [`Extension`], unchecked, from four bytes, e.g. `*b"html"`.
	/// This should be lowercase and not include a period.
	///
	/// This method is intended for known values that you want to check
	/// unknown values against. Sanity-checking is traded for performance, but
	/// you're only hurting yourself if you misuse it.
	///
	/// For compile-time generation, see [`Extension::codegen`].
	///
	/// ## Examples
	///
	/// ```
	/// use dowser::Extension;
	/// const MY_EXT: Extension = Extension::new4(*b"html");
	/// ```
	pub const fn new4(src: [u8; 4]) -> Self {
		Self::Ext4(u32::from_le_bytes(src))
	}
}

/// # From Paths.
impl Extension {
	#[must_use]
	/// # Try From Path (2).
	///
	/// This method is used to (try to) pull a 2-byte extension from a file
	/// path. This requires that the path be at least 4 bytes, with anything
	/// but a forward/backward slash at `[len - 4]` and a dot at `[len - 3]`.
	///
	/// If successful, it will return an [`Extension::Ext2`] that can be
	/// compared against your reference [`Extension`]. Casing will be fixed
	/// automatically.
	///
	/// ## Examples
	///
	/// ```
	/// use dowser::Extension;
	///
	/// const MY_EXT: Extension = Extension::new2(*b"gz");
	/// assert_eq!(Extension::try_from2("/path/to/file.gz"), Some(MY_EXT));
	/// assert_eq!(Extension::try_from2("/path/to/file.GZ"), Some(MY_EXT));
	///
	/// assert_eq!(Extension::try_from2("/path/to/file.png"), None);
	/// assert_ne!(Extension::try_from2("/path/to/file.br"), Some(MY_EXT));
	/// ```
	pub fn try_from2<P>(path: P) -> Option<Self>
	where P: AsRef<Path> {
		Self::slice_ext2(path_slice!(path))
	}

	#[must_use]
	/// # Try From Path (3).
	///
	/// This method is used to (try to) pull a 3-byte extension from a file
	/// path. This requires that the path be at least 5 bytes, with anything
	/// but a forward/backward slash at `[len - 5]` and a dot at `[len - 4]`.
	///
	/// If successful, it will return an [`Extension::Ext3`] that can be
	/// compared against your reference [`Extension`]. Casing will be fixed
	/// automatically.
	///
	/// ## Examples
	///
	/// ```
	/// use dowser::Extension;
	///
	/// const MY_EXT: Extension = Extension::new3(*b"png");
	/// assert_eq!(Extension::try_from3("/path/to/file.png"), Some(MY_EXT));
	/// assert_eq!(Extension::try_from3("/path/to/FILE.PNG"), Some(MY_EXT));
	///
	/// assert_eq!(Extension::try_from3("/path/to/file.html"), None);
	/// assert_ne!(Extension::try_from3("/path/to/file.jpg"), Some(MY_EXT));
	/// ```
	pub fn try_from3<P>(path: P) -> Option<Self>
	where P: AsRef<Path> {
		Self::slice_ext3(path_slice!(path))
	}

	#[must_use]
	/// # Try From Path (4).
	///
	/// This method is used to (try to) pull a 4-byte extension from a file
	/// path. This requires that the path be at least 6 bytes, with anything
	/// but a forward/backward slash at `[len - 6]` and a dot at `[len - 5]`.
	///
	/// If successful, it will return an [`Extension::Ext4`] that can be
	/// compared against your reference [`Extension`]. Casing will be fixed
	/// automatically.
	///
	/// ## Examples
	///
	/// ```
	/// use dowser::Extension;
	///
	/// const MY_EXT: Extension = Extension::new4(*b"html");
	/// assert_eq!(Extension::try_from4("/path/to/file.html"), Some(MY_EXT));
	/// assert_eq!(Extension::try_from4("/path/to/FILE.HTML"), Some(MY_EXT));
	///
	/// assert_eq!(Extension::try_from4("/path/to/file.png"), None);
	/// assert_ne!(Extension::try_from4("/path/to/file.xhtm"), Some(MY_EXT));
	/// ```
	pub fn try_from4<P>(path: P) -> Option<Self>
	where P: AsRef<Path> {
		Self::slice_ext4(path_slice!(path))
	}
}

/// # From Slices.
impl Extension {
	#[inline]
	#[must_use]
	/// # Extension Slice (2).
	///
	/// This method is used to (try to) pull a 2-byte extension from a file
	/// path in slice form. This requires that the path be at least 4 bytes,
	/// with anything but a forward/backward slash at `[len - 4]` and a dot at
	/// `[len - 3]`.
	///
	/// If successful, it will return an [`Extension::Ext2`] that can be
	/// compared against your reference [`Extension`]. Casing will be fixed
	/// automatically.
	///
	/// ## Examples
	///
	/// ```
	/// use dowser::Extension;
	///
	/// const MY_EXT: Extension = Extension::new2(*b"gz");
	/// assert_eq!(Extension::slice_ext2(b"/path/to/file.gz"), Some(MY_EXT));
	/// assert_eq!(Extension::slice_ext2(b"/path/to/file.GZ"), Some(MY_EXT));
	///
	/// assert_eq!(Extension::slice_ext2(b"/path/to/file.png"), None);
	/// assert_ne!(Extension::slice_ext2(b"/path/to/file.br"), Some(MY_EXT));
	/// ```
	pub const fn slice_ext2(path: &[u8]) -> Option<Self> {
		if let [.., x, b'.', a, b] = path {
			if valid_pre_dot(*x) {
				Some(Self::Ext2(u16::from_le_bytes([
					a.to_ascii_lowercase(),
					b.to_ascii_lowercase(),
				])))
			}
			else { None }
		}
		else { None }
	}

	#[inline]
	#[must_use]
	/// # Extension Slice (3).
	///
	/// This method is used to (try to) pull a 3-byte extension from a file
	/// path in slice form. This requires that the path be at least 5 bytes,
	/// with anything but a forward/backward slash at `[len - 5]` and a dot at
	/// `[len - 4]`.
	///
	/// If successful, it will return an [`Extension::Ext3`] that can be
	/// compared against your reference [`Extension`]. Casing will be fixed
	/// automatically.
	///
	/// ## Examples
	///
	/// ```
	/// use dowser::Extension;
	///
	/// const MY_EXT: Extension = Extension::new3(*b"png");
	/// assert_eq!(Extension::slice_ext3(b"/path/to/file.png"), Some(MY_EXT));
	/// assert_eq!(Extension::slice_ext3(b"/path/to/FILE.PNG"), Some(MY_EXT));
	///
	/// assert_eq!(Extension::slice_ext3(b"/path/to/file.html"), None);
	/// assert_ne!(Extension::slice_ext3(b"/path/to/file.jpg"), Some(MY_EXT));
	/// ```
	pub const fn slice_ext3(path: &[u8]) -> Option<Self> {
		if let [.., x, b'.', a, b, c] = path {
			if valid_pre_dot(*x) {
				Some(Self::Ext3(u32::from_le_bytes([
					b'.',
					a.to_ascii_lowercase(),
					b.to_ascii_lowercase(),
					c.to_ascii_lowercase(),
				])))
			}
			else { None }
		}
		else { None }
	}

	#[inline]
	#[must_use]
	/// # Extension Slice (4).
	///
	/// This method is used to (try to) pull a 4-byte extension from a file
	/// path in slice form. This requires that the path be at least 6 bytes,
	/// with anything but a forward/backward slash at `[len - 6]` and a dot at
	/// `[len - 5]`.
	///
	/// If successful, it will return an [`Extension::Ext4`] that can be
	/// compared against your reference [`Extension`]. Casing will be fixed
	/// automatically.
	///
	/// ## Examples
	///
	/// ```
	/// use dowser::Extension;
	///
	/// const MY_EXT: Extension = Extension::new4(*b"html");
	/// assert_eq!(Extension::slice_ext4(b"/path/to/file.html"), Some(MY_EXT));
	/// assert_eq!(Extension::slice_ext4(b"/path/to/FILE.HTML"), Some(MY_EXT));
	///
	/// assert_eq!(Extension::slice_ext4(b"/path/to/file.png"), None);
	/// assert_ne!(Extension::slice_ext4(b"/path/to/file.xhtm"), Some(MY_EXT));
	/// ```
	pub const fn slice_ext4(path: &[u8]) -> Option<Self> {
		if let [.., x, b'.', a, b, c, d] = path {
			if valid_pre_dot(*x) {
				Some(Self::Ext4(u32::from_le_bytes([
					a.to_ascii_lowercase(),
					b.to_ascii_lowercase(),
					c.to_ascii_lowercase(),
					d.to_ascii_lowercase(),
				])))
			}
			else { None }
		}
		else { None }
	}

	#[allow(unsafe_code)]
	#[must_use]
	/// # Slice Extension.
	///
	/// This returns the file extension portion of a path as a byte slice,
	/// similar to [`std::path::Path::extension`], but faster since it is dealing with
	/// straight bytes.
	///
	/// The extension is found by jumping to the last period, ensuring the byte
	/// _before_ that period is not a path separator, and that there are one or
	/// more bytes _after_ that period (none of which are path separators).
	///
	/// If the above are all good, a slice containing everything after that
	/// last period is returned.
	///
	/// ## Examples
	///
	/// ```
	/// use dowser::Extension;
	///
	/// // Uppercase in, uppercase out.
	/// assert_eq!(
	///     Extension::slice_ext(b"/path/to/IMAGE.JPEG"),
	///     Some(&b"JPEG"[..])
	/// );
	///
	/// // Lowercase in, lowercase out.
	/// assert_eq!(
	///     Extension::slice_ext(b"/path/to/file.docx"),
	///     Some(&b"docx"[..])
	/// );
	///
	/// // These are all bad, though:
	/// assert_eq!(
	///     Extension::slice_ext(b"/path/to/.htaccess"),
	///     None
	/// );
	/// assert_eq!(
	///     Extension::slice_ext(b"/path/to/"),
	///     None
	/// );
	/// assert_eq!(
	///     Extension::slice_ext(b"/path/to/file."),
	///     None
	/// );
	/// ```
	pub fn slice_ext(src: &[u8]) -> Option<&[u8]> {
		let dot = src.iter().rposition(|&b| matches!(b, b'.' | b'/' | b'\\'))?;

		if
			0 < dot &&
			dot + 1 < src.len() &&
			src[dot] == b'.' &&
			// Safety: we tested 0 < dot, so the subtraction won't overflow.
			valid_pre_dot(unsafe { *(src.get_unchecked(dot - 1)) })
		{
			Some(&src[dot + 1..])
		}
		else { None }
	}
}

/// # Codegen Helpers.
impl Extension {
	#[allow(clippy::needless_doctest_main)] // For demonstration!
	#[must_use]
	/// # Codegen Helper.
	///
	/// This _compile-time_ method can be used in a `build.rs` script to
	/// generate a pre-computed [`Extension`] value of any supported length
	/// (2-4 bytes).
	///
	/// Unlike the runtime methods, this will automatically fix case and period
	/// inconsistencies, but ideally you should still pass it just the letters,
	/// in lowercase, because you have the power to do so. Haha.
	///
	/// ## Examples
	///
	/// ```
	/// use dowser::Extension;
	///
	/// // This is what it looks like.
	/// assert_eq!(
	///     Extension::codegen(b"js"),
	///     "Extension::Ext2(29_546_u16)"
	/// );
	/// assert_eq!(
	///     Extension::codegen(b"jpg"),
	///     "Extension::Ext3(1_735_420_462_u32)"
	/// );
	/// assert_eq!(
	///     Extension::codegen(b"html"),
	///     "Extension::Ext4(1_819_112_552_u32)"
	/// );
	/// ```
	///
	/// In a typical `build.rs` workflow, you'd be building up a string of
	/// other code around it, and saving it all to a file, like:
	///
	/// ```no_run
	/// use dowser::Extension;
	/// use std::fs::File;
	/// use std::io::Write;
	/// use std::path::PathBuf;
	///
	/// fn main() {
	///     let out = format!(
	///         "const MY_EXT: Extension = {};",
	///         Extension::codegen(b"jpg")
	///     );
	///
	///     let out_path = PathBuf::from(std::env::var("OUT_DIR").unwrap())
	///         .join("compile-time-vars.rs");
	///     let mut f = File::create(out_path).unwrap();
	///     f.write_all(out.as_bytes()).unwrap();
	///     f.flush().unwrap();
	/// }
	///
	/// ```
	///
	/// Then in your main program, say `lib.rs`, you'd toss an `include!()` to
	/// that file to import the code _as code_, like:
	///
	/// ```no_run,ignore
	/// use dowser::Extension;
	///
	/// include!(concat!(env!("OUT_DIR"), "/compile-time-vars.rs"));
	/// ```
	///
	/// Et voilà, you've saved yourself a nanosecond of runtime effort! Haha.
	///
	/// ## Panics
	///
	/// This will panic if the extension (minus punctuation) is not 2-4 bytes
	/// or contains whitespace or path separators.
	pub fn codegen(mut src: &[u8]) -> String {
		// Jump past the last period, if any.
		if let Some(pos) = src.iter().rposition(|b| b'.'.eq(b)) {
			assert!(pos + 2 < src.len(), "Extensions must be 2-4 bytes (not including punctuation).");
			src = &src[pos + 1..];
		}

		// Make sure there is no whitespace.
		assert!(
			src.iter().all(|b| ! b.is_ascii_whitespace() && b'/'.ne(b) && b'\\'.ne(b)),
			"Extensions cannot contain whitespace or path separators."
		);

		match src.len() {
			2 => [
				"Extension::Ext2(",
				NiceU16::with_separator(
					u16::from_le_bytes([
						src[0].to_ascii_lowercase(),
						src[1].to_ascii_lowercase(),
					]),
					b'_',
				).as_str(),
				"_u16)",
			].concat(),
			3 => [
				"Extension::Ext3(",
				NiceU32::with_separator(
					u32::from_le_bytes([
						b'.',
						src[0].to_ascii_lowercase(),
						src[1].to_ascii_lowercase(),
						src[2].to_ascii_lowercase(),
					]),
					b'_',
				).as_str(),
				"_u32)",
			].concat(),
			4 => [
				"Extension::Ext4(",
				NiceU32::with_separator(
					u32::from_le_bytes([
						src[0].to_ascii_lowercase(),
						src[1].to_ascii_lowercase(),
						src[2].to_ascii_lowercase(),
						src[3].to_ascii_lowercase(),
					]),
					b'_',
				).as_str(),
				"_u32)",
			].concat(),
			_ => panic!("Extensions must be 2-4 bytes (not including punctuation)."),
		}
	}
}



#[inline]
/// # Valid Pre-Dot Character.
///
/// This simply checks that a byte is not a path separator.
const fn valid_pre_dot(b: u8) -> bool { ! matches!(b, b'/' | b'\\') }




#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn t_codegen() {
		assert_eq!(Extension::codegen(b"js"), "Extension::Ext2(29_546_u16)");
		assert_eq!(Extension::codegen(b"JS"), "Extension::Ext2(29_546_u16)");
		assert_eq!(Extension::codegen(b"/path/to/file.JS"), "Extension::Ext2(29_546_u16)");

		assert_eq!(Extension::codegen(b"jpg"), "Extension::Ext3(1_735_420_462_u32)");
		assert_eq!(Extension::codegen(b"JPG"), "Extension::Ext3(1_735_420_462_u32)");
		assert_eq!(Extension::codegen(b".jpg"), "Extension::Ext3(1_735_420_462_u32)");

		assert_eq!(Extension::codegen(b"html"), "Extension::Ext4(1_819_112_552_u32)");
		assert_eq!(Extension::codegen(b"htML"), "Extension::Ext4(1_819_112_552_u32)");
		assert_eq!(Extension::codegen(b"index.html"), "Extension::Ext4(1_819_112_552_u32)");
	}

	#[test]
	#[should_panic]
	fn t_codegen_bad1() { let _res = Extension::codegen(b""); }

	#[test]
	#[should_panic]
	fn t_codegen_bad2() { let _res = Extension::codegen(b"xhtml"); }

	#[test]
	#[should_panic]
	fn t_codegen_bad3() { let _res = Extension::codegen(b"x./html"); }
}
