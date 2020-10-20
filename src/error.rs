#[derive(Debug)]
pub enum ReadError {
	/** An IO error */
	Io(::std::io::Error),
	/** The chunk_type is invalid (a byte was outside the range `A-Za-z`) */
	InvalidChunkType([u8; 4]),
	/** The calculated CRC did not match the given CRC */
	CrcMismatch {
		stated: u32,
		calculated: u32,
	},
	/** The given magic header didn't match the expected PNG header */
	MagicMismatch([u8; 8]),
	/** The given magic header didn't match the expected PNG header */
	Image(image::error::ImageError),
	Generic(String),
	Encoding(String),
}

impl ::std::fmt::Display for ReadError {
	fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
		match self {
			ReadError::Io(x) => write!(f, "{}", x),
			ReadError::InvalidChunkType(x) => write!(f, "Illegal chunk type: {:?}", x),
			ReadError::CrcMismatch { stated, calculated } => write!(
				f,
				"CRC mismatch: file {:x}; calculated {:x}",
				stated, calculated
			),
			ReadError::MagicMismatch(x) => {
				let bytes = x;
				let chars: String = x
					.iter()
					.map(|x| char::from(*x))
					.map(|x| if x.is_ascii_graphic() { x } else { '.' })
					.collect();
				write!(
					f,
					"PNG magic header didn't match expected value: {:?} | {:?}",
					bytes, chars
				)
			}
			ReadError::Image(x) => write!(f, "{:?}", x),
			ReadError::Generic(x) => write!(f, "{}", x),
			ReadError::Encoding(x) => write!(f, "{}", x),
		}
	}
}
