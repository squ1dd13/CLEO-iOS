use std::sync::Mutex;

pub struct Settings {
    pub sixty_fps: bool,
    pub show_fps: bool,
}

impl Settings {
    fn default() -> Settings {
        Settings {
            sixty_fps: true,
            show_fps: false,
        }
    }
}

/// Obtain a mutable reference to the shared settings struct and call a function,
/// passing the mutable reference as an argument.
pub fn with_shared<T>(with: fn(&mut Settings) -> T) -> T {
    let mut locked = SETTINGS.lock();
    with(locked.as_mut().unwrap())
}

lazy_static::lazy_static! {
    static ref SETTINGS: Mutex<Settings> = Mutex::new(Settings::default());
}
