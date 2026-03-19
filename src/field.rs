use core::fmt;
use std::marker::PhantomData;

use serde::Deserialize;

use crate::{DeserializePathSegment, PathSegment, Z, current_field};

/// Visits a field in a map.
pub struct FieldVisitor<'query, 'de, D>(pub &'query mut Option<D>, pub PhantomData<&'de ()>);

impl<'query, 'de, D: Deserialize<'de>> serde::de::Visitor<'de> for FieldVisitor<'query, 'de, D> {
    type Value = ();

    fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let _ = f;
        todo!()
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::MapAccess<'de>,
    {
        match crate::current_field() {
            Some(PathSegment::Field(_)) => {
                while let Some(status) = map.next_key::<FieldKeyVisitStatus>()? {
                    match status {
                        FieldKeyVisitStatus::Visited => {
                            crate::next_field();
                            map.next_value_seed(DeserializePathSegment(self.0))?;
                        }
                        FieldKeyVisitStatus::Ignore => {
                            map.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(())
            }
            _ => todo!(),
        }
    }
}

/// Visits keys until the given key is found.
enum FieldKeyVisitStatus {
    Visited,
    Ignore,
}

impl<'de> serde::de::Deserialize<'de> for FieldKeyVisitStatus {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde_query::__priv::serde::Deserializer<'de>,
    {
        deserializer.deserialize_identifier(FieldKeyVisitor)
    }
}

/// Visits fields in a map.
struct FieldKeyVisitor;

impl<'de> serde::de::Visitor<'de> for FieldKeyVisitor {
    type Value = FieldKeyVisitStatus;

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let Some(PathSegment::Field(field)) = crate::current_field() else {
            unreachable!()
        };
        fmt::Formatter::write_fmt(f, format_args!("field '{field}'"))
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        let Some(PathSegment::Field(field)) = crate::current_field() else {
            unreachable!()
        };
        Ok(if value == field {
            FieldKeyVisitStatus::Visited
        } else {
            FieldKeyVisitStatus::Ignore
        })
    }

    fn visit_bytes<E>(self, value: &[u8]) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        let Some(PathSegment::Field(field)) = crate::current_field() else {
            unreachable!()
        };
        Ok(if value == field.as_bytes() {
            FieldKeyVisitStatus::Visited
        } else {
            FieldKeyVisitStatus::Ignore
        })
    }
}
