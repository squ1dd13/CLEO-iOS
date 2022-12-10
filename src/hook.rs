//! Handles finding a hooking library, and provides types and macros for using the library
//! to hook game code.

use cached::proc_macro::cached;
use dlopen::symbor::Library;
use eyre::Context;
use log::error;

const GAME_SLIDE_INDEX: u32 = 1;

fn get_single_symbol<T: Copy>(path: &str, sym_name: &str) -> eyre::Result<T> {
    let lib = Library::open(path).wrap_err_with(|| format!("failed to open library {}", path))?;
    let symbol = unsafe { lib.symbol::<T>(sym_name) }
        .wrap_err_with(|| format!("unable to find {} in {}", sym_name, path))?;
    Ok(*symbol)
}

#[cached(result = true)]
fn get_raw_hook_fn() -> eyre::Result<usize> {
    get_single_symbol("libsubstrate.dylib", "MSHookFunction")
}

#[cached(result = true)]
fn get_shit_raw_hook_fn() -> eyre::Result<usize> {
    get_single_symbol("libhooker.dylib", "LHHookFunctions")
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

/// Returns `true` if image 0's slide is not the slide for game code.
pub fn has_weird_aslr() -> bool {
    get_aslr_offset(0) != get_game_aslr_offset()
}

pub fn get_game_aslr_offset() -> usize {
    // Pre iOS 15, game code uses the ALSR slide for image 0. From iOS 15, we have to use image 1.
    // The value we need is always the smaller of the two, so find that instead of checking the iOS
    // version.
    get_aslr_offset(0).min(get_aslr_offset(1))
}

#[repr(C)]
enum ShitHookError {
    Ok,
    SelNotFound,
    FuncTooShort,
    BadInstruction,
    PagingError,
    NoSymbol,
}

impl std::fmt::Display for ShitHookError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            ShitHookError::Ok => "hook successful",
            ShitHookError::SelNotFound => "objective-c selector not found",
            ShitHookError::FuncTooShort => "function too short to hook",
            ShitHookError::BadInstruction => "bad instruction found at start of function",
            ShitHookError::PagingError => "error handling memory pages",
            ShitHookError::NoSymbol => "no symbol given",
        };

        f.write_str(s)
    }
}

// Represents libhooker's struct LHFunctionHook.
#[repr(C)]
struct ShitFunctionHook<FuncType> {
    function: FuncType,
    replacement: FuncType,
    old_ptr: usize,
    options: usize,
}

fn gen_shit_hook_fn<FuncType>() -> fn(FuncType, FuncType, &mut Option<FuncType>) {
    |function, replacement, original| {
        let hook_struct = ShitFunctionHook {
            function,
            replacement,
            old_ptr: unsafe { std::mem::transmute(original) },
            options: 0,
        };

        unsafe {
            let hook_fn: fn(*const ShitFunctionHook<FuncType>, i32) -> ShitHookError =
                std::mem::transmute(get_shit_raw_hook_fn().expect("need a hook function"));
            let struct_ptr: *const ShitFunctionHook<FuncType> = &hook_struct;

            let res = hook_fn(struct_ptr, 1);

            // SelNotFound is the default return value: we hook one function and if there are no
            // errors the return value should be the number of functions hooked, which is 1, which
            // is SelNotFound. If the return value is something else, there's an error. Ignoring
            // SelNotFound is fine because LHHookFunctions does not deal with selectors and thus
            // could never return that error.
            if !matches!(res, ShitHookError::SelNotFound) {
                error!("hook failed: {}", res);
            }
        }
    }
}

fn get_hook_fn<FuncType>() -> fn(FuncType, FuncType, &mut Option<FuncType>) {
    let shit_hook = get_shit_raw_hook_fn();

    if let Ok(_) = shit_hook {
        return gen_shit_hook_fn();
    }

    let raw = get_raw_hook_fn().expect("get_hook_fn: get_raw_hook_fn failed");

    // Reinterpret cast the address to get a function pointer.
    // We get the address as a usize so that it can be cached once and then reused
    //  to get different signatures.
    // hack: manual transmute
    unsafe {
        let addr_ptr: *const usize = &raw;
        *(addr_ptr as *const fn(FuncType, FuncType, &mut Option<FuncType>))
    }
}

#[derive(Debug)]
pub enum Target<FuncType: std::fmt::Debug> {
    /// A function pointer.
    _Function(FuncType),

    /// A raw address, to which the ASLR offset for the current image will be applied.
    Address(usize),

    /// A raw address, to which the ASLR offset for the image given as the second parameter will be applied.
    _ForeignAddress(usize, u32),
}

impl<FuncType: std::fmt::Debug> Target<FuncType> {
    fn get_absolute(&self) -> usize {
        match self {
            Target::_Function(func) => unsafe { std::mem::transmute_copy(func) },

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

    pub fn hook_hard(&self, replacement: FuncType) {
        log::debug!("installing hard hook on target {:?}", self);
        get_hook_fn::<FuncType>()(self.get_as_fn(), replacement, &mut None);
    }

    pub fn hook_soft(&self, replacement: FuncType, original_out: &mut Option<FuncType>) {
        log::debug!("installing soft hook on target {:?}", self);
        get_hook_fn::<FuncType>()(self.get_as_fn(), replacement, original_out);
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

pub fn get_global<T: Copy>(address: usize) -> T {
    let slid: *const T = slide(address);
    unsafe { *slid }
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
        ) as bool;

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

pub fn is_german_game() -> bool {
    std::env::current_exe()
        .unwrap()
        .display()
        .to_string()
        .ends_with("ger")
}

pub fn generate_backtrace() -> String {
    // Generate a resolved backtrace. The symbol names aren't always correct, but we
    //  should still display them because they are helpful for Rust functions.
    let resolved = backtrace::Backtrace::new();
    let slide = get_game_aslr_offset() as u64;

    let mut lines = vec![
        format!("ASLR offset for image 0 is {:#x}.", slide),
        "Warning: All addresses will be assumed to be from image 0.".to_string(),
    ];

    for (i, frame) in resolved.frames().iter().enumerate() {
        let address = frame.symbol_address() as u64;

        let string = format!(
            "{}: {:#x} - {:#x} = {:#x}\n  symbols: {:?}",
            i,
            address,
            slide,
            address - slide,
            frame.symbols()
        );

        lines.push(string);
    }

    lines.join("\n\n")
}
