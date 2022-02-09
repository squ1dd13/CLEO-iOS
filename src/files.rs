mod loader;
mod res;
mod stream;

pub use loader::get_game_path;
pub use res::*;

pub fn init() {
    loader::init();
    stream::init();
    res::init();
}
