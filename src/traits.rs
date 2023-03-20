use std::marker::PhantomData;

use flatbuffers::{FlatBufferBuilder, Table};

/// Applied to the struct
pub trait AdapterExt {
    fn flatten(&self, builder: &mut FlatBufferBuilder);
}


/// A different factory can be targeted by creating
/// a new macro based on the internal package's
/// LeanBufferInternal, hypothetically,
/// this package can be used to merge the generated code
/// with the generated flatbuffers bindings
pub struct Factory<T> {
    pub phantom_data: PhantomData<T>,
}

/// Applied to a dyn factory object, because extension traits
/// do not support static functions e.g. `fn new_object() -> Self`
pub trait FactoryExt<T>
where
    T: ?Sized,
{
    fn inflate<'a>(&self, table: &mut Table<'a>) -> T;
    fn new_object(&self) -> T;
}
