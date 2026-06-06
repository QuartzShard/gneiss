use std::collections::{HashMap, HashSet};

use crate::stm_crc32::crc;

extern crate std;

#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C, packed)]
pub struct PackManifest {
	num_files: u32,
	crc:       u32,
	// From pbpack.py:
	// Note that we never actually set the timestamp in newly generated pbpacks. The timestamp
	// field in the manifest is unused in practice and we'll probably reuse that field for
	// another purpose in the future.
	timestamp: u32,
}

impl PackManifest {
	pub fn num_files(&self) -> u32 {
		self.num_files
	}

	pub fn crc(&self) -> u32 {
		self.crc
	}

	pub fn timestamp(&self) -> u32 {
		self.timestamp
	}
}

// Defensive, matching pbpack.py: MANIFEST_SIZE_BYTES
const _: () = assert!(size_of::<PackManifest>() == 12);

#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C, packed)]
pub struct PackTable([PackTableEntry; 256]);

#[derive(Debug, Default, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C, packed)]
pub struct PackTableEntry {
	file_id: u32,
	offset:  u32,
	length:  u32,
	crc:     u32,
}

#[derive(Debug)]
#[repr(C)]
pub struct PbPack {
	manifest: PackManifest,
	table:    PackTable,
	content:  Vec<u8>,
}

impl PbPack {
	/// Construct a new PbPack from a list of resource blobs
	pub fn new(resources: &[&[u8]]) -> PbPack {
		assert!(
			resources.len() <= 256,
			"Max Resources = 256, You declared {}",
			resources.len()
		);
		let mut table = PackTable([PackTableEntry::default(); 256]);

		// 2 pass for deduplication:
		// - Forward populates the table but not the content blob or offsets,
		let mut seen = HashSet::new();
		let mut content_offset = 0;
		for ((res, row), file_id) in resources.iter().zip(table.0.iter_mut()).zip(1u32..) {
			*row = PackTableEntry {
				file_id,
				// TBD
				offset: 0,
				length: res.len() as u32,
				crc: crc(res),
			};
			// Only count if it's new
			if seen.insert(res) {
				content_offset += res.len()
			};
		}

		// - Reverse populates offsets and blob, deduplicating if necessary.
		let mut content = vec![0u8; content_offset];
		let mut uniq_content: HashMap<&[u8], u32> = HashMap::new();
		for (res, row) in resources.iter().zip(table.0.iter_mut()).rev() {
			row.offset = match uniq_content.get(res) {
				// The exact same blob already exists, just point at that
				Some(offset) => *offset,
				// Fill from the back, so that table[-1] is as the highest offset.
				None => {
					content_offset -= res.len();
					let _ = uniq_content.insert(res, content_offset as u32);
					content[content_offset..content_offset + res.len()].copy_from_slice(res);
					content_offset as u32
				}
			};
		}

		PbPack {
			manifest: PackManifest {
				num_files: resources.len() as u32,
				crc:       crc(&content),
				timestamp: 0,
			},
			table,
			content,
		}
	}

	pub fn as_bytes(&self) -> Vec<u8> {
		let mut buf = Vec::with_capacity(
			size_of::<PackManifest>() + size_of::<PackTable>() + self.content.len(),
		);
		buf.extend_from_slice(bytemuck::bytes_of(&self.manifest));
		buf.extend_from_slice(bytemuck::bytes_of(&self.table));
		buf.extend_from_slice(&self.content);
		buf
	}

	pub fn manifest(&self) -> PackManifest {
		self.manifest
	}

	pub fn table(&self) -> PackTable {
		self.table
	}
}
