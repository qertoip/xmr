// Copyright 2018 Jean Pierre Dudey <jeandudey@hotmail.com>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! # varint
//! XZ variable length integers reading/writing

// Nice explanation:
// * https://developers.google.com/protocol-buffers/docs/encoding#varints
// * https://tukaani.org/xz/xz-file-format-1.0.4.txt

use std::mem::size_of;

use bytes::{BytesMut, Buf, BufMut};
use num::cast::ToPrimitive;

pub const MOST_SIGNIFICANT_BIT: u8 = 0b10000000;
//const DROP_MSB: u8 = 0b01111111;
const EXTRACT_SEVEN_LEAST_SIGNIFICANT_BITS: u64 = 0b01111111;

/// Write an integer as a varint.
pub fn write<I: ToPrimitive>(output: &mut BytesMut, number: I) {
    let mut number = number.to_u64().expect("varint number must not be negative");
    while number > 127 {
        let byte = (number & EXTRACT_SEVEN_LEAST_SIGNIFICANT_BITS) as u8 | MOST_SIGNIFICANT_BIT;
        output.put_u8(byte);
        number >>= 7;
    }
    output.put_u8(number as u8);
}

/// Read a varint.
pub fn read<B: Buf>(buf: &mut B) -> Result<u64, ReadError> {
    let bits = (size_of::<u64>() * 8) as u64;
    let mut output = 0u64;
    let mut shift = 0u64;
    loop {
        let byte = buf.get_u8();

        if shift + 7 >= bits && byte >= 1 << (bits - shift) {
            return Err(ReadError::Overflow);
        }

        if byte == 0 && shift != 0 {
            return Err(ReadError::Represent);
        }

        // Does the actualy placing into output, stripping the first bit
        output |= ((byte & 0x7f) as u64) << shift;

        /* If there is no next */
        if (byte & 0x80) == 0 {
            break;
        }

        shift += 7;
    }

    Ok(output)
}

/// Calcuate how many bytes a varint occupies in memory.
pub fn length<I: ToPrimitive>(i: I) -> usize {
    let mut i = i.to_u64().unwrap();
    let mut count = 1;
    while i >= 0x80 {
        count += 1;
        i >>= 7;
    }
    count
}

/// An error occurred during reading.
#[derive(Debug, Clone, Copy)]
pub enum ReadError {
    /// The integer is too large to fit in the current type.
    Overflow,
    /// The integer cannot be represented.
    Represent,
}

impl std::fmt::Display for ReadError {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            ReadError::Overflow => write!(fmt, "the integer is too large"),
            ReadError::Represent => write!(fmt, "the integer cannot be represented"),
        }
    }
}


#[cfg(test)]
pub mod tests {
    use super::*;
    use std::u16;
    use bytes::{BytesMut, IntoBuf};

    // write negative

    #[test]
    #[should_panic]
    fn write_negative_should_panick() {
        write(&mut BytesMut::new(), -1);
    }

    // write 1 byte

    #[test]
    fn write_0() { assert_varint(0, &[0]); }

    #[test]
    fn write_1() { assert_varint(1, &[1]); }

    #[test]
    fn write_127() { assert_varint(127, &[127]); }

    // write 2 bytes

    #[test]
    fn write_128() { assert_varint(128, &[0b1000_0000, 0b0000_0001]); }

    #[test] // https://developers.google.com/protocol-buffers/docs/encoding#varints
    fn write_300() { assert_varint(300, &[0b1010_1100, 0b0000_0010]); }

    #[test]
    fn write_largest_int_fitting_in_2_bytes() { assert_varint(16383, &[255, 127]); }

    // write 3 bytes

    #[test]
    fn write_smallest_int_requiring_3_bytes() { assert_varint(16384, &[128, 128, 1]); }

    #[test]
    fn write_largest_int_fitting_in_3_bytes() { assert_varint(2097151, &[255, 255, 127]); }

    // write 4 bytes

    #[test]
    fn write_smallest_int_requiring_4_bytes() { assert_varint(2097152, &[128, 128, 128, 1]); }

    #[test]
    fn write_largest_int_fitting_in_4_bytes() { assert_varint(268435455, &[255, 255, 255, 127]); }

    // write 5 bytes

    #[test]
    fn write_smallest_int_requiring_5_bytes() { assert_varint(268435456, &[128, 128, 128, 128, 1]); }

    #[test]
    fn write_example_5_byte_varint() { assert_varint(29359738367 as u64, &[255, 155, 232, 175, 109]); }

    #[test]
    fn write_largest_int_fitting_in_5_bytes() { assert_varint(34359738367 as u64, &[255, 255, 255, 255, 127]); }

    // write u64::MAX

    #[test]
    fn write_u64_max() { assert_varint(std::u64::MAX, &[255, 255, 255, 255, 255, 255, 255, 255, 255, 1]); }

    #[test]
    fn read_write_is_equal() {
        let mut write_buf = BytesMut::new();
        for input in 0..u16::MAX {
            write(&mut write_buf, input as u16);
            {
                let mut read_buf = write_buf.as_ref().into_buf();
                let output = read(&mut read_buf).expect("reading should be fine") as u16;
                assert_eq!(input, output);
            }
            write_buf.clear();
        }
    }

    fn assert_varint<T: ToPrimitive>(n: T, bytes: &[u8]) {
        let mut buf = BytesMut::new();
        write(&mut buf, n);
        assert_eq!(buf, bytes);
    }

}
