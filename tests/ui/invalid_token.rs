use serde_cursor::Cursor;

// '@' is not a valid path segment start
type X = Cursor!(@.field: i32);

fn main() {}
