use std::collections::BTreeSet;
use std::collections::HashSet;
use std::collections::LinkedList;
use std::collections::VecDeque;
use std::hash::Hash;

/// Sequences of items, such as [`Vec<T>`] or [`HashSet<T>`].
/// The index-all `.*` syntax requires this trait to be implemented.
///
/// The purpose of this trait is to enable the index-all `.*.` syntax
/// when collecting fields of an array:
///
/// ```toml
/// [[package]]
/// serde = "1.0"
///
/// [[package]]
/// rand = "0.9"
/// ```
///
/// The index-all `.*` accesses every element in an array:
///
/// ```
/// # mod fs { pub fn read_to_string(_: &str) -> Result<String, Box<dyn std::error::Error>> { Ok(String::from("package = [{ name = 'serde' }, { name = 'rand' }]")) } }
/// use serde_cursor::Cursor;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let file = fs::read_to_string("Cargo.lock")?;
///
/// let packages: Vec<String> = toml::from_str::<Cursor!(package.*.name)>(&file)?.0;
///
/// assert_eq!(packages, vec!["serde", "rand"]);
/// # Ok(()) }
/// ```
///
/// Instead of `packages: Vec<String>`, we could have used any of these types:
///
/// - [`HashSet<String>`]
/// - [`VecDeque<String>`]
/// - [`LinkedList<String>`]
pub trait Sequence: Default {
    type Item;
    fn push(&mut self, item: Self::Item);
}

impl<T> Sequence for Vec<T> {
    type Item = T;
    fn push(&mut self, item: Self::Item) {
        self.push(item);
    }
}

impl<T> Sequence for VecDeque<T> {
    type Item = T;
    fn push(&mut self, item: Self::Item) {
        self.push_back(item);
    }
}

impl<T> Sequence for LinkedList<T> {
    type Item = T;
    fn push(&mut self, item: Self::Item) {
        self.push_back(item);
    }
}

impl<T> Sequence for HashSet<T>
where
    T: Eq + Hash,
{
    type Item = T;
    fn push(&mut self, item: Self::Item) {
        self.insert(item);
    }
}

impl<T> Sequence for BTreeSet<T>
where
    T: Ord,
{
    type Item = T;
    fn push(&mut self, item: Self::Item) {
        self.insert(item);
    }
}

impl Sequence for String {
    type Item = char;
    fn push(&mut self, item: Self::Item) {
        self.push(item);
    }
}

impl<T> Sequence for Option<T> {
    type Item = T;
    fn push(&mut self, item: Self::Item) {
        *self = Some(item);
    }
}
