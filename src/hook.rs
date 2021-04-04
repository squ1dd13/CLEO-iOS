use cached::proc_macro::cached;
use dlopen::symbor::{FromRawResult, Library};

pub fn get_single_symbol<T: Copy>(path: &str, sym_name: &str) -> Result<T, dlopen::Error> {
    let lib = Library::open(path)?;
    let symbol = unsafe { lib.symbol::<T>(sym_name) }?;
    Ok(*symbol)
}

const HOOK_LIB_NAME: &str = "libsubstrate.dylib";
const HOOK_FUNC_NAME: &str = "MSHookFunction";

#[cached(result = true)]
fn get_raw_hook_fn() -> Result<usize, dlopen::Error> {
    get_single_symbol(HOOK_LIB_NAME, HOOK_FUNC_NAME)
}

pub fn get_hook_fn<FunctionType>() -> fn(FunctionType, FunctionType, &Option<FunctionType>) {
    let raw = get_raw_hook_fn().expect("get_hook_fn: get_raw_hook_fn failed");

    // Reinterpret cast the address to get a function pointer.
    // We get the address as a usize so that it can be cached once and then reused
    //  to get different signatures.
    let addr_ptr: *const usize = &raw;
    unsafe { *(addr_ptr as *const fn(FunctionType, FunctionType, &Option<FunctionType>)) }
}

pub fn install<FunctionType>(
    target: FunctionType,
    replacement: FunctionType,
    original_out: &mut Option<FunctionType>,
) {
    get_hook_fn::<FunctionType>()(target, replacement, original_out);
}

/*

let target = ...;

let hook = Hook::<fn(u32) -> u32>::new(target);
hook.body = |number: u32| {
    let orig_ret = hook.original(number);
    orig_ret + 2
}

hook.install();

* or *

hook!(target, |number: u32| {
    ...
});

*/

// pub struct Hook<'a, TargetType> {
//     target: &'a TargetType,
//     pub body: Option<&'a TargetType>,
//     pub original: &'a TargetType,
// }

// impl<TargetType> Hook<'_, TargetType> {
//     pub fn new<'a>(target: &'a TargetType) -> Hook<TargetType> {
//         Hook::<TargetType> {
//             target,
//             body: None,
//             original: target,
//         }
//     }

//     pub fn install(&self) {
//         let body = self.body.unwrap();
//         self.original = hook_saved(&self.target, &body);
//     }
// }

// pub fn hook_saved<FunctionType>(target: &FunctionType, replacement: &FunctionType) -> FunctionType {
//     let original: Option<FunctionType> = None;
//     get_hook_fn::<FunctionType>()(*target, *replacement, &original);

//     original.unwrap()
// }

// pub fn get_image_aslr_slide(index: u32) -> usize {
//     static mut c_fn: Option<unsafe extern "C" fn(u32) -> usize> = None;

//     if let None = c_fn {}

//     let lib = Library::open("/usr/lib/system/libdyld.dylib");

//     if let Err(error) = lib {
//         log.error(format!("Error loading dyld: {}", error));
//     } else {
//         let lib = lib.unwrap();

//         let fun = unsafe {
//             lib.symbol::<unsafe extern "C" fn(image_index: u32) -> usize>(
//                 "_dyld_get_image_vmaddr_slide",
//             )
//         }
//         .unwrap();

//         log.normal(format!("dl_result = {:?}", fun));
//         log.normal(format!("aslr offset = {}", unsafe { fun(0) }));
//     }
// }
