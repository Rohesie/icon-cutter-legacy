/// Represents an error that can occur when processing a dmi
#[derive(Debug)]
pub enum DmiReadError {
	/** An IO error */
	Io(::std::io::Error),
	/** The chunk_type is invalid (a byte was outside the range `A-Za-z`) */
	InvalidChunkType([u8; 4]),
	/** The calculated CRC did not match the given CRC */
	CrcMismatch { stated: u32, calculated: u32 },
	/** The given magic header didn't match the expected PNG header */
	MagicMismatch([u8; 8]),
	/** The given magic header didn't match the expected PNG header */
	Image(image::error::ImageError),
	Encoding(String),
}

impl ::std::fmt::Display for DmiReadError {
	fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
		match self {
			DmiReadError::Io(x) => write!(f, "{}", x),
			DmiReadError::InvalidChunkType(x) => write!(f, "Illegal chunk type: {:?}", x),
			DmiReadError::CrcMismatch { stated, calculated } => write!(
				f,
				"CRC mismatch: file {:x}; calculated {:x}",
				stated, calculated
			),
			DmiReadError::MagicMismatch(x) => {
				let bytes = x;
				let chars: String = x
					.iter()
					.map(|x| char::from(*x))
					.map(|x| if x.is_ascii_graphic() { x } else { '.' })
					.collect();
				write!(f, "PNG magic header didn't match expected value: {:?} | {:?}", bytes, chars)
			},
			DmiReadError::Image(x) => write!(f, "{:?}", x),
			DmiReadError::Encoding(x) => write!(f, "{}", x),
		}
	}
}
