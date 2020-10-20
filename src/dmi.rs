mod chunk;
mod crc;

use super::error;
use std::convert::TryFrom;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

/// The PNG magic header
const MAGIC: [u8; 8] = [137, 80, 78, 71, 13, 10, 26, 10];

#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct Dmi {
	pub header: [u8; 8],
	pub chunk_ihdr: chunk::Chunk,
	pub chunk_ztxt: chunk::Chunk,
	pub chunk_idat: chunk::Chunk,
	pub chunk_iend: chunk::Chunk,
	pub other_chunks: Vec<chunk::Chunk>,
}

impl Dmi {
	pub fn write_ztxt_chunk(&mut self, new_text: String) -> Result<bool, error::ReadError> {
		let return_val = self.chunk_ztxt.write_ztxt_chunk(new_text)?;
		Ok(return_val)
	}

	pub fn dmi_to_vec(&mut self) -> Result<Vec<u8>, error::ReadError> {
		let mut dmi_bytes: Vec<u8> = vec![];

		dmi_bytes.extend_from_slice(&self.header);
		dmi_bytes = self.chunk_ihdr.write_to_vec(dmi_bytes)?;
		/* //Let's drop the other chunks for now. None of them are necessary for dmi files.
		for current_chunk in &mut self.other_chunks {
			dmi_bytes = current_chunk.write_to_vec(dmi_bytes)?;
		}
		*/
		dmi_bytes = self.chunk_ztxt.write_to_vec(dmi_bytes)?;
		dmi_bytes = self.chunk_idat.write_to_vec(dmi_bytes)?;
		dmi_bytes = self.chunk_iend.write_to_vec(dmi_bytes)?;
		Ok(dmi_bytes)
	}

	pub fn write(&mut self, path: &Path) -> Result<bool, error::ReadError> {
		let dmi_bytes: Vec<u8> = self.dmi_to_vec()?;

		let mut file = File::create(&path).map_err(|x| error::ReadError::Io(x))?;
		file.write_all(&dmi_bytes)
			.map_err(|x| error::ReadError::Io(x))?;

		Ok(true)
	}

	pub fn save(&mut self, path: String) -> Result<bool, error::ReadError> {
		let dmi_bytes: Vec<u8> = self.dmi_to_vec()?;

		let path = Path::new(&path);
		let mut file = File::create(&path).map_err(|x| error::ReadError::Io(x))?;
		file.write_all(&dmi_bytes)
			.map_err(|x| error::ReadError::Io(x))?;

		Ok(true)
	}
}

pub fn dmi_from_vec(bytes_vec: &[u8]) -> Result<Dmi, error::ReadError> {
	let header = <[u8; 8]>::try_from(&bytes_vec[0..8]).unwrap();

	if header != MAGIC {
		return Err(error::ReadError::MagicMismatch(header));
	}

	let mut index = 8; //Without the magic header.

	let mut chunk_ihdr = chunk::Chunk::default();
	let mut chunk_ztxt = chunk::Chunk::default();
	let mut chunk_idat = chunk::Chunk::default();
	let chunk_iend;
	let mut other_chunks: Vec<chunk::Chunk> = vec![];

	loop {
		let (current_chunk, new_index) = chunk::Chunk::read_from_vec(&bytes_vec, index)?;
		index = new_index;
		match &current_chunk.chunk_type {
			b"IHDR" => chunk_ihdr = current_chunk.clone(),
			b"zTXt" => chunk_ztxt = current_chunk.clone(),
			b"IDAT" => chunk_idat = current_chunk.clone(),
			b"IEND" => {
				chunk_iend = current_chunk.clone();
				break;
			}
			_ => other_chunks.push(current_chunk.clone()),
		}
	}

	let dmi_result = Dmi {
		header,
		chunk_ihdr,
		chunk_ztxt,
		chunk_idat,
		chunk_iend,
		other_chunks,
	};
	Ok(dmi_result)
}
