use std::any::{Any, TypeId, type_name};
use std::alloc::Layout;

type PruneFunction = fn(*mut u8);

pub struct TypeInfo {
    id: TypeId,
    name: &'static str,
    layout: Layout,
    prune_fn: PruneFunction,
}

impl TypeInfo {
    pub/* const*/ fn of<T: Any>() -> TypeInfo { // Use T: Object to get functions
        TypeInfo {
            id: TypeId::of::<T>(),
            name: type_name::<T>(),
            layout: Layout::new::<T>(),
            prune_fn: |ptr: *mut u8| unsafe {
                std::ptr::drop_in_place(ptr.cast::<T>())
            }
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

    pub fn drop_in_place(&self, ptr: *mut u8) {
        (self.prune_fn)(ptr);
    }
}
