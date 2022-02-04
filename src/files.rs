mod loader;
mod res;
mod stream;

pub use loader::get_game_path;
pub use res::{get_documents_path, get_log_path};

pub fn init() {
    loader::init();
    stream::init();
    res::init();
}
