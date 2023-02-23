use std::marker::PhantomData;

use flatbuffers::{FlatBufferBuilder, Table};

/// Applied to the struct
pub trait AdapterExt {
    fn to_fb(&self, builder: &mut FlatBufferBuilder);
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
    fn make(&self, table: &mut Table) -> T;
    fn new_object(&self) -> T;
}
