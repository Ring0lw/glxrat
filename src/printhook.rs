use core::ffi::VaList;
use std::ffi::{CStr, c_char, c_int, c_void};

unsafe extern "C" {
    static mut stdout: *mut c_void;
    fn fputs(s: *const c_char, stream: *mut c_void) -> c_int;
    fn __vprintf_chk(flag: c_int, fmt: *const c_char, ap: VaList) -> c_int;
}

const ratline: &CStr = c"RAT INJECTED!!!!\n";
const ratbanner: &CStr = c"Running synchronized to the rat.  The ratrate should be\n";

fn mentions(fmt: *const c_char, needle: &[u8]) -> bool {
    if fmt.is_null() {
        return false;
    }
    let text = unsafe { CStr::from_ptr(fmt) }.to_bytes();
    text.windows(needle.len()).any(|w| w == needle)
}

unsafe fn shout(line: &CStr) -> c_int {
    unsafe {
        let out = stdout;
        fputs(line.as_ptr(), out);
    }
    line.count_bytes() as c_int
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn __printf_chk(flag: c_int, fmt: *const c_char, args: ...) -> c_int {
    if mentions(fmt, b"FPS") {
        return unsafe { shout(ratline) };
    }
    if mentions(fmt, b"vertical refresh") {
        return unsafe { shout(ratbanner) };
    }
    unsafe { __vprintf_chk(flag, fmt, args) }
}
