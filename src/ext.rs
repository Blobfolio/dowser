/*!
# Dowser: File Extension
*/

use std::{
	fmt,
	hash,
	path::Path,
};

#[cfg(unix)]
use std::os::unix::ffi::OsStrExt;



/// # ASCII Lower Bit.
///
/// An ASCII letter is uppercase if this bit is missing, lowercase if not.
const ASCII_CASE_MASK: u8 = 0b0010_0000;

/// # Max Extension Length.
///
/// Extensions with lengths between `1..=8` are supported.
const EXT_SIZE: usize = 8;

/// # Zeroed Buffer.
///
/// The `Extension` buffer is zero-padded, so a zeroed buffer is as good a
/// place as any to start.
const ZEROES: [u8; EXT_SIZE] = [0_u8; EXT_SIZE];



/// # Helper: Anything But Slashes.
///
/// It would be hard to spot typos in this pattern. The macro acts like a kind
/// of variable so we don't have to worry about typing it correctly.
macro_rules! notslash {
	() => ( 0..=46 | 48..=91 | 93..=255 );
}



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
/// Convert a path to a slice… less well.
macro_rules! path_slice {
	($path:ident) => ($path.as_ref().to_string_lossy().as_bytes());
}



#[derive(Clone, Copy, Eq, Ord, PartialEq, PartialOrd)]
/// # Case-Insensitive File Extension.
///
/// This struct can be used to case-insensitively parse and compare "normal"
/// file path extensions faster and more efficiently than native [`Path`]-based
/// alternatives.
///
/// To keep its size down and speed up, `Extension` ignores the edge cases of
/// infinity, focusing solely on values composed of ASCII alphanumerics, `!`,
/// `#`, `+`, `-`, and/or `_`, with lengths between `1..=8`.
///
/// In practice, the vast majority of file extensions were created with even
/// constrainier constraints in mind anyway, so pose no particular problems for
/// `Extension`.
///
/// There are, however, a few notable outliers — `"webmanifest"` and `"🔥"` to
/// name two — for which a more open-ended solution like [`Path::extension`]
/// would be required.
///
/// ## Examples
///
/// When matching paths against a _single_ target extension, the
/// [`Extension::matches_path`] helper is choice.
///
/// ```
/// use dowser::{Dowser, Extension};
///
/// // Target extensions should be constant.
/// const EXT: Extension = Extension::new("avi").unwrap();
///
/// // Loop through a bunch of paths.
/// for p in Dowser::from("./videos") {
///     if EXT.matches_path(&p) {
///         // Do something with the AVI…
///     }
/// }
/// ```
///
/// When matching against multiple target extensions, it will usually be more
/// efficient to explicitly parse the path's extension for comparison instead.
///
/// ```
/// use dowser::Extension;
///
/// // Same as before, target extensions should be constant.
/// const EXT_FLAC: Extension = Extension::new("flac").unwrap();
/// const EXT_MP3: Extension =  Extension::new("mp3").unwrap();
/// const EXT_WAV: Extension =  Extension::new("wav").unwrap();
///
/// // FYI the path loop doesn't have to come from Dowser.
/// let mut lossless = Vec::new();
/// let mut lossy = Vec::new();
/// if let Ok(rd) = std::fs::read_dir("/usr/share") {
///     for e in rd.filter_map(Result::ok) {
///         if e.file_type().is_ok_and(|f| ! f.is_dir()) {
///             // Finally, a path! Haha.
///             let path = e.path();
///
///             // Extract the extension, then check for matches.
///             match Extension::from_path(&path) {
///                 Some(EXT_FLAC | EXT_WAV) => {
///                     lossless.push(path);
///                 },
///                 Some(EXT_MP3) => {
///                     lossy.push(path);
///                 },
///                 // Something else. Let it go.
///                 _ => {},
///             }
///         }
///     }
/// }
/// ```
///
/// [`Extension`] can be useful for grouping too, since it's small and `Copy`.
///
/// ```
/// use std::collections::{BTreeMap, BTreeSet, HashMap};
/// use std::path::PathBuf;
/// use dowser::{Dowser, Extension};
///
/// // Groupings can leverage Hash or Ord, dealer's choice.
/// let mut ordered = BTreeMap::<Extension, BTreeSet<PathBuf>>::new();
/// let mut hashed =  HashMap::<Extension, BTreeSet<PathBuf>>::new();
/// # let mut limit = 0;
/// for p in Dowser::from("/usr/share") {
///     if let Some(ext) = Extension::from_path(&p) {
///         ordered.entry(ext).or_default().insert(p.clone());
///         hashed.entry(ext).or_default().insert(p);
///     }
///     # limit += 1;
///     # if 100 < limit { break; }
/// }
///
/// // The keys may not line up, but the counts will come out the same
/// // either way.
/// assert_eq!(ordered.len(), hashed.len());
/// ```
///
/// Speaking of ordering, [`Extension`]s sort by length, then value.
///
/// ```
/// use dowser::Extension;
///
/// const SORTED: [Extension; 7] = [
///     Extension::new("7z").unwrap(),
///     Extension::new("gz").unwrap(),
///     Extension::new("xz").unwrap(),
///
///     Extension::new("bz2").unwrap(),
///     Extension::new("zip").unwrap(),
///     Extension::new("zst").unwrap(),
///
///     Extension::new("svgz").unwrap(),
/// ];
///
/// assert!(SORTED.is_sorted()); // Told you!
/// ```
pub struct Extension([u8; EXT_SIZE]);

impl AsRef<[u8]> for Extension {
	#[inline]
	fn as_ref(&self) -> &[u8] { self.as_bytes() }
}

impl AsRef<str> for Extension {
	#[inline]
	fn as_ref(&self) -> &str { self.as_str() }
}

impl fmt::Debug for Extension {
	#[inline]
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "Extension({self})")
	}
}

impl fmt::Display for Extension {
	#[inline]
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		<str as fmt::Display>::fmt(self.as_str(), f)
	}
}

impl hash::Hash for Extension {
	#[inline]
	/// # Hash.
	///
	/// The `Extension` data is hashed en masse via a single call to
	/// [`Hasher::write_u64`](std::hash::Hasher::write_u64).
	fn hash<H: hash::Hasher>(&self, state: &mut H) {
		state.write_u64(u64::from_be_bytes(self.0));
	}
}

impl Extension {
	#[inline]
	#[must_use]
	/// # New Extension.
	///
	/// Create a new [`Extension`] from a string slice representation of same,
	/// e.g. `"jpg"`.
	///
	/// [`Extension`]s are case-insensitive, but must be between `1..=8` bytes
	/// in length and contain only ASCII alphanumerics, `!`, `#`, `+`, `-`,
	/// and/or `_`, or `None` will be returned instead.
	///
	/// ## Examples
	///
	/// ```
	/// use dowser::Extension;
	///
	/// // For best results, stick with const/literals so the checks and
	/// // case-tweaking involved with setup can be handled at compile-time.
	/// const JPG: Extension = Extension::new("jpg").unwrap();
	/// let jpeg = const { Extension::new("jpeg").unwrap() };
	///
	/// // More programmatic implementations will still work, but may incur a
	/// // runtime setup cost, and won't be usable in righthand `matches!`
	/// // contexts.
	/// let ext = "png";
	/// let png = Extension::new(ext).unwrap();
	///
	/// // Extensions are always stored lower case, but nothing will break if
	/// // you accidentally pass UPPER.
	/// assert_eq!(
	///     Extension::new("html"),
	///     Extension::new("HTML"),
	/// );
	///
	/// // Wrong is wrong, though.
	/// for i in [
	///     "cpp*",        // Asterisk.
	///     "gif ",        // Space.
	///     "1.gz",        // Dot.
	///     "ÖöÖö",        // Not ASCII.
	///     "🍆",          // Even less ASCII.
	///     "",            // Too short.
	///     "webmanifest", // Too long.
	/// ] {
	///     assert!(Extension::new(i).is_none(), "{i:?}");
	/// }
	/// ```
	pub const fn new(src: &str) -> Option<Self> { Self::new_slice(src.as_bytes()) }

	#[doc(hidden)]
	#[inline]
	#[must_use]
	/// # New Extension (From Slice).
	///
	/// Same as [`Extension::new`], but for extensions represented as byte
	/// slices.
	pub const fn new_slice(mut src: &[u8]) -> Option<Self> {
		if ! src.is_empty() && src.len() <= EXT_SIZE {
			// Optimistically copy bytes to the buffer, right to left.
			let mut dst = ZEROES;
			let mut idx = EXT_SIZE;
			while let [ rest @ .., n ] = src && let Some(n) = sanitize_byte(*n) {
				if idx == 0 { return None; } // Too long!
				idx -= 1;
				dst[idx] = n;
				src = rest;
			}

			// If we wrote something and everything, we're good!
			if idx < EXT_SIZE && src.is_empty() { Some(Self(dst)) }
			else { None }
		}
		else { None }
	}

	#[inline]
	#[must_use]
	/// # New Extension (From Path).
	///
	/// Try to parse and return a new [`Extension`] from a file `Path`.
	///
	/// Unlike [`Path::extension`], this will return `None` if the extension
	/// is not `1..=8` bytes in length, or contains anything other than ASCII
	/// alphanumerics, `!`, `#`, `+`, `-`, or `_`.
	///
	/// ## Examples
	///
	/// ```
	/// use dowser::Extension;
	/// use std::path::Path;
	///
	/// const HTM: Extension =  Extension::new("htm").unwrap();
	/// const HTML: Extension = Extension::new("html").unwrap();
	///
	/// // Random path collection.
	/// let mut files: Vec<&Path> = vec![
	///     Path::new("/path/to/404.htm"),
	///     Path::new("/path/to/about.html"),
	///     Path::new("/path/to/about.JPG"),
	///     Path::new("/path/to/contact.html"),
	///     Path::new("/path/to/image.jpg"),
	///     Path::new("/path/to/index.html"),
	///     Path::new("/path/to/logo.png"),
	/// ];
	///
	/// // Reduce to only those with JPE?G/PNG extensions.
	/// files.retain(|p| matches!(
	///     Extension::from_path(p),
	///     Some(HTM | HTML),
	/// ));
	///
	/// // The results:
	/// assert_eq!(
	///     files,
	///     [
	///         Path::new("/path/to/404.htm"),
	///         Path::new("/path/to/about.html"),
	///         Path::new("/path/to/contact.html"),
	///         Path::new("/path/to/index.html"),
	///     ],
	/// );
	/// ```
	pub fn from_path<P: AsRef<Path>>(src: P) -> Option<Self> {
		Self::from_path_slice(path_slice!(src))
	}

	#[inline]
	#[must_use]
	/// # New Extension (From Path Slice).
	///
	/// Same as [`Extension::from_path`], but for paths represented as byte
	/// slices.
	pub const fn from_path_slice(mut src: &[u8]) -> Option<Self> {
		// Optimistically copy (valid) bytes to the buffer, right to left.
		let mut dst = ZEROES;
		let mut idx = EXT_SIZE;
		while let [ rest @ .., n ] = src && let Some(n) = sanitize_byte(*n) {
			if idx == 0 { return None; }
			idx -= 1;
			dst[idx] = n;
			src = rest;
		}

		// If we wrote something and the remainder ends with a not-slash and
		// dot, we're good!
		if idx < EXT_SIZE && matches!(src, [ .., notslash!(), b'.' ]) { Some(Self(dst)) }
		else { None }
	}
}

impl Extension {
	#[inline]
	#[must_use]
	/// # As Byte Slice.
	///
	/// Return the extension as a (lower case) byte slice.
	///
	/// ## Examples
	///
	/// ```
	/// use dowser::Extension;
	///
	/// assert_eq!(
	///     Extension::new("x_b").unwrap().as_bytes(),
	///     b"x_b",
	/// );
	/// assert_eq!(
	///     Extension::new("X_B").unwrap().as_bytes(),
	///     b"x_b",
	/// );
	/// ```
	pub const fn as_bytes(&self) -> &[u8] {
		// The buffer is zero-padded, so we just need to chop those off
		// before returning.
		let mut out = self.0.as_slice();
		while let [ 0, rest @ .. ] = out { out = rest; }
		out
	}

	#[must_use]
	/// # As String Slice.
	///
	/// Return the extension as a (lower case) string slice.
	///
	/// ## Examples
	///
	/// ```
	/// use dowser::Extension;
	///
	/// assert_eq!(
	///     Extension::new("c++").unwrap().as_str(),
	///     "c++",
	/// );
	/// assert_eq!(
	///     Extension::new("C++").unwrap().as_str(),
	///     "c++",
	/// );
	/// ```
	pub const fn as_str(&self) -> &str {
		// Safety: the inner buffer is ASCII, even the unused bytes.
		let Ok(out) = std::str::from_utf8(self.as_bytes()) else { unreachable!(); };
		out
	}

	#[must_use]
	/// # Is Empty?
	///
	/// This should always return false.
	pub const fn is_empty(self) -> bool { self.0[EXT_SIZE - 1] == 0 }

	#[must_use]
	/// # Length.
	///
	/// Return the length of the extension.
	///
	/// ## Examples
	///
	/// ```
	/// use dowser::Extension;
	///
	/// assert_eq!(
	///     Extension::new("c").unwrap().len(),
	///     1,
	/// );
	/// assert_eq!(
	///     Extension::new("h!").unwrap().len(),
	///     2,
	/// );
	/// assert_eq!(
	///     Extension::new("e##").unwrap().len(),
	///     3,
	/// );
	/// assert_eq!(
	///     Extension::new("html").unwrap().len(),
	///     4,
	/// );
	/// assert_eq!(
	///     Extension::new("xhtml").unwrap().len(),
	///     5,
	/// );
	/// assert_eq!(
	///     Extension::new("n-gage").unwrap().len(),
	///     6,
	/// );
	/// assert_eq!(
	///     Extension::new("geojson").unwrap().len(),
	///     7,
	/// );
	/// assert_eq!(
	///     Extension::new("manifest").unwrap().len(),
	///     8,
	/// );
	/// ```
	pub const fn len(self) -> usize { self.as_bytes().len() }
}

impl Extension {
	#[inline]
	#[must_use]
	/// # Path Has Matching Extension?
	///
	/// Returns `true` if the path ends with the extension.
	///
	/// ## Examples
	///
	/// ```
	/// use dowser::Extension;
	///
	/// const TXT: Extension = Extension::new("txt").unwrap();
	///
	/// // Paths can be relative or absolute.
	/// assert!(TXT.matches_path("/usr/share/plain.txt"));
	/// assert!(TXT.matches_path("../docs/plain.txt"));
	/// assert!(TXT.matches_path("./plain.txt"));
	/// assert!(TXT.matches_path("plain.txt"));
	///
	/// // Case doesn't matter.
	/// assert!(TXT.matches_path("PLAIN.TXT"));
	///
	/// // Unicode (outside the extension) won't break anything.
	/// assert!(TXT.matches_path("♥.txt"));
	///
	/// // Nor will extra dots in the middle.
	/// assert!(TXT.matches_path("untitled.fanfic.2025.txt"));
	///
	/// // Wrong is wrong, though.
	/// assert!(! TXT.matches_path("txt"));         // No stem, no extension.
	/// assert!(! TXT.matches_path(".txt"));
	/// assert!(! TXT.matches_path("./txt"));
	/// assert!(! TXT.matches_path("file.txt "));   // Trailing space.
	/// assert!(! TXT.matches_path("file.txt.gz")); // Last extension wins.
	/// assert!(! TXT.matches_path("file.rtf"));    // Rich people problems.
	/// assert!(! TXT.matches_path("file.🕱"));      // Looks scary!
	/// ```
	///
	/// If matching path(s) against two or more extensions, it can be more
	/// efficient to do something like this instead:
	///
	/// ```
	/// # use dowser::Extension;
	/// const EXT_BR: Extension = Extension::new("br").unwrap();
	/// const EXT_GZ: Extension = Extension::new("gz").unwrap();
	/// const EXT_ZST: Extension = Extension::new("zst").unwrap();
	///
	/// let files = [
	///     "src/index.html",
	///     "src/index.html.br",
	///     "src/index.html.gz",
	///     "src/index.html.zst",
	/// ];
	///
	/// for file in files {
	///     if matches!(
	///         Extension::from_path(file),
	///         Some(EXT_BR | EXT_GZ | EXT_ZST),
	///     ) {
	///         // Happy times.
	/// # assert!(file != "src/index.html");
	///     }
	/// }
	/// ```
	pub fn matches_path<P: AsRef<Path>>(self, path: P) -> bool {
		self.matches_path_slice(path_slice!(path))
	}

	#[inline]
	#[must_use]
	/// # Path Has Matching Extension?
	///
	/// Same as [`Extension::matches_path`], but for paths represented as byte
	/// slices.
	pub const fn matches_path_slice(self, path: &[u8]) -> bool {
		/// # Check Byte.
		const fn check_byte(us: u8, them: u8) -> bool {
			if let Some(them) = sanitize_byte(them) { them == us }
			else { false }
		}

		// Split the path as if it had an extension the same size as the one
		// we have stored.
		let mut ext1 = self.as_bytes();
		if ext1.len() + 2 <= path.len() {
			let (path, mut ext2) = path.split_at(path.len() - ext1.len());

			// Shrink the extensions on each matching byte.
			while let [ a, rest1 @ .. ] = ext1 && let [ b, rest2 @ .. ] = ext2 && check_byte(*a, *b) {
				ext1 = rest1;
				ext2 = rest2;
			}

			// They're equal if we shrank all the way to zero, and the
			// remainder of the path ends with a not-slash and dot.
			ext1.is_empty() && matches!(path, [ .., notslash!(), b'.' ])
		}
		// Lengths didn't work out.
		else { false }
	}
}



#[inline(always)]
#[expect(clippy::inline_always, reason = "Foundational.")]
/// # Verify Byte.
///
/// Checks the byte is ASCII alphanumeric, `!`, `#`, `+`, `-`, or `_`, and
/// returns it, lowercasing if UPPER.
const fn sanitize_byte(src: u8) -> Option<u8> {
	match src {
		b'!' | b'#' | b'+' | b'-' | b'0'..=b'9' | b'_' | b'a'..=b'z' => Some(src),
		b'A'..=b'Z' => Some(src | ASCII_CASE_MASK),
		_ => None,
	}
}



#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn t_sanitize_byte() {
		for i in u8::MIN..=u8::MAX {
			// Alphanumerics should come back lowercase.
			if i.is_ascii_alphanumeric() || matches!(i, b'!' | b'#' | b'+' | b'-' | b'_') {
				assert_eq!(
					sanitize_byte(i),
					Some(i.to_ascii_lowercase()),
				);
			}
			// Everything else should be rejected.
			else { assert!(sanitize_byte(i).is_none()); }
		}
	}

	#[test]
	fn t_notslash() {
		for i in u8::MIN..=u8::MAX {
			// The notslash!() pattern is supposed to match everything but
			// forward and backward slashes.
			assert_eq!(
				i != b'/' && i != b'\\',
				matches!(i, notslash!()),
			);
		}
	}

	#[test]
	/// # Extension Sanity.
	///
	/// Most of these concepts are independently covered by the doctests, but
	/// not necessarily for each possible size.
	fn t_ext_len() {
		const RAW: [&str; EXT_SIZE] = [
			"1",
			"gz",
			"av1",
			"html",
			"vcard",
			"jsonld",
			"geojson",
			"manifest",
		];

		for i in RAW {
			let Some(ext) = Extension::new(i) else {
				panic!("Extension failed: {i:?}");
			};

			// Matching should work regardless of case.
			let mut file = format!("file.{i}");
			assert!(ext.matches_path(&file));
			assert_eq!(Extension::from_path(&file), Some(ext));

			file.make_ascii_uppercase();
			assert!(ext.matches_path(&file));
			assert_eq!(Extension::from_path(&file), Some(ext));

			// Change the extension, change the result.
			file.push('s');
			assert!(! ext.matches_path(&file));

			// Double-check string/length sanity.
			assert_eq!(ext.as_str(), i);
			assert_eq!(ext.to_string(), i);
			assert_eq!(ext.len(), i.len());

			// Extension from a capital should be the same.
			let Some(ext2) = Extension::new(&i.to_ascii_uppercase()) else {
				panic!("Extension failed: {:?}", i.to_ascii_uppercase());
			};
			assert_eq!(ext, ext2);
			assert_eq!(ext.as_str(), ext2.as_str());
		}

		// Sorting should factor length first.
		let mut exts = RAW.into_iter().filter_map(Extension::new).collect::<Vec<_>>();
		assert!(exts.is_sorted());

		// Then bytes (if the same size).
		exts.push(Extension::new("Z").unwrap());
		exts.push(Extension::new("c").unwrap());
		exts.push(Extension::new("C").unwrap());
		exts.sort();
		exts.dedup(); // Should kill one of the "c"s.

		// The "z" and other "c" should have floated up to the almost-top.
		assert_eq!(exts[0].as_str(), "1");
		assert_eq!(exts[1].as_str(), "c");
		assert_eq!(exts[2].as_str(), "z");
		assert_eq!(exts[3].as_str(), "gz");

		// If we remove them, we should be right back where we started.
		// (Sort/dedupe shouldn't have affected anything else.)
		exts.remove(2);
		exts.remove(1);
		assert!(exts.iter().map(|e| e.as_str()).eq(RAW.into_iter()));
	}

	#[test]
	/// # Realworld Extensions.
	///
	/// Make sure all the extensions featured in Wiki's article and the LotF
	/// database — with lengths 1-8 — are parseable by Extension.
	///
	/// [https://en.wikipedia.org/wiki/List_of_filename_extensions]
	/// [https://wordpress.org/plugins/blob-mimes/]
	fn t_ext_real() {
		let raw = std::fs::read_to_string("tests/extensions.txt").expect("Missing extensions.txt.");
		let all: Vec<&str> = raw.lines()
			.filter_map(|line| {
				let line = line.trim();
				if line.is_empty() { None }
				else { Some(line) }
			})
			.collect();

		assert_eq!(all.len(), 2790); // Make sure we got everything.

		for ext in all {
			assert!(
				Extension::new(ext).is_some(),
				"Failed for {ext}.",
			);
		}
	}
}
