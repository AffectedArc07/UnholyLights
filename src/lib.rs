use byondapi::value::ByondValue;

pub mod lighting_object_functions;

#[no_mangle]
pub unsafe extern "C" fn init_lib(
    _argc: byondapi_sys::u4c,
    _argv: *mut ByondValue,
) -> ByondValue {
    return ByondValue::new_str("CUDAALights Lib Initialised").unwrap()
}
