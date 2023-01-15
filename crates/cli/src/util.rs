use std::io::Read;
use std::iter;

use anyhow::Error;

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

pub fn read_tagged<T, F, R>(reader: &mut T, f: F) -> Result<R, Error>
where
    T: Read,
    for<'a> F: FnOnce(&'a mut dyn Read) -> Result<R, Error>,
{
    let len = leb128::read::unsigned(&mut *reader)?.try_into()?;
    f(&mut reader.take(len))
}
