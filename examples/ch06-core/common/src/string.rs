use crate::memcpy;
use core::ffi::CStr;

pub fn strcpy(dst: &mut [u8], src: &CStr) -> Option<()> {
    let bytes = src.to_bytes();
    if dst.len() <= bytes.len() {
        return None;
    }
    memcpy(&mut dst[..bytes.len()], bytes);
    dst[bytes.len()] = 0;
    Some(())
}

pub fn strcmp(left: &CStr, right: &CStr) -> i32 {
    for (&a, &b) in left
        .to_bytes_with_nul()
        .iter()
        .zip(right.to_bytes_with_nul())
    {
        if a != b {
            return i32::from(a) - i32::from(b);
        }
    }
    0
}
