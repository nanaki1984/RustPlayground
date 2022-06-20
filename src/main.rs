use std::any::{TypeId, type_name};
use std::collections::HashMap;
use std::marker::PhantomData;
use std::mem::{size_of, align_of};
use std::vec::{Vec};
use std::time::{Duration, Instant};
use rl_core::alloc::InlineAllocator;
use rl_core::array::Array;
use rl_core::map::Map;
/*
struct MockRawArray<A : AllocatorBase> {
    alloc: A,
}

struct MockArray<T, A: ArrayAllocator<T>>(MockRawArray<A>, PhantomData<T>);

struct MockArrayContainer {
    field: MockArray<i32, InlineAllocator<5, i32>>,
}
*/
#[derive(Debug)]
struct TypeInfo {
    id: TypeId,
    name: &'static str,
    size: usize,
    align: usize,
}

impl TypeInfo {
    /*const*/ fn from_type<T>() -> TypeInfo
        where T : 'static + Sized
    {
        TypeInfo {
            id: TypeId::of::<T>(),
            name: type_name::<T>(),
            size: size_of::<T>(),
            align: align_of::<T>(),
        }
    }
}

trait Typed {
    fn type_info() -> &'static TypeInfo;
}

#[repr(align(16))]
#[derive(Debug)]
struct TestStruct
{
    x: f32,
    y: f32,
    z: f32,
    w: f32,
    s: f32,
}

impl TestStruct {
    const fn test<const FLAG: bool>() -> &'static str
    {
        if FLAG {
            "True"
        } else {
            "False"
        }
    }

    fn add(&self, other: &TestStruct) -> TestStruct
    {
        TestStruct {
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z + other.z,
            w: self.w + other.w,
            s: self.s + other.s,
        }
    }
}

/*
impl Typed for TestStruct {
    fn type_info() -> &'static TypeInfo {
        static ti: TypeInfo = TypeInfo::from_type::<TestStruct>();
        return &ti;
    }
}
*/
struct TestStruct2(&'static str);

impl Drop for TestStruct2 {
    fn drop(&mut self) {
        println!("{}", self.0);
    }
}

#[inline(never)]
fn use_vec() {
    let now = Instant::now();

    let mut sum = 0;
    //let mut vec = Vec::new();
    for i in (0..100) {
        let mut vec = Vec::new();
        for _ in (0..1000) {
            vec.insert(0, i);
            vec.push(i);
        }
        // TODO: why into_iter "consumes" vec?! it's a version w/out lifetime of vec
        //sum += vec.into_iter().sum::<i32>();
        sum += vec.iter().sum::<i32>();
        //for j in 0..vec.len() {
        //    sum += vec[j];
        //}
        //vec.clear();
    }

    println!("vec {} ms (sum {})", now.elapsed().as_millis(), sum);
}

#[inline(never)]
fn use_array() {
    let now = Instant::now();

    let mut sum = 0;
    //let mut array = Array::new();
    for i in (0..100) {
        let mut array = Array::new();
        for _ in (0..1000) {
            array.insert(0, i);
            array.push_back(i);
        }
        sum += array.iter().sum::<i32>();
        //for j in 0..array.num() {
        //    sum += array[j];
        //}
        //array.clear();
    }

    println!("array {} ms (sum {})", now.elapsed().as_millis(), sum);
}

fn main() {
    let ti = TypeInfo::from_type::<TestStruct>();
    //let ti = TestStruct::type_info();
    println!("Hello, world!");
    println!("{ti:?}");
    println!("True is {}, False is {}", TestStruct::test::<true>(), TestStruct::test::<false>());

    let v0 = TestStruct { x: 1.0, y: 1.0, z: 2.0, w: 2.0, s: 0.0 };
    let v1 = TestStruct { x: 3.0, y: 3.0, z: 3.0, w: 4.0, s: 0.0 };
    let v2 = v0.add(&v1);

    println!("{v2:?}");

    let mut array = Array::new();
    array.push_back(TestStruct2("Hello"));
    array.push_back(TestStruct2("World"));

    // test assert
    //array.remove(2);

    let mut string = "".to_string();
    for tmp in &array {
        string += tmp.0;
    }
    println!("{}", string);

    let world = array.swap_remove(1);
    println!("BeforeClear");
    array.clear();
    drop(world);
    println!("AfterClear");
    drop(array);

    println!("TestStruct2 size: {}", std::mem::size_of::<TestStruct2>());
    let mut inline_array: Array<TestStruct2, InlineAllocator<4, TestStruct2>> = Array::custom_allocator();
    inline_array.push_back(TestStruct2("One"));
    inline_array.push_back(TestStruct2("Two"));
    inline_array.push_back(TestStruct2("Three"));
    inline_array.push_back(TestStruct2("Four"));
    //inline_array.push_back(TestStruct2("Five"));

    // call to find them in asm
    use_array();
    use_vec();
/*
    struct SomeDroppableStruct {
        number: i32,
    }
    impl Drop for SomeDroppableStruct {
        fn drop(&mut self) {
            println!("dtor: {}", self.number);
        }
    }
    let mut test_map = Map::new();
    test_map.insert("key", SomeDroppableStruct{ number: 10 });
    test_map.insert("other_key", SomeDroppableStruct{ number: 20 });
    let some_num = test_map.remove("key").unwrap();
    println!("Before drop");
    drop(some_num);
    println!("After drop");
*/
}
