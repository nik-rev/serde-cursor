use std::marker::PhantomData;

use crate::ConstStr;

pub enum PathSegment {
    Field(&'static str),
    Index(usize),
}

pub trait ConstPathSegment {
    const VALUE: PathSegment;
}

pub struct FieldName<S: ConstStr>(PhantomData<S>);
pub struct Index<const N: usize>;

impl<S: ConstStr> ConstPathSegment for FieldName<S> {
    const VALUE: PathSegment = PathSegment::Field(S::VALUE);
}

impl<const N: usize> ConstPathSegment for Index<N> {
    const VALUE: PathSegment = PathSegment::Index(N);
}
