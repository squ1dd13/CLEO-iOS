use crate::{call_original, hook, targets};

// CTimer::GetCyclesPerMillisecond is called between the FPS limit being set and when it is enforced,
//  so if we overwrite the limit here, our new value will be enforced.
fn cycles_per_millisecond() -> u32 {
    unsafe {
        *hook::slide::<*mut u32>(0x1008f07b8) = 60;
    }

    call_original!(targets::cycles_per_millisecond)
}

fn idle(p1: u64, p2: u64) {
    const SHOW_FPS: bool = false;

    unsafe {
        *hook::slide::<*mut bool>(0x10081c519) = SHOW_FPS;
    }

    call_original!(targets::idle, p1, p2);
}

pub fn hook() {
    targets::idle::install(idle);
    targets::cycles_per_millisecond::install(cycles_per_millisecond);
}
