use serde_cursor::Cursor;

// the macro expects '.' not '::' for segments
type X = Cursor!(a::b: i32);

fn main() {}
