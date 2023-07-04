use libafl::inputs::{HasBytesVec, BytesInput};

pub(crate) fn bytes_input_to_u32(bytes_input: &BytesInput) -> u32 {
    let mut array_input = [0u8; 4];
    for (dst, src) in array_input.iter_mut().zip(bytes_input.bytes()) {
        *dst = *src
    }
    u32::from_be_bytes(array_input)
}
