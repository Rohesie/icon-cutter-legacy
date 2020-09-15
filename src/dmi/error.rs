/// Represents an error that can occur when processing a dmi
#[derive(Debug)]
pub enum DmiError{
	/** An IO error */
	Io(::std::io::Error),
	/** The chunk's chunk_type is invalid (a byte was outside the range `A-Za-z`) */
	InvalidChunkType([u8;4]),
	/** The calculated CRC did not match the given CRC */
	CrcMismatch{stated:u32, calculated:u32},
	/** The given magic header didn't match the expected PNG header */
	MagicMismatch([u8;8]),
}
