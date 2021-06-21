use crate::{call_original, hook, targets};

// CTimer::GetCyclesPerMillisecond is called between the FPS limit being set and when it is enforced,
//  so if we overwrite the limit here, our new value will be enforced.
fn cycles_per_millisecond() -> u32 {
    unsafe {
        crate::settings::with_shared(&mut |options| {
            *hook::slide::<*mut u32>(0x1008f07b8) = if options[0].value { 60 } else { 30 };
        });
    }

    call_original!(targets::cycles_per_millisecond)
}

fn idle(p1: u64, p2: u64) {
    unsafe {
        crate::settings::with_shared(&mut |options| {
            *hook::slide::<*mut bool>(0x10081c519) = options[1].value;
        });
    }

    call_original!(targets::idle, p1, p2);
}

fn write_fragment_shader(mask: u32) {
    call_original!(crate::targets::write_fragment_shader, mask);

    let real_address = crate::hook::slide::<*mut u8>(0x100934e68);

    unsafe {
        let shader = std::ffi::CStr::from_ptr(real_address.cast())
            .to_str()
            .unwrap_or("unable to get value")
            .to_string();

        // Shader changes can be made here by replacing lines. (If CLEO ever does include
        //  any real ability for shader modding, it will be more refined than this.)

        let c_string = std::ffi::CString::new(shader).expect("CString::new failed");
        let bytes = c_string.as_bytes_with_nul();

        for i in 0..bytes.len() {
            real_address.offset(i as isize).write(bytes[i]);
        }
    }
}

pub fn hook() {
    targets::idle::install(idle);
    targets::cycles_per_millisecond::install(cycles_per_millisecond);
    targets::write_fragment_shader::install(write_fragment_shader);
}
