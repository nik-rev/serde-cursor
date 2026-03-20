use crate::{Cons, ConstPathSegment, Cursor, Nil, PathSegment, Wildcard};
use serde_core::ser::{Serialize, SerializeMap, SerializeSeq, Serializer};
use std::marker::PhantomData;

impl<T, P> serde_core::Serialize for Cursor<T, P>
where
    P: ToCursor<T>,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde_core::Serializer,
    {
        P::serialize_nested(&self.0, serializer)
    }
}

pub trait ToCursor<T> {
    fn serialize_nested<S>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer;
}

// base case: no more path, just serialize the value
impl<T: Serialize> ToCursor<T> for Nil {
    fn serialize_nested<S>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        value.serialize(serializer)
    }
}

// step case: wrap in a map (field) or seq (Index)
impl<S, P, T> ToCursor<T> for Cons<S, P>
where
    S: ConstPathSegment,
    P: ToCursor<T>,
{
    fn serialize_nested<Ser>(value: &T, serializer: Ser) -> Result<Ser::Ok, Ser::Error>
    where
        Ser: Serializer,
    {
        match S::VALUE {
            PathSegment::Field(name) => {
                let mut map = serializer.serialize_map(Some(1))?;

                map.serialize_entry(name, &ToCursorWrapper::<P, T>(value, PhantomData))?;

                map.end()
            }
            PathSegment::Index(index) => {
                let mut seq = serializer.serialize_seq(Some(index + 1))?;

                // so if we have e.g. Query!(4), which accesses the 4th element,
                // when serializing we will create 3 "null"s and serialize the 4th element
                for _ in 0..index {
                    seq.serialize_element(&())?; // null
                }

                // the actual element that we are serializing
                seq.serialize_element(&ToCursorWrapper::<P, T>(value, PhantomData))?;

                seq.end()
            }
        }
    }
}

impl<P, T, C> ToCursor<C> for Cons<Wildcard, P>
where
    for<'a> &'a C: IntoIterator<Item = &'a T>,
    P: ToCursor<T>,
{
    fn serialize_nested<S>(value: &C, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut seq = serializer.serialize_seq(None)?;

        for item in value {
            seq.serialize_element(&ToCursorWrapper::<P, T>(item, PhantomData))?;
        }

        seq.end()
    }
}

// helper to bridge the recursion
struct ToCursorWrapper<'a, P, T>(&'a T, PhantomData<P>);

impl<'a, P, T> Serialize for ToCursorWrapper<'a, P, T>
where
    P: ToCursor<T>,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        P::serialize_nested(self.0, serializer)
    }
}

#[cfg(feature = "serde_with")]
impl<T, U, P> serde_with::SerializeAs<Cursor<T, P>> for Cursor<U, P>
where
    U: serde_with::SerializeAs<T>,
{
    fn serialize_as<S>(source: &Cursor<T, P>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde_core::Serializer,
    {
        serde_core::Serialize::serialize(
            &serde_with::ser::SerializeAsWrap::<T, U>::new(&source.0),
            serializer,
        )
    }
}

// impl<P, T, C> ToCursor<C> for Cons<Wildcard, P>
// where
//     C: IntoIterator<Item = T>,
//     T: Serialize,
//     for<'a> &'a C: IntoIterator<Item = &'a T>,
//     P: ToCursor<T>,
// {
//     fn serialize_nested<S>(value: &C, serializer: S) -> Result<S::Ok, S::Error>
//     where
//         S: Serializer,
//     {
//         let mut seq = serializer.serialize_seq(None)?;
//         for item in value {
//             seq.serialize_element(&ToCursorWrapper::<P, T>(item, PhantomData))?;
//         }
//         seq.end()
//     }
// }
