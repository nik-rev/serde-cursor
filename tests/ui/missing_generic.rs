use serde_cursor::CursorPath;

// T must be a type parameter that's available in the current scope
type X = CursorPath!(foo.bar + T);

fn main() {}
