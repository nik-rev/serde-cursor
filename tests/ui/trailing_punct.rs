use serde_cursor::Cursor;

// A single dot is not a valid path
type X = Cursor!( . );

fn main() {}
