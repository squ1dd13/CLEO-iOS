use std::sync::Mutex;

pub struct OptionInfo {
    pub title: &'static str,
    pub description: &'static str,
    pub value: bool,
}

impl OptionInfo {
    const fn new(title: &'static str, description: &'static str, value: bool) -> OptionInfo {
        OptionInfo {
            title,
            description,
            value,
        }
    }
}

pub fn with_shared<T>(with: &mut impl FnMut(&mut [OptionInfo]) -> T) -> T {
    let mut locked = SETTINGS.lock();
    with(locked.as_mut().unwrap())
}

lazy_static::lazy_static! {
    static ref SETTINGS: Mutex<Vec<OptionInfo>> = Mutex::new(vec![
        OptionInfo::new(
            "60 FPS",
            "Increase the framerate limit from 30 to 60 FPS.",
            true,
        ),
        OptionInfo::new(
            "Show FPS",
            "Enable the game's built-in FPS visualisation.",
            false,
        ),
    ]);
}
