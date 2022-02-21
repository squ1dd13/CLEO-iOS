pub use loader::get_game_path;
pub use res::*;

mod loader;
mod old_stream;
mod res;
mod stream;

pub fn init() {
    loader::init();
    old_stream::init();
    res::init();
}
