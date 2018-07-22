#![feature(plugin)]
#![plugin(interpolate_idents)]

macro_rules! define_foo {
    ($x:ident) => ( interpolate_idents! {
        fn [foo_ $x _1]() -> u32 { 1 }

        struct [Foo $x] { [$x _30]: u32 }
        impl [Foo $x] {
            pub fn new() -> [Foo $x] {
                [Foo $x] { [$x _30]: 30 }
            }
        }
    } )
}

define_foo!(bar);

#[test]
fn test_macro() {
    assert_eq!(foo_bar_1(), 1);
    assert_eq!(Foobar::new().bar_30, 30);
}

macro_rules! define_brackets {
    () => ( interpolate_idents! {
        fn brackets(data: &[i32; 1]) -> Vec<i32> {
            let mut b: Vec<i32> = vec![];
            let c: Vec<i32> = vec![1, 2, 3];
            let d: Vec<i32> = vec![1; 25];
            b.push(c[1]);
            b.push(d[1]);
            b.push(data[0]);
            b
        }
    } )
}

define_brackets!();

#[test]
fn test_brackets() {
    let data = [1; 1];
    assert_eq!(brackets(&data), vec![2, 1, 1]);
}


macro_rules! define_underscore_idents {
    ($x:ident) => ( interpolate_idents! {
        fn [_ $x]() -> u32 { 1 }
        fn [$x _]() -> u32 { 2 }
        fn [_ $x _ $x _]() -> u32 { 3 }
    } )
}

define_underscore_idents!(bar);

#[test]
fn test_underscores() {
    assert_eq!(_bar(), 1);
    assert_eq!(bar_(), 2);
    assert_eq!(_bar_bar_(), 3);
}

macro_rules! define_attributes {
    ($x:ident) => ( interpolate_idents! {
        #[inline]
        fn [_ $x]() -> u32 { 1 }

        #[allow(unreachable_code)]
        fn [_ $x _2]() -> u32 { return 1; 2 }
    } )
}

define_attributes!(attr);

#[test]
fn test_attributes() {
    assert_eq!(_attr(), 1);
}
