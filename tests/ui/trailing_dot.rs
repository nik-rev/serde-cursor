use serde_cursor::Cursor;

// fails because it expects a segment after the dot
type X = Cursor!(a.b.: i32);

fn main() {}
