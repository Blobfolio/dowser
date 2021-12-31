/*!
# Dowser: Utility Methods.
*/

use rayon::{
	iter::ParallelIterator,
	prelude::IntoParallelIterator,
};
use std::path::Path;



#[must_use]
/// # Disk Usage
///
/// This sums the provided file sizes in parallel. If you only have a couple
/// files to check, it is likely going to be faster to handle that manually (
/// in serial), but if there are tons, this can get you your answer quickly.
///
/// ## Examples
///
/// ```no_run
/// let paths = vec![ "/path/one", "path/two", "path/three" ];
/// let size = dowser::utility::du(&paths);
/// ```
pub fn du<I, P>(src: I) -> u64
where P: AsRef<Path>, I: IntoParallelIterator<Item=P> {
	src.into_par_iter()
		.map(|p| std::fs::metadata(p).map_or(0, |m| m.len()))
		.sum()
}

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



#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn t_du() {
		let files = vec![
			std::fs::canonicalize("./tests/assets/file.txt").unwrap(),
			std::fs::canonicalize("./tests/assets/functioning.JPEG").unwrap(),
			std::fs::canonicalize("./tests/assets/is-executable.sh").unwrap(),
		];

		let size = du(&files);
		assert_eq!(size, 30_383_u64);

		// Make sure ownership is still OK.
		assert_eq!(files.len(), 3);
	}

	#[test]
	fn t_resolve_path() {
		let test_dir = std::fs::canonicalize("./tests/links").expect("Missing dowser link directory.");

		let raw = vec![
			test_dir.join("01"),
			test_dir.join("02"),
			test_dir.join("03"),
			test_dir.join("04"),
			test_dir.join("05"), // Directory.
			test_dir.join("06"), // Directory.
			test_dir.join("07"), // Sym to six.
			test_dir.join("06/08"),
			test_dir.join("06/09"),
			test_dir.join("06/10"), // Sym to one.
		];

		let canon = {
			let mut tmp: Vec<PathBuf> = raw.iter()
				.filter_map(|x| std::fs::canonicalize(x).ok())
				.collect();
			tmp.sort();
			tmp.dedup();
			tmp
		};

		// There should be two fewer entries as two are symlinks.
		assert_eq!(raw.len(), 10);
		assert_eq!(canon.len(), 8, "{:?}", canon);
		assert!(! canon.contains(&raw[6]));
		assert!(! canon.contains(&raw[9]));

		let trusting = {
			let mut tmp: Vec<PathBuf> = raw.iter()
				.filter_map(|x| resolve_path(x.clone(), true).map(|(_, _, p)| p))
				.collect();
			tmp.sort();
			tmp.dedup();
			tmp
		};

		assert_eq!(trusting, canon);
	}
}
