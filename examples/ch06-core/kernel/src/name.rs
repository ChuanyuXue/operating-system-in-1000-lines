pub fn copy_name(field: &mut [u8], name: &str) -> Option<usize> {
    let (text, padding) = field.split_at_mut_checked(name.len())?;
    text.copy_from_slice(name.as_bytes());
    padding.fill(0);
    Some(text.len())
}
