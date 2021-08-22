pub(crate) fn bytes_to_number(bytes: &[u8]) -> usize {
    let mut arr = [0;8];
    bytes.into_iter().enumerate().for_each(|(idx, value)| arr[idx] = *value);
    usize::from_le_bytes(arr)
}

pub(crate) fn bytes_to_float(bytes: &[u8]) -> f64{
    let mut arr = [0;8];
    bytes.into_iter().enumerate().for_each(|(idx, value)| arr[idx] = *value);
    f64::from_le_bytes(arr)
}
