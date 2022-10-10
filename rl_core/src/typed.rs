use std::any::{Any, TypeId, type_name};
use std::alloc::Layout;

pub trait Typed: 'static {
    fn type_info(&self) -> TypeInfo;
}

impl<T: Any> Typed for T {
    fn type_info(&self) -> TypeInfo {
        TypeInfo::of::<T>()
    }
}

pub struct TypeInfo {
    id: TypeId,
    name: &'static str,
    layout: Layout,
}

impl TypeInfo {
    pub/* const*/ fn of<T: Any>() -> TypeInfo {
        TypeInfo {
            id: TypeId::of::<T>(),
            name: type_name::<T>(),
            layout: Layout::new::<T>(),
        }
    }

    pub fn get_id(&self) -> &TypeId {
        &self.id
    }

    pub fn get_name(&self) -> &'static str {
        self.name
    }

    pub fn get_layout(&self) -> &Layout {
        &self.layout
    }
}
