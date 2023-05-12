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

#[cucumber_world]
impl<T: From<&'static str>> inner::World<T> {
    /// With docs
    #[given("Hullo")]
    #[when("Boih")]
    fn test(&mut self) {
        self.do_thing();
    }
}
