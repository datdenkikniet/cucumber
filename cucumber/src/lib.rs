pub use cucumber_macros::cucumber_world;

pub enum Error {}

pub trait World {
    fn run(self) -> Result<(), Error>;
}
