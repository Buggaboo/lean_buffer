use std::marker::PhantomData;

use flatbuffers::{FlatBufferBuilder, Table};

/// Applied to the struct
pub trait AdapterExt {
    fn flatten(&self, builder: &mut FlatBufferBuilder);
}

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
