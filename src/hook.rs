use cached::proc_macro::cached;
use dlopen::symbor::Library;

pub fn get_single_symbol<T: Copy>(path: &str, sym_name: &str) -> Result<T, dlopen::Error> {
    let lib = Library::open(path)?;
    let symbol = unsafe { lib.symbol::<T>(sym_name) }?;
    Ok(*symbol)
}

#[cached(result = true)]
fn get_raw_hook_fn() -> Result<usize, dlopen::Error> {
    const HOOK_LIB_NAME: &str = "libsubstrate.dylib";
    const HOOK_FUNC_NAME: &str = "MSHookFunction";

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
    // Get a hook function for the correct signature and call it.
    get_hook_fn::<FunctionType>()(target, replacement, original_out);
}
