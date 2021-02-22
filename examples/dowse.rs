/*!
# Dowser Example: Walk and Count

This simply finds all files under /usr/share and reports the total.
*/



/// Do it.
fn main() {
	let len: usize = dowser::dowse(&["/usr/share"]).len();
	println!("There are {} files in /usr/share.", len);
}
