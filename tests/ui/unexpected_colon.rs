use serde_cursor::Cursor;

// colon after a dot with no identifier
type X = Cursor!(a. : bool);

fn main() {}
