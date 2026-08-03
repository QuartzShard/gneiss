//! SPDX-License-Identifier: Apache-2.0
//! Faithful port of `stm32_crc.py`

extern crate std;
use std::borrow::Cow;

const CRC_POLY: u32 = 0x04C1_1DB7;
const CRC_TABLE: [u32; 256] = compute_table::<8>();
const fn compute_table<const BITS: usize>() -> [u32; 1 << BITS] {
	let mut table = [0; 1 << BITS];
	let mut i: usize = 0;
	while i < (1 << BITS) {
		let mut rr = (i as u32) << (32 - BITS);
		let mut j = 0;
		while j < BITS {
			if rr & 0x8000_0000 != 0 {
				rr = (rr << 1) ^ CRC_POLY;
			} else {
				rr <<= 1;
			}
			j += 1;
		}
		table[i] = rr;
		i += 1;
	}
	table
}

fn process_word(mut crc: u32, mut data: Cow<[u8]>) -> u32 {
	if data.len() < 4 {
		let mut padded = vec![0; 4];
		data.to_mut().reverse();
		padded[..data.len()].copy_from_slice(&data);
		data = padded.into();
	}
	for b in data.iter().rev() {
		crc = (crc << 8) ^ CRC_TABLE[((crc >> 24) ^ u32::from(*b)) as usize];
	}
	crc
}

pub fn crc(buf: &[u8]) -> u32 {
	buf.chunks(4)
		.map(Cow::Borrowed)
		.fold(0xFFFF_FFFF, process_word)
}

#[cfg(test)]
mod test {
	use super::crc;
	#[test]
	fn matches_py_impl() {
		assert_eq!(0x89F3_BAB2, crc(b"123 567 901 34"));
		assert_eq!(0xAFF1_9057, crc(b"123456789"));
		assert_eq!(0x0519_B130, crc(b"\xfe\xff\xfe\xff"));
		assert_eq!(0x495E_02CA, crc(b"\xfe\xff\xfe\xff\x88"));
	}
}
