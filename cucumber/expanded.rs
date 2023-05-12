#![feature(prelude_import)]
#[prelude_import]
use std::prelude::rust_2021::*;
#[macro_use]
extern crate std;
use cucumber::cucumber_world;
mod inner {
    pub struct World<T> {
        state: Option<T>,
    }
    impl<T> World<T>
    where
        T: From<&'static str>,
    {
        pub fn do_thing(&mut self) {
            self.state = Some("bruh".into());
        }
    }
}
impl<T: From<&'static str>> inner::World<T> {
    /// With docs
    fn test(&mut self) {
        self.do_thing();
    }
}
#[rustc_main]
#[no_coverage]
pub fn main() -> () {
    extern crate test;
    test::test_main_static(&[])
}
