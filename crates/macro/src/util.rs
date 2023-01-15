use std::iter;

pub fn tag_length(bytes: &mut Vec<u8>) {
    let mut temp = Vec::new();
    leb128::write::unsigned(&mut temp, bytes.len() as _).unwrap();
    prepend(bytes, &temp);
}

pub fn prepend(bytes: &mut Vec<u8>, data: &[u8]) {
    let l = bytes.len();
    let ld = data.len();
    bytes.extend(iter::repeat(0).take(ld));
    bytes.copy_within(0..l, ld);
    bytes[0..ld].copy_from_slice(data);
}
