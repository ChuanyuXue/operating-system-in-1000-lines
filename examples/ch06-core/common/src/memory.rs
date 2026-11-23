pub fn memcpy(dst: &mut [u8], src: &[u8]) {
    assert_eq!(dst.len(), src.len());
    for (out, &byte) in dst.iter_mut().zip(src) {
        *out = byte;
    }
}

pub fn memset(dst: &mut [u8], value: u8) {
    for byte in dst {
        *byte = value;
    }
}
