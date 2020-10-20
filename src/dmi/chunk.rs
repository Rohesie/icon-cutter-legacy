use super::super::error::ReadError;
use super::crc;
use deflate;

#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct Chunk {
	pub length: [u8; 4],
	pub chunk_type: [u8; 4],
	pub data: Vec<u8>,
	pub crc: [u8; 4],
}

impl Chunk {
	/// Reads a PNG chunk from a vector buffer.
	pub fn read_from_vec(bytes_vec: &[u8], index: usize) -> Result<(Chunk, usize), ReadError> {
		let chunk_length_array = [
			bytes_vec[index],
			bytes_vec[index + 1],
			bytes_vec[index + 2],
			bytes_vec[index + 3],
		];
		let chunk_length = u32::from_be_bytes(chunk_length_array) as usize;
		let mut new_index = index + 4;
		let chunk_type: [u8; 4] = [
			bytes_vec[new_index],
			bytes_vec[new_index + 1],
			bytes_vec[new_index + 2],
			bytes_vec[new_index + 3],
		];
		if !chunk_type
			.iter()
			.all(|c| (0x41 <= *c && *c <= 0x5A) || (0x61 <= *c && *c <= 0x7A))
		{
			return Err(ReadError::InvalidChunkType(chunk_type));
		}
		new_index += 4;
		let chunk_data: Vec<u8> = bytes_vec[new_index..(new_index + chunk_length)]
			.iter()
			.cloned()
			.collect();
		new_index += chunk_length;
		let crc_array = [
			bytes_vec[new_index],
			bytes_vec[new_index + 1],
			bytes_vec[new_index + 2],
			bytes_vec[new_index + 3],
		];
		let stated_crc = u32::from_be_bytes(crc_array);
		let cacluated_crc = crc::calculate_crc(chunk_type.iter().chain(chunk_data.iter()));
		if stated_crc != cacluated_crc {
			return Err(ReadError::CrcMismatch {
				stated: stated_crc,
				calculated: cacluated_crc,
			});
		};
		new_index += 4;
		Ok((
			Chunk {
				length: chunk_length_array,
				chunk_type,
				data: chunk_data,
				crc: crc_array,
			},
			new_index,
		))
	}

	/// Writes a PNG chunk to a vector buffer.
	pub fn write_to_vec(&mut self, mut bytes_vec: Vec<u8>) -> Result<Vec<u8>, ReadError> {
		bytes_vec.extend_from_slice(&self.length);
		bytes_vec.extend_from_slice(&self.chunk_type);
		bytes_vec.extend_from_slice(&self.data);
		bytes_vec.extend_from_slice(&self.crc);
		Ok(bytes_vec)
	}

	pub fn write_ztxt_chunk(&mut self, new_text: String) -> Result<bool, ReadError> {
		let mut data: Vec<u8> = vec![];

		let keyword = "Description".as_bytes();
		data.extend_from_slice(keyword);

		let compression_method: [u8; 2] = [0, 0];
		data.extend_from_slice(&compression_method);

		let dmi_data = new_text;
		let dmi_data = deflate::deflate_bytes_zlib(dmi_data.as_bytes());
		data.extend(dmi_data);

		let chunk_type = b"zTXt";

		let crc = crc::calculate_crc(chunk_type.iter().chain(data.iter())).to_be_bytes();

		self.length = (data.len() as u32).to_be_bytes();
		self.chunk_type = *chunk_type;
		self.data = data;
		self.crc = crc;
		Ok(true)
	}
}
