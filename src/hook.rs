use cached::proc_macro::cached;
use dlopen::symbor::Library;

pub fn get_single_symbol<T: Copy>(path: &str, sym_name: &str) -> Result<T, dlopen::Error> {
    let lib = Library::open(path)?;
    let symbol = unsafe { lib.symbol::<T>(sym_name) }?;
    Ok(*symbol)
}

#[cached(result = true)]
fn get_raw_hook_fn() -> Result<usize, dlopen::Error> {
    // todo: Change hook procedure for libhooker.
    const HOOK_LIB_NAME: &str = "libsubstrate.dylib";
    const HOOK_FUNC_NAME: &str = "MSHookFunction";

    get_single_symbol(HOOK_LIB_NAME, HOOK_FUNC_NAME)
}

#[cached(result = true)]
fn get_aslr_offset_fn() -> Result<fn(u32) -> usize, dlopen::Error> {
    get_single_symbol::<fn(image: u32) -> usize>(
        "/usr/lib/system/libdyld.dylib",
        "_dyld_get_image_vmaddr_slide",
    )
}

#[cached]
pub fn get_image_aslr_offset(image: u32) -> usize {
    let function = get_aslr_offset_fn()
        .expect("Failed to get ASLR offset function! All base addresses will be invalid.");
    function(image)
}

fn get_hook_fn<FuncType>() -> fn(FuncType, FuncType, &mut Option<FuncType>) {
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
    Function(FuncType),

    /// A raw address, to which the ASLR offset for the current image will be applied.
    Address(usize),

    /// A raw address, to which the ASLR offset for the image given as the second parameter will be applied.
    ForeignAddress(usize, u32),
}

impl<FuncType> Target<FuncType> {
    fn get_absolute(&self) -> usize {
        match self {
            Target::Function(func) => unsafe { std::mem::transmute_copy(func) },

            Target::Address(addr) => {
                let aslr_offset = get_image_aslr_offset(0);
                addr + aslr_offset
            }

            Target::ForeignAddress(addr, image) => {
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
        pub mod $name {
            use super::*;

            const TARGET: crate::hook::Target<$sig> = crate::hook::Target::Address($addr);

            pub fn install(replacement: $sig) {
                TARGET.hook_hard(replacement);
            }
        }
    };
}

#[macro_export]
macro_rules! create_soft_target {
    ($name:ident, $addr:literal, $sig:ty) => {
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
        unsafe { base::ORIGINAL }.unwrap()()
    }};
    ($hook_module:path, $($args:expr),+) => {{
        // Workaround for $hook_module::x not working - see #48067.
        use $hook_module as base;
        unsafe { base::ORIGINAL }.unwrap()($($args),+)
    }}
}

pub fn slide<T: Copy>(address: usize) -> T {
    unsafe {
        let addr_ptr: *const usize = &(address + crate::hook::get_image_aslr_offset(0));
        *(addr_ptr as *const T)
    }
}
