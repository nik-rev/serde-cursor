use serde_cursor::CursorPath;

// expected to see "+ T"
type X<T> = CursorPath!(foo.bar);

fn main() {}
