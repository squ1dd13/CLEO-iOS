use libc::c_char;

use crate::{call_original, hook, settings, targets};

// CTimer::GetCyclesPerMillisecond is called between the FPS limit being set and when it is enforced,
//  so if we overwrite the limit here, our new value will be enforced.
fn cycles_per_millisecond() -> u32 {
    unsafe {
        settings::with_shared(&mut |options| {
            *hook::slide::<*mut u32>(0x1008f07b8) = if options.get(settings::Key::SixtyFPS).value {
                60
            } else {
                30
            };
        });
    }

    call_original!(targets::cycles_per_millisecond)
}

fn idle(p1: u64, p2: u64) {
    unsafe {
        settings::with_shared(&mut |options| {
            *hook::slide::<*mut bool>(0x10081c519) = options.get(settings::Key::ShowFPS).value;
        });
    }

    call_original!(targets::idle, p1, p2);
}

#[repr(C)]
struct RGBA {
    red: u8,
    green: u8,
    blue: u8,
    alpha: u8,
}

fn display_fps() {
    let delta_time = crate::hook::slide::<fn() -> u32>(0x1004e8c70)();
    let current_delta = crate::hook::slide::<*mut isize>(0x1007baf00);
    let new_delta_index = unsafe { *current_delta } % 40;

    unsafe {
        *current_delta += 1;
    }

    let delta_times: *mut u32 = crate::hook::slide(0x1007bae60);

    unsafe {
        delta_times
            .offset(new_delta_index as isize)
            .write(delta_time);
    }

    // eq: CFont::SetBackground(...)
    crate::hook::slide::<fn(u8, u8)>(0x100381b94)(1, 0);

    // eq: CFont::SetBackgroundColor(...)
    crate::hook::slide::<fn(*const RGBA)>(0x100381ba8)(&RGBA {
        red: 0,
        green: 0,
        blue: 0,
        alpha: 180,
    });

    // eq: CFont::SetScale(...)
    crate::hook::slide::<fn(f32)>(0x1003819e0)(1.12);

    // eq: CFont::SetOrientation(...)
    crate::hook::slide::<fn(u32)>(0x100381be4)(0);

    // eq: CFont::SetJustify(...)
    crate::hook::slide::<fn(u8)>(0x100381bd4)(0);

    // eq: CFont::SetCentreSize(...)
    crate::hook::slide::<fn(f32)>(0x100381ad0)(200.0);

    // eq: CFont::SetProportional(...)
    crate::hook::slide::<fn(u8)>(0x100381b84)(0);

    // eq: CFont::SetFontStyle(...)
    crate::hook::slide::<fn(u8)>(0x100381a20)(1);

    // eq: CFont::SetEdge(...)
    crate::hook::slide::<fn(u8)>(0x100381b58)(0);

    // eq: CFont::SetColor(...)
    crate::hook::slide::<fn(*const RGBA)>(0x100381824)(&RGBA {
        red: 9,
        green: 243,
        blue: 11,
        alpha: 255,
    });

    let fps = unsafe {
        let delta_last_frame = *delta_times.offset((*current_delta - 1) % 40);
        let delta_this_frame = *delta_times.offset(*current_delta % 40);
        let delta = delta_last_frame - delta_this_frame;

        39000.0 / delta as f32
    };

    // CFont::PrintString expects UTF16, so encode our FPS string as such.
    let mut bytes: Vec<u16> = format!("FPS: {:.2}", fps).encode_utf16().collect();
    bytes.push(0);

    let (x, y) = unsafe {
        let screen_wide = *crate::hook::slide::<*const i32>(0x1008f07b0);
        let screen_high = *crate::hook::slide::<*const i32>(0x1008f07b4);

        (screen_wide as f32 * 0.5, screen_high as f32 * 0.05)
    };

    // eq: CFont::PrintString(...)
    crate::hook::slide::<fn(f32, f32, *const u16)>(0x1003809c8)(x, y, bytes.as_ptr());
}

crate::hooks! (

fn write_fragment_shader(mask: u32) -> () as 0x100137528 {
    crate::original!(mask);

    let real_address = crate::hook::slide::<*mut u8>(0x100934e68);

    unsafe {
        let shader = std::ffi::CStr::from_ptr(real_address.cast())
            .to_str()
            .unwrap_or("unable to get value")
            .to_string();

        log::trace!("shader: {}", shader);

        let c_string = std::ffi::CString::new(shader).expect("CString::new failed");
        let bytes = c_string.as_bytes_with_nul();

        for i in 0..bytes.len() {
            real_address.offset(i as isize).write(bytes[i]);
        }
    }
}

);

crate::hooks! (

fn set_loading_messages(msg_1: *const c_char, msg_2: *const c_char) -> () as 0x1002b5a78 {
    unsafe {
        let msg_1 = std::ffi::CStr::from_ptr(msg_1).to_str().unwrap_or("???");
        let msg_2 = std::ffi::CStr::from_ptr(msg_2).to_str().unwrap_or("???");

        log::info!("{}: {}", msg_1, msg_2);
    }
}

);

pub fn hook() {
    targets::idle::install(idle);
    targets::cycles_per_millisecond::install(cycles_per_millisecond);
    // targets::write_fragment_shader::install(write_fragment_shader);
    targets::display_fps::install(display_fps);
    // targets::loading_messages::install(set_loading_messages);
}
