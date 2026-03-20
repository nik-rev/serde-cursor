use serde_cursor::Cursor;

// floats are not valid path segments
type X = Cursor!(a.3.14.c: i32);

fn main() {}
