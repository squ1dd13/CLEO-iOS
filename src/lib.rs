use backtrace::Backtrace;
use dlopen::symbor::Library;
use std::ffi::{CStr, CString};
use std::os::raw::c_char;

mod hook;
mod logging;

#[no_mangle]
pub extern "C" fn rust_greeting(to: *const c_char) -> *mut c_char {
    let c_str = unsafe { CStr::from_ptr(to) };

    let recipient = match c_str.to_str() {
        Err(_) => "there",
        Ok(string) => string,
    };

    CString::new("Hello ".to_owned() + recipient)
        .unwrap()
        .into_raw()
}

/*
let lib = Library::open("libsubstrate.dylib");

if let Err(error) = lib {
    log.error(format!("Error loading Substrate: {}", error));
} else {
    let lib = lib.unwrap();

    let fun = unsafe {
        lib.symbol::<unsafe extern "C" fn(symbol: usize, replace: usize, result: *mut usize)>(
            "MSHookFunction",
        )
    }
    .unwrap();

    log.normal(format!("dl_result = {:?}", fun));
}
*/

static mut STATIC_LOG: Option<logging::Logger> = None;

fn get_log() -> &'static mut logging::Logger {
    unsafe { STATIC_LOG.as_mut() }.unwrap()
}

#[ctor::ctor]
fn init() {
    unsafe { STATIC_LOG = Some(logging::Logger::new("tweak")) };

    let log = get_log();

    log.connect_udp("192.168.1.183:4568");
    log.connect_file("/var/mobile/Documents/tweak.log");

    // Log an empty string so we get a break after the output from the last run.
    log.normal("");

    let lib = Library::open("/usr/lib/system/libdyld.dylib");

    if let Err(error) = lib {
        log.error(format!("Error loading dyld: {}", error));
    } else {
        let lib = lib.unwrap();

        let fun = unsafe {
            lib.symbol::</*unsafe extern "C" */fn(image_index: u32) -> usize>(
                "_dyld_get_image_vmaddr_slide",
            )
        }
        .unwrap();

        log.normal(format!("dl_result = {:?}", fun));
        log.normal(format!("aslr offset = {}", unsafe { fun(0) }));
    }

    let symbol = hook::get_single_symbol::<fn(image_index: u32) -> usize>(
        "/usr/lib/system/libdyld.dylib",
        "_dyld_get_image_vmaddr_slide",
    );

    if let Ok(aslr_slide_fn) = symbol {
        log.normal(format!("get_single_symbol returned {:?}", aslr_slide_fn));
        log.normal(format!("return value of GSS function is {:?}", unsafe {
            aslr_slide_fn(0)
        }));

        let mshf_sym = hook::get_single_symbol::<
            /*unsafe extern "C" */
            fn(
                /*unsafe extern "C" */ fn(image_index: u32) -> usize,
                /*unsafe extern "C" */ fn(image_index: u32) -> usize,
                &'static Option</*unsafe extern "C" */ fn(image_index: u32) -> usize>,
            ),
        >("libsubstrate.dylib", "MSHookFunction");

        if let Ok(ms_hook_function) = mshf_sym {
            log.normal(format!("get_single_symbol returned {:?}", ms_hook_function));

            static mut original: Option<fn(u32) -> usize> = None;

            fn replacement(image_index: u32) -> usize {
                get_log().normal("hooked!");
                get_log().normal(format!("called for index {}", image_index));
                unsafe { original.unwrap()(image_index) }
            }

            hook::install(aslr_slide_fn, replacement, unsafe { &mut original });
        } else {
            log.error(format!(
                "get_single_symbol failed: {}",
                mshf_sym.err().unwrap()
            ))
        }
    } else {
        log.error(format!(
            "get_single_symbol failed: {}",
            symbol.err().unwrap()
        ))
    }

    log.normal("Test plain string");
    log.warning("Test warning");
    log.error("Test error");
    log.important("Test important");
}

#[no_mangle]
pub extern "C" fn rust_greeting_free(s: *mut c_char) {
    unsafe {
        if s.is_null() {
            return;
        }
        CString::from_raw(s)
    };
}
