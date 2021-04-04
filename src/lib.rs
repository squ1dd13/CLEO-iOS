mod hook;
mod logging;

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

    let symbol = hook::get_single_symbol::<fn(image_index: u32) -> usize>(
        "/usr/lib/system/libdyld.dylib",
        "_dyld_get_image_vmaddr_slide",
    );

    if let Ok(aslr_slide_fn) = symbol {
        log.normal(format!("get_single_symbol returned {:?}", aslr_slide_fn));
        log.normal(format!("return value of GSS function is {:?}", unsafe {
            aslr_slide_fn(0)
        }));

        static mut ORIGINAL: Option<fn(u32) -> usize> = None;

        fn replacement(image_index: u32) -> usize {
            get_log().normal("hooked!");
            get_log().normal(format!("called for index {}", image_index));
            unsafe { ORIGINAL.unwrap()(image_index) }
        }

        hook::install(aslr_slide_fn, replacement, unsafe { &mut ORIGINAL });
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
