use serde_core::de::{
    Deserialize, DeserializeSeed, Deserializer, IgnoredAny, MapAccess, SeqAccess, Visitor,
};
use std::fmt;
use std::marker::PhantomData;

pub use const_str::C1;
pub use const_str::C2;
pub use const_str::C3;
pub use const_str::C4;
pub use const_str::StrLen;

pub use crate::const_str::ConstStr;

pub mod const_str;

pub enum PathSegment {
    Field(&'static str),
    Index(usize),
}

pub trait ConstPathSegment {
    const VALUE: PathSegment;
}

pub struct Nil;
pub struct Cons<S, T>(PhantomData<(S, T)>);

pub trait Path {
    fn head() -> Option<PathSegment>;
    type Tail: Path;
}

impl Path for Nil {
    fn head() -> Option<PathSegment> {
        None
    }
    type Tail = Nil;
}

impl<S: ConstPathSegment, T: Path> Path for Cons<S, T> {
    type Tail = T;
    fn head() -> Option<PathSegment> {
        Some(S::VALUE)
    }
}

pub trait PathNavigator<'de, D>: Path {
    fn navigate<De>(deserializer: De) -> Result<D, De::Error>
    where
        De: Deserializer<'de>;
}

// base case: we are at the target property
impl<'de, D: Deserialize<'de>> PathNavigator<'de, D> for Nil {
    fn navigate<De>(deserializer: De) -> Result<D, De::Error>
    where
        De: Deserializer<'de>,
    {
        D::deserialize(deserializer)
    }
}

// Step Case: We are still digging into the object
impl<'de, S, T, D> PathNavigator<'de, D> for Cons<S, T>
where
    S: ConstPathSegment,
    T: PathNavigator<'de, D>,
    D: Deserialize<'de>,
{
    fn navigate<De>(deserializer: De) -> Result<D, De::Error>
    where
        De: Deserializer<'de>,
    {
        match S::VALUE {
            PathSegment::Field(name) => deserializer.deserialize_map(FieldVisitor::<T, D> {
                target: name,
                _marker: PhantomData,
            }),
            PathSegment::Index(index) => deserializer.deserialize_seq(SequenceVisitor::<T, D> {
                target_index: index,
                _marker: PhantomData,
            }),
        }
    }
}

struct SequenceVisitor<P, D> {
    target_index: usize,
    _marker: PhantomData<(P, D)>,
}

impl<'de, P, D> Visitor<'de> for SequenceVisitor<P, D>
where
    P: PathNavigator<'de, D>,
    D: Deserialize<'de>,
{
    type Value = D;

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

        // found the index!, recurse to the next part of the path
        let result = seq
            .next_element_seed(PathSeed::<P, D>(PhantomData))?
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

impl<'de, P, D> Visitor<'de> for FieldVisitor<P, D>
where
    P: PathNavigator<'de, D>,
    D: Deserialize<'de>,
{
    type Value = D;

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
                result = Some(map.next_value_seed(PathSeed::<P, D>(PhantomData))?);
            } else {
                map.next_value::<IgnoredAny>()?;
            }
        }
        result.ok_or_else(|| {
            serde_core::de::Error::custom(format!("field '{}' not found", self.target))
        })
    }
}

struct PathSeed<P, D>(PhantomData<(P, D)>);

impl<'de, P, D> DeserializeSeed<'de> for PathSeed<P, D>
where
    P: PathNavigator<'de, D>,
    D: Deserialize<'de>,
{
    type Value = D;

    fn deserialize<De>(self, deserializer: De) -> Result<Self::Value, De::Error>
    where
        De: Deserializer<'de>,
    {
        P::navigate(deserializer)
    }
}

pub struct Cursor<D, P> {
    pub value: D,
    _path: PhantomData<P>,
}

impl<'de, D, P> Deserialize<'de> for Cursor<D, P>
where
    D: Deserialize<'de>,
    P: PathNavigator<'de, D>,
{
    fn deserialize<De>(deserializer: De) -> Result<Self, De::Error>
    where
        De: Deserializer<'de>,
    {
        let value = P::navigate(deserializer)?;
        Ok(Self {
            value,
            _path: PhantomData,
        })
    }
}

pub struct FieldName<S: ConstStr>(PhantomData<S>);
pub struct Index<const N: usize>;

impl<S: ConstStr> ConstPathSegment for FieldName<S> {
    const VALUE: PathSegment = PathSegment::Field(S::VALUE);
}

impl<const N: usize> ConstPathSegment for Index<N> {
    const VALUE: PathSegment = PathSegment::Index(N);
}
