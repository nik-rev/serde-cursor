use crate::ConstPathSegment;
use crate::Nil;
use crate::PathSegment;
use core::fmt;
use std::marker::PhantomData;

use serde_core::de::DeserializeSeed;
use serde_core::de::MapAccess;
use serde_core::{
    Deserialize, Deserializer,
    de::{IgnoredAny, SeqAccess, Visitor},
};

use crate::Cursor;
use crate::{Cons, Sequence, Wildcard};

struct SequenceVisitor<P, T> {
    target_index: usize,
    _marker: PhantomData<(P, T)>,
}

impl<'de, P, T> Visitor<'de> for SequenceVisitor<P, T>
where
    P: Path<'de, T>,
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
        // skip elements before the target index
        for i in 0..self.target_index {
            if seq.next_element::<IgnoredAny>()?.is_none() {
                return Err(serde_core::de::Error::custom(format!(
                    "index {} out of bounds (length {})",
                    self.target_index, i
                )));
            }
        }

        // found the index, recurse to the next part of the path
        let result = seq
            .next_element_seed(PathSeed::<P, T>(PhantomData))?
            .ok_or_else(|| {
                serde_core::de::Error::custom(format!("index {} out of bounds", self.target_index))
            })?;

        // consume the rest of the sequence
        // some deserializers (like serde_json) will error if the sequence isn't exhausted
        while seq.next_element::<IgnoredAny>()?.is_some() {}

        Ok(result)
    }
}

struct FieldVisitor<P, D> {
    target: &'static str,
    _marker: PhantomData<(P, D)>,
}

impl<'de, P, T> Visitor<'de> for FieldVisitor<P, T>
where
    P: Path<'de, T>,
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
        let mut result = None;

        while let Some(key) = map.next_key::<String>()? {
            if key == self.target && result.is_none() {
                result = Some(map.next_value_seed(PathSeed::<P, T>(PhantomData))?);
            } else {
                map.next_value::<IgnoredAny>()?;
            }
        }

        result.ok_or_else(|| {
            serde_core::de::Error::custom(format!("field '{}' not found", self.target))
        })
    }
}

// struct PathSeed2<P, T>(&'static str, PhantomData<(P, T)>);

// impl<'de, P, T> DeserializeSeed<'de> for PathSeed2<P, T>
// where
//     P: Path<'de, T>,
//     T: Deserialize<'de>,
// {
//     type Value = T;

//     fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
//     where
//         D: Deserializer<'de>,
//     {
//         deserializer.deserialize_map(FieldVisitor::<P, T> {
//             target: self.0,
//             _marker: PhantomData,
//         })
//     }
// }

struct PathSeed<P, T>(PhantomData<(P, T)>);

impl<'de, P, T> DeserializeSeed<'de> for PathSeed<P, T>
where
    P: Path<'de, T>,
    T: Deserialize<'de>,
{
    type Value = T;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        P::navigate(deserializer)
    }
}

#[diagnostic::on_unimplemented(
    message = "`{T}` doesn't implement `serde_cursor::Sequence`",
    note = "try: `Vec<{T}>`"
)]
#[doc(hidden)]
pub trait Path<'de, T> {
    fn navigate<D>(deserializer: D) -> Result<T, D::Error>
    where
        D: Deserializer<'de>;
}

// base case: we are at the target property
impl<'de, T: Deserialize<'de>> Path<'de, T> for Nil {
    fn navigate<D>(deserializer: D) -> Result<T, D::Error>
    where
        D: Deserializer<'de>,
    {
        T::deserialize(deserializer)
    }
}

// step case: we are still digging into the object
impl<'de, S, P, T> Path<'de, T> for Cons<S, P>
where
    S: ConstPathSegment,
    P: Path<'de, T>,
    T: Deserialize<'de>,
{
    fn navigate<D>(deserializer: D) -> Result<T, D::Error>
    where
        D: Deserializer<'de>,
    {
        match S::VALUE {
            PathSegment::Field(name) => deserializer.deserialize_map(FieldVisitor::<P, T> {
                target: name,
                _marker: PhantomData,
            }),
            PathSegment::Index(index) => deserializer.deserialize_seq(SequenceVisitor::<P, T> {
                target_index: index,
                _marker: PhantomData,
            }),
        }
    }
}

// struct SequenceFieldVisitor<P, T, V> {
//     target: &'static str,
//     _marker: PhantomData<(P, T, V)>,
// }

// impl<'de, P, T, V> Visitor<'de> for SequenceFieldVisitor<P, T, V>
// where
//     P: Path<'de, T>,
//     T: Deserialize<'de>,
//     V: Seq<Item = T>,
// {
//     type Value = V;

//     fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
//         formatter.write_fmt(format_args!("sequence of map with field {}", self.target))
//     }

//     fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
//     where
//         A: SeqAccess<'de>,
//     {
//         let mut values = V::default();

//         while let Some(value) =
//             seq.next_element_seed(PathSeed2::<P, T>(self.target, PhantomData))?
//         {
//             V::push(&mut values, value);
//         }

//         Ok(values)
//     }
// }

struct WildcardVisitor<P, C> {
    _marker: PhantomData<(P, C)>,
}

impl<'de, P, C> Path<'de, C> for Cons<Wildcard, P>
where
    C: Sequence,
    P: Path<'de, C::Item>,
    C::Item: Deserialize<'de>,
{
    fn navigate<D>(deserializer: D) -> Result<C, D::Error>
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
    P: Path<'de, C::Item>,
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
        let mut items = C::default();
        while let Some(item) = seq.next_element_seed(PathSeed::<P, C::Item>(PhantomData))? {
            items.push(item);
        }
        Ok(items)
    }
}

impl<'de, T, P> Deserialize<'de> for Cursor<T, P>
where
    T: Deserialize<'de>,
    P: Path<'de, T>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = P::navigate(deserializer)?;
        Ok(Self(value, PhantomData))
    }
}

#[cfg(feature = "serde_with")]
impl<'de, T, P> serde_with::DeserializeAs<'de, T> for Cursor<T, P> {
    fn deserialize_as<D>(deserializer: D) -> Result<T, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(
            serde_with::de::DeserializeAsWrap::<T, Cursor<T, P>>::deserialize(deserializer)?
                .into_inner(),
        )
    }
}
