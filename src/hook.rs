//! Handles finding a hooking library, and provides types and macros for using the library
//! to hook game code.

use anyhow::{Context, Result};
use cached::proc_macro::cached;
use dlopen::symbor::Library;
use log::error;
use once_cell::sync::Lazy;
use parking_lot::Mutex;
use std::borrow::{Borrow, BorrowMut};
use std::cell::{Cell, RefCell, UnsafeCell};
use std::ops::{Deref, DerefMut};

fn get_single_symbol<T: Copy>(path: &str, sym_name: &str) -> Result<T> {
    let lib = Library::open(path).context("Failed to open hooking library")?;
    let symbol =
        unsafe { lib.symbol::<T>(sym_name) }.context("Unable to find symbol in library")?;
    Ok(*symbol)
}

#[cached(result = true)]
fn get_raw_hook_fn() -> Result<usize> {
    get_single_symbol("libsubstrate.dylib", "MSHookFunction")
}

#[cached(result = true)]
fn get_shit_raw_hook_fn() -> Result<usize> {
    get_single_symbol("libhooker.dylib", "LHHookFunctions")
}

#[cached(result = true)]
fn get_aslr_offset_fn() -> Result<fn(u32) -> usize> {
    get_single_symbol::<fn(image: u32) -> usize>(
        "/usr/lib/system/libdyld.dylib",
        "_dyld_get_image_vmaddr_slide",
    )
}

#[cached]
pub fn get_image_aslr_offset(image: u32) -> usize {
    let function = get_aslr_offset_fn().expect("Failed to get ASLR offset function!");
    function(image)
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
            let hook_fn: fn(*const ShitFunctionHook<FuncType>, i32) -> i32 =
                std::mem::transmute(get_shit_raw_hook_fn().expect("need a hook function"));
            let struct_ptr: *const ShitFunctionHook<FuncType> = &hook_struct;

            if hook_fn(struct_ptr, 1) != 1 {
                error!("Hook failed!");
            }
        }
    }
}

fn get_hook_fn<FuncType>() -> fn(FuncType, FuncType, &mut Option<FuncType>) {
    // Use libhooker if found.
    if get_shit_raw_hook_fn().is_ok() {
        return gen_shit_hook_fn();
    }

    let raw = get_raw_hook_fn().expect("get_hook_fn: get_raw_hook_fn failed");

    // Reinterpret cast the address to get a function pointer.
    // We get the address as a usize so that it can be cached once and then reused
    //  to get different signatures.
    unsafe {
        let addr_ptr: *const usize = &raw;
        *(addr_ptr as *const fn(FuncType, FuncType, &mut Option<FuncType>))
    }
}

pub enum Target<FuncType> {
    /// A function pointer.
    _Function(FuncType),

    /// A raw address, to which the ASLR offset for the current image will be applied.
    Address(usize),

    /// A raw address, to which the ASLR offset for the image given as the second parameter will be applied.
    _ForeignAddress(usize, u32),
}

impl<FuncType> Target<FuncType> {
    fn get_absolute(&self) -> usize {
        match self {
            Target::_Function(func) => unsafe { std::mem::transmute_copy(func) },

            Target::Address(addr) => {
                let aslr_offset = get_image_aslr_offset(0);
                addr + aslr_offset
            }

            Target::_ForeignAddress(addr, image) => {
                let aslr_offset = get_image_aslr_offset(*image);
                addr + aslr_offset
            }
        }
    }

    fn get_as_fn(&self) -> FuncType {
        unsafe { std::mem::transmute_copy(&self.get_absolute()) }
    }

    pub fn hook_hard(&self, replacement: FuncType) {
        get_hook_fn::<FuncType>()(self.get_as_fn(), replacement, &mut None);
    }

    pub fn hook_soft(&self, replacement: FuncType, original_out: &mut Option<FuncType>) {
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

            const TARGET: crate::hook::Target<$sig> = crate::hook::Target::Address($addr);

            pub fn install(replacement: $sig) {
                TARGET.hook_hard(replacement);
            }
        }
    };
}

pub struct Hook<FuncType> {
    address: usize,
    original_fn: Mutex<Option<FuncType>>,
}

impl<FnType> Hook<FnType> {
    /// Creates a new hook for a function with the specified address. This does not install the
    /// hook.
    pub const fn new(address: usize) -> Hook<FnType> {
        Hook {
            address,
            original_fn: Mutex::new(None),
        }
    }

    /// Returns the target address with the runtime ASLR slide. Assumes that the hooked function
    /// is in image `0`.
    fn address_with_slide(&self) -> usize {
        self.address + get_image_aslr_offset(0)
    }

    /// The same as `address_with_slide`, except that the return type is the target function type.
    fn address_fn(&self) -> FnType {
        unsafe { std::mem::transmute_copy(&self.address_with_slide()) }
    }

    /// Replaces the target function's implementation with that of the function given. The
    /// original function pointer can be obtained by calling `original()`.
    pub fn install(&self, replacement: FnType) {
        // Hook the function, storing a pointer to the original implementation.
        get_hook_fn::<FnType>()(
            self.address_fn(),
            replacement,
            self.original_fn.lock().deref_mut(),
        );
    }

    /// Returns a pointer to the original implementation of the hooked function.
    pub fn original(&self) -> FnType
    where
        FnType: Copy,
    {
        self.original_fn
            .lock()
            .expect("`original()` requires that an original function exists")
    }
}

#[macro_export]
macro_rules! create_soft_target {
    ($name:ident, $addr:literal, $sig:ty) => {
        #[allow(dead_code)]
        pub mod $name {
            #[allow(unused_imports)]
            use super::*;

            const TARGET: crate::hook::Target<$sig> = crate::hook::Target::Address($addr);
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

/// Hooks the function at a memory address. Does not provide a pointer to the original implementation.
pub fn hook<T>(address: usize, replacement: T) {
    crate::hook::Target::Address(address).hook_hard(replacement);
}

pub fn slide<T: Copy>(address: usize) -> T {
    unsafe {
        let addr_ptr: *const usize = &(address + crate::hook::get_image_aslr_offset(0));
        *(addr_ptr as *const T)
    }
}

pub fn get_global<T: Copy>(address: usize) -> T {
    let slid: *const T = slide(address);
    unsafe { *slid }
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
    let slide = get_image_aslr_offset(0) as u64;

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
