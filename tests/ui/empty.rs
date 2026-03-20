use serde_cursor::Cursor;

// Empty macro call
type X = Cursor!();

// No path
type Y = Cursor!(: i32);

fn main() {}
