//! Handles finding a hooking library, and provides types and macros for using the library
//! to hook game code.

use cached::proc_macro::cached;
use dlopen::symbor::Library;
use eyre::Context;

fn get_single_symbol<T: Copy>(path: &str, sym_name: &str) -> eyre::Result<T> {
    let lib = Library::open(path).wrap_err_with(|| format!("failed to open library {path}"))?;
    let symbol = unsafe { lib.symbol::<T>(sym_name) }
        .wrap_err_with(|| format!("unable to find {sym_name} in {path}"))?;
    Ok(*symbol)
}

#[cached(result = true)]
fn get_aslr_offset_fn() -> eyre::Result<fn(u32) -> usize> {
    get_single_symbol::<fn(image: u32) -> usize>(
        "/usr/lib/system/libdyld.dylib",
        "_dyld_get_image_vmaddr_slide",
    )
}

#[cached]
fn get_aslr_offset(image: u32) -> usize {
    let function = get_aslr_offset_fn().expect("Failed to get ASLR offset function!");
    function(image)
}

pub fn get_game_aslr_offset() -> usize {
    // Pre iOS 15, game code uses the ALSR slide for image 0. From iOS 15, we have to use image 1.
    // The value we need is always the smaller of the two, so find that instead of checking the iOS
    // version.
    get_aslr_offset(0).min(get_aslr_offset(1))
}

#[derive(Debug)]
pub enum Target<FuncType: std::fmt::Debug> {
    /// A function pointer.
    _Function(FuncType),

    /// A raw address to which no ASLR slide will be applied.
    NoSlideAddress(usize),

    /// A raw address, to which the ASLR offset for the current image will be applied.
    Address(usize),

    /// A raw address, to which the ASLR offset for the image given as the second parameter will be applied.
    _ForeignAddress(usize, u32),
}

impl<FuncType: std::fmt::Debug> Target<FuncType> {
    fn get_absolute(&self) -> usize {
        match self {
            Target::_Function(func) => unsafe { std::mem::transmute_copy(func) },

            Target::NoSlideAddress(addr) => *addr,

            Target::Address(addr) => {
                let aslr_offset = get_game_aslr_offset();
                addr + aslr_offset
            }

            Target::_ForeignAddress(addr, image) => {
                let aslr_offset = get_aslr_offset(*image);
                addr + aslr_offset
            }
        }
    }

    fn get_as_fn(&self) -> FuncType {
        unsafe { std::mem::transmute_copy(&self.get_absolute()) }
    }

    pub fn hook_hard(&self, replacement: FuncType)
    where
        FuncType: Copy,
    {
        log::debug!("installing hard hook on target {:?}", self);
        unsafe {
            let _ = hlhook::install_hook(self.get_as_fn(), replacement).unwrap();
        }
    }

    pub fn hook_soft(&self, replacement: FuncType, original_out: &mut Option<FuncType>)
    where
        FuncType: Copy,
    {
        log::debug!("installing soft hook on target {:?}", self);
        unsafe {
            let trampoline = hlhook::install_hook(self.get_as_fn(), replacement).unwrap();
            *original_out = Some(trampoline);
        }
    }
}

#[macro_export]
macro_rules! create_hard_target {
    ($name:ident, $addr:literal, $sig:ty) => {
        #[allow(dead_code)]
        pub mod $name {
            #[allow(unused_imports)]
            use super::*;

            const TARGET: $crate::hook::Target<$sig> = $crate::hook::Target::Address($addr);

            pub fn install(replacement: $sig) {
                TARGET.hook_hard(replacement);
            }
        }
    };
}

#[macro_export]
macro_rules! create_soft_target {
    ($name:ident, $addr:literal, $sig:ty) => {
        #[allow(dead_code)]
        pub mod $name {
            #[allow(unused_imports)]
            use super::*;

            const TARGET: $crate::hook::Target<$sig> = $crate::hook::Target::Address($addr);
            pub static mut ORIGINAL: Option<$sig> = None;

            pub fn install(replacement: $sig) {
                TARGET.hook_soft(replacement, unsafe { &mut ORIGINAL });
            }
        }
    };
}

#[macro_export]
macro_rules! deref_original {
    ($orig_name:expr) => {
        unsafe { $orig_name.unwrap() }
    };
}

#[macro_export]
macro_rules! call_original {
    ($hook_module:path) => {{
        use $hook_module as base;
        #[allow(unused_unsafe)]
        unsafe { base::ORIGINAL }.unwrap()()
    }};
    ($hook_module:path, $($args:expr),+) => {{
        // Workaround for $hook_module::x not working - see #48067.
        use $hook_module as base;
        #[allow(unused_unsafe)]
        unsafe { base::ORIGINAL }.unwrap()($($args),+)
    }}
}

pub fn slide<T: Copy>(address: usize) -> T {
    let slide = crate::hook::get_game_aslr_offset();

    unsafe {
        let addr_ptr: *const usize = &(address + slide);
        *(addr_ptr as *const T)
    }
}

pub fn deref_global<T: Copy>(address: usize) -> T {
    let slid: *const T = slide(address);
    unsafe { *slid }
}

/// Returns `true` if CLEO is able to hook functions.
pub fn can_hook() -> bool {
    // note: `test_function` and `hooked_impl` cannot simply return a single value, because this
    // makes them too small to hook. A log message is included in each to add some extra
    // instructions.

    fn test_function() -> bool {
        log::warn!("unhooked implementation running");

        // This will not run if we hook it successfully.
        false
    }

    if test_function() {
        // Test function already hooked, so we assume that hooking works (since it clearly worked
        // at one point). This would be invalid if the hooking library worked once but doesn't work
        // again for some reason, but that's unlikely to happen.
        return true;
    }

    fn hooked_impl() -> bool {
        log::info!("hooked implementation running :)");

        true
    }

    // Try to hook the test function so that it returns `true`.
    Target::NoSlideAddress(test_function as usize).hook_hard(hooked_impl as usize);

    // If the hook failed, this will return `false` as normal.
    test_function()
}

// hack: this is a really shit API. it's just here until `hook` gets rewritten from the ground up.
pub fn hook_objc(
    class_name: impl AsRef<str>,
    selector: impl AsRef<str>,
    orig_selector: impl AsRef<str>,
    replacement: *const (),
) {
    let class_name = class_name.as_ref();
    let selector = selector.as_ref();
    let orig_selector = orig_selector.as_ref();

    log::debug!(
        "Hooking [{class_name} {selector}] with implementation {:#x} (orig -> {orig_selector})",
        replacement as usize
    );

    unsafe {
        use objc::runtime::*;

        let class_name_c = std::ffi::CString::new(class_name).unwrap();
        let selector_c = std::ffi::CString::new(selector).unwrap();
        let orig_selector_c = std::ffi::CString::new(orig_selector).unwrap();

        let class = objc::runtime::objc_getClass(class_name_c.into_raw());

        log::debug!("Found class {class_name}.");

        let target_sel = objc::runtime::sel_registerName(selector_c.into_raw());
        let renamed_sel = objc::runtime::sel_registerName(orig_selector_c.into_raw());

        log::debug!("Created selectors {selector}/{orig_selector}.");

        let target_meth = objc::runtime::class_getInstanceMethod(class, target_sel);
        let target_impl_addr = objc::runtime::method_getImplementation(target_meth) as usize;

        log::debug!(
            "Found target method with implementation {:#x}.",
            target_impl_addr,
        );

        let type_enc = objc::runtime::method_getTypeEncoding(target_meth);

        log::debug!(
            "Found target type encoding: {}",
            std::ffi::CStr::from_ptr(type_enc)
                .to_string_lossy()
                .as_ref()
        );

        let success = objc::runtime::class_addMethod(
            std::mem::transmute(class),
            renamed_sel,
            std::mem::transmute(replacement),
            type_enc as *const i8,
        ) == YES;

        if success {
            log::debug!("Added method for {orig_selector} to class.");
        } else {
            log::error!("Failed to add method for {orig_selector}.");
            return;
        }

        let new_meth = objc::runtime::class_getInstanceMethod(class, renamed_sel);

        objc::runtime::method_exchangeImplementations(
            target_meth as *mut Method,
            new_meth as *mut Method,
        );

        log::debug!("Successfully swapped method implementations. Enjoy!");
    }
}
