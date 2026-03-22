use serde_cursor::CursorPath;

// path must exist
type X<T> = CursorPath!(+ T);

fn main() {}
