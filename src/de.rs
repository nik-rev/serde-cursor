use core::fmt;
use std::marker::PhantomData;

use serde_core::de::DeserializeSeed;
use serde_core::de::IgnoredAny;
use serde_core::de::MapAccess;
use serde_core::de::SeqAccess;
use serde_core::de::Visitor;
use serde_core::Deserialize;
use serde_core::Deserializer;

use crate::Cons;
use crate::ConstPathSegment;
use crate::Cursor;
use crate::Nil;
use crate::PathSegment;
use crate::Sequence;
use crate::Wildcard;

impl<'de, T, P> Deserialize<'de> for Cursor<T, P>
where
    T: Deserialize<'de>,
    P: DeserializePath<'de, T>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = <P as DeserializePath<'de, T>>::deserialize(deserializer)?;
        Ok(Self(value, PhantomData))
    }
}

/// Deserializes the path to the type returned by [`Cursor!`]
///
/// For more information, see the [crate-level](crate) documentation.
#[diagnostic::on_unimplemented(
    message = "`{T}` doesn't implement `serde_cursor::Sequence`",
    note = "try: `Vec<{T}>`"
)]
pub trait DeserializePath<'de, T> {
    fn deserialize<D>(deserializer: D) -> Result<T, D::Error>
    where
        D: Deserializer<'de>;
}

// base case: we are at the target property
//
// Cursor!(package.4.name: String)
//                   ^^^^ we are here
//
// So we call: <String as Deserialize>::deserialize
impl<'de, T: Deserialize<'de>> DeserializePath<'de, T> for Nil {
    fn deserialize<D>(deserializer: D) -> Result<T, D::Error>
    where
        D: Deserializer<'de>,
    {
        // The recursion ends here.
        <T as Deserialize<'de>>::deserialize(deserializer)
    }
}

// step case: we are still digging into the object
//
// Cursor!(package.name: String)
//         ^^^^^^^ we may be here
//
// Cons<
//     Field<"package">, <-- we may be here
//     Cons<
//         Field<"name">,
//         Nil
//     >
// >
//
// Now calling either of:
//
// - deserializer.deserialize_map
// - deserializer.deserialize_seq
//
// Deserializes the entire rest of the data:
//
// Cursor!(package.name.whatever: String)
//                 ^^^^^^^^^^^^^^^^^^^ all of this will be deserialized in this step
//
// Cons<
//     Field<"package">,
//
//     Cons<              |
//         Field<"name">, |
//         Nil            |
//     >                  |
//     ^^^^^^^^^^^^^^^^^^^^ all of this will be deserialized (a single recursive step)
// >
//
impl<'de, S, P, T> DeserializePath<'de, T> for Cons<S, P>
where
    S: ConstPathSegment, // const S: PathSegment
    P: DeserializePath<'de, T>,
    T: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<T, D::Error>
    where
        D: Deserializer<'de>,
    {
        use serde_core::de::Error;
        let segment = <S as ConstPathSegment>::VALUE;

        // The path `P` corresponds to everything after "package"
        // while `String` corresponds to the value of the field

        let result = match segment {
            // The current segment is a named field.
            //
            // Cursor!(package.serde.name: String)
            //         ^^^^^^^ we are here
            PathSegment::Field(name) => {
                // Cursor!(package.serde.name: String)
                //                 ^^^^^^^^^^ deserialize all of this
                deserializer.deserialize_map(FieldVisitor::<P, T> {
                    target: name,
                    _marker: PhantomData,
                })
            }

            // The current segment is an index into a sequence.
            //
            // Cursor!(packages.4.name: String)
            //         ^^^^^^^^ we are here
            PathSegment::Index(index) => {
                // Cursor!(packages.4.name: String)
                //                 ^^^^^^^ deserialize all of this
                deserializer.deserialize_seq(SequenceVisitor::<P, T> {
                    target_index: index,
                    _marker: PhantomData,
                })
            }
        };

        // wrap the error with the current path segment
        result.map_err(|e| {
            let err_str = e.to_string();
            let path_str = match segment {
                PathSegment::Field(name) => format!(".{}", name),
                PathSegment::Index(i) => format!("[{}]", i),
            };

            // if the error is a nested path (starts with . or [), just join them
            if err_str.starts_with('.') || err_str.starts_with('[') {
                D::Error::custom(format!("{}{}", path_str, err_str))
            }
            // if this is the "top" level of the path, ensure it starts with a dot
            else {
                let prefix = if path_str.starts_with('[') {
                    format!(".{}", path_str)
                } else {
                    path_str
                };
                D::Error::custom(format!("{}: {}", prefix, err_str))
            }
        })
    }
}

/// Deserializes field named `target` at the Path (`P` implementing [`DeserializePath`])
/// into the type `T` implementing [`Deserialize`].
struct FieldVisitor<P, T> {
    /// The field that we are searching for.
    target: &'static str,
    _marker: PhantomData<(P, T)>,
}

impl<'de, P, T> Visitor<'de> for FieldVisitor<P, T>
where
    P: DeserializePath<'de, T>,
    T: Deserialize<'de>,
{
    type Value = T;

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "map with field '{}'", self.target)
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        // We search for the same key in the map as `self.target`.

        let mut result = None;

        // If `self.target` is "key", then we follow these steps:
        //
        // {
        //     "foo": ?, // value ignored.
        //     ^^^^^ Is this "key"? NO! value is not deserialized
        //
        //     "bar": ?, // value Ignored.
        //     ^^^^^ Is this "key"? NO! value is not deserialized
        //
        //     "key": ?
        //     ^^^^^ Is this "key"? YES! value is deserialized
        // }
        //
        //     "key": ?
        //            ^ deserializing the value is job of the `PathSeed`
        while let Some(key) = map.next_key::<String>()? {
            if key == self.target && result.is_none() {
                // The value is deserialized using the `DeserializeSeed` implementation
                // for `PathSeed`, which delegates to `DeserializePath::deserialize`
                result = Some(map.next_value_seed(PathSeed::<P, T>(PhantomData))?);
            } else {
                // The value is ignored, not deserialized - we don't have to
                // read a large document into memory
                map.next_value::<IgnoredAny>()?;
            }
        }

        match result {
            // The map contains a key of the same name as `self.target`.
            Some(val) => Ok(val),

            // The map does not contain a key of the same name as `self.target`.
            None => {
                // This allows Option<T> to become None instead of failing
                // deserialization completely.
                //
                // Say we are searching for the "dependencies" key, but we can't find it.
                //
                // let value = from_str::<Cursor!(package.dependencies.serde.name: Option<String>)>(json)?.0;
                //                                        ^^^^^^^^^^^^ key not found
                //
                // Then `value` will be `None`, instead of an error occurring.
                //
                // We are basically inserting this key into the map:
                //
                // "package": {
                //     "dependencies": null
                //     ^^^^^^^^^^^^^^^^^^^^ this wasn't here before, but we added it
                //                          effectively treat missing value same as if the
                //                          value is specified to be "null"
                // }
                let result =
                    T::deserialize(serde_core::de::value::UnitDeserializer::<A::Error>::new());

                // If the Option<T> case failed, that means this is a regular type - like
                // a String, for example. So the field was required, and we report an error.
                result.map_err(|_| {
                    <A::Error as serde_core::de::Error>::custom(format!(
                        "missing field '{}'",
                        self.target
                    ))
                })
            }
        }
    }
}

/// Deserializes the element at `target_index` at the Path (`P` implementing [`DeserializePath`]).
struct SequenceVisitor<P, T> {
    /// The index in the sequence we want to navigate into.
    target_index: usize,
    _marker: PhantomData<(P, T)>,
}

impl<'de, P, T> Visitor<'de> for SequenceVisitor<P, T>
where
    P: DeserializePath<'de, T>,
    T: Deserialize<'de>,
{
    type Value = T;

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "a sequence containing at least {} elements",
            self.target_index + 1
        )
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        // Cursor!(packages.4.name: String)
        //          ^^^^^^^^ we are here, looking for index 4

        // Skip elements before the target index to avoid memory bloat.
        // [
        //   {...}, // index 0: IgnoredAny
        //   {...}, // index 1: IgnoredAny
        //   {...}, // index 2: IgnoredAny
        //   {...}, // index 3: IgnoredAny
        //   {...}  // index 4: This is the one!
        // ]
        for i in 0..self.target_index {
            if seq.next_element::<IgnoredAny>()?.is_none() {
                return Err(serde_core::de::Error::custom(format!(
                    "index {} out of bounds (length {})",
                    self.target_index, i
                )));
            }
        }

        // We found the index. Now we use PathSeed to continue the recursion
        // for the rest of the path (e.g., ".name").
        let result = seq
            .next_element_seed(PathSeed::<P, T>(PhantomData))?
            .ok_or_else(|| {
                serde_core::de::Error::custom(format!("index {} out of bounds", self.target_index))
            })?;

        // Exhaust the sequence. Some formats (like binary formats or strict JSON
        // parsers) require the visitor to finish the entire sequence.
        while seq.next_element::<IgnoredAny>()?.is_some() {}

        Ok(result)
    }
}

/// Visitor for the Wildcard (`*`) path segment.
///
/// ```txt
/// Cursor!(package.*.name: Vec<String>)
///                 ^
/// ```
///
/// Collects multiple items into a sequence `C` implementing [`Sequence`],
///
/// In this case every `name` field corresponds to `String`, all of the
/// `name`s will be collected into a single `Vec<String>`.
struct WildcardVisitor<P, C> {
    _marker: PhantomData<(P, C)>,
}

impl<'de, P, C> DeserializePath<'de, C> for Cons<Wildcard, P>
where
    C: Sequence,
    P: DeserializePath<'de, C::Item>,
    C::Item: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<C, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_seq(WildcardVisitor::<P, C> {
            _marker: PhantomData,
        })
    }
}

impl<'de, P, C> Visitor<'de> for WildcardVisitor<P, C>
where
    C: Sequence,
    P: DeserializePath<'de, C::Item>,
    C::Item: Deserialize<'de>,
{
    type Value = C;

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("a sequence")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut items = <C as Default>::default();
        let mut index = 0;

        // Instead of skipping, we visit EVERY element in the sequence.
        //
        // For every element, we apply the rest of the path `P`.
        //
        // Cursor!(packages.*.name: Vec<String>)
        //          [
        //            {"name": "serde"}, // Apply ".name" -> "serde"
        //            {"name": "anyhow"} // Apply ".name" -> "anyhow"
        //          ]
        while let Some(item) = seq
            .next_element_seed(PathSeed::<P, C::Item>(PhantomData))
            .map_err(|e| serde_core::de::Error::custom(format!("[{}]{}", index, e)))?
        {
            items.push(item);
            index += 1;
        }

        Ok(items)
    }
}

/// A [`DeserializeSeed`] that allows us to pass our path-traversal state.
struct PathSeed<P, T>(PhantomData<(P, T)>);

impl<'de, P, T> DeserializeSeed<'de> for PathSeed<P, T>
where
    P: DeserializePath<'de, T>,
    T: Deserialize<'de>,
{
    type Value = T;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        <P as DeserializePath<'de, T>>::deserialize(deserializer)
    }
}

#[cfg(feature = "serde_with")]
impl<'de, T, P> serde_with::DeserializeAs<'de, T> for Cursor<T, P>
where
    T: Deserialize<'de>,
    P: DeserializePath<'de, T>,
{
    fn deserialize_as<D>(deserializer: D) -> Result<T, D::Error>
    where
        D: Deserializer<'de>,
    {
        <P as DeserializePath<'de, T>>::deserialize(deserializer)
    }
}
