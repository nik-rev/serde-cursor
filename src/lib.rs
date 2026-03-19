#![feature(adt_const_params)]
#![feature(unsized_const_params)]
#![allow(incomplete_features)]

use std::marker::PhantomData;

use serde::{Deserialize, de::DeserializeSeed};

use crate::field::FieldVisitor;

mod field;

trait Path {}

impl<T> Path for T {}

struct Cursor<D, P> {
    pub value: D,
    _path: PhantomData<P>,
}

impl<'de, D: Deserialize<'de>, P: Path> Deserialize<'de> for Cursor<D, P> {
    fn deserialize<De>(deserializer: De) -> Result<Self, De::Error>
    where
        De: serde_core::Deserializer<'de>,
    {
        let mut value: Option<D> = None;

        <DeserializePathSegment<D> as DeserializeSeed>::deserialize(
            DeserializePathSegment(&mut value),
            deserializer,
        )?;

        let value = value.unwrap();

        Ok(Self {
            value,
            _path: PhantomData,
        })
    }
}

/// Deserializes a single path segment.
struct DeserializePathSegment<'query, D>(&'query mut Option<D>);

impl<'de, 'query, D: Deserialize<'de>> DeserializeSeed<'de> for DeserializePathSegment<'query, D> {
    type Value = ();

    fn deserialize<De>(self, deserializer: De) -> Result<Self::Value, De::Error>
    where
        De: serde::Deserializer<'de>,
    {
        match current_field() {
            Some(PathSegment::Field(_)) => {
                deserializer.deserialize_map(FieldVisitor(self.0, PhantomData))?;
                Ok(())
            }
            None => {
                *self.0 = Some(<D as Deserialize>::deserialize(deserializer)?);
                Ok(())
            }
            _ => todo!(),
        }
    }
}

const FIELDS: [&str; 3] = ["a", "b", "c"];
static mut FIELD_COUNT: usize = 0;

fn current_field() -> Option<PathSegment> {
    Some(PathSegment::Field(FIELDS.get(unsafe { FIELD_COUNT })?))
}

fn next_field() {
    unsafe { FIELD_COUNT += 1 }
}

const Z: PathSegment = PathSegment::Field("lol");

#[derive(std::marker::ConstParamTy, PartialEq, Eq)]
enum PathSegment {
    Field(&'static str),
    ArrayExact(usize),
    Array,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_name() {
        let value = serde_json::json!({
            "a": {
                "b": {
                    "c": true
                }
            }
        });

        let value = serde_json::from_value::<Cursor<bool, ()>>(value)
            .unwrap()
            .value;

        assert!(value);
    }
}
