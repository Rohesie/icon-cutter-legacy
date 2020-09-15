use super::glob;

pub fn smooth_dir_to_combination_key(smooth_dirs:u8) -> u8 {
	let mut combination_key = glob::NONE;
	for dir in glob::ADJ_CARDINALS.iter() {
		if smooth_dirs & dir != glob::NONE {
			combination_key |= dir;
		}
	}
	if smooth_dirs & glob::ADJ_N != glob::NONE {
		if smooth_dirs & glob::ADJ_E != glob::NONE && smooth_dirs & glob::ADJ_NE != glob::NONE {
			combination_key |= glob::ADJ_NE;
		}
		if smooth_dirs & glob::ADJ_W != glob::NONE && smooth_dirs & glob::ADJ_NW != glob::NONE {
			combination_key |= glob::ADJ_NW;
		}
	}
	if smooth_dirs & glob::ADJ_S != glob::NONE {
		if smooth_dirs & glob::ADJ_E != glob::NONE && smooth_dirs & glob::ADJ_SE != glob::NONE {
			combination_key |= glob::ADJ_SE;
		}
		if smooth_dirs & glob::ADJ_W != glob::NONE && smooth_dirs & glob::ADJ_SW != glob::NONE {
			combination_key |= glob::ADJ_SW;
		}
	}
	return combination_key;
}

pub fn smooth_dir_to_corner_type(corner:u8, smooth_dirs:u8) -> u8 {
	match corner {
		glob::NE_INDEX => {
			if smooth_dirs & glob::ADJ_N == glob::NONE {
				if smooth_dirs & glob::ADJ_E == glob::NONE {
					return glob::CONVEX
				}
				return glob::HORIZONTAL
			}
			if smooth_dirs & glob::ADJ_E == glob::NONE {
				return glob::VERTICAL
			}
			if smooth_dirs & glob::ADJ_NE == glob::NONE {
				return glob::CONCAVE
			}
			glob::FLAT
		},
		glob::SE_INDEX => {
			if smooth_dirs & glob::ADJ_S == glob::NONE {
				if smooth_dirs & glob::ADJ_E == glob::NONE  {
					return glob::CONVEX
				}
				return glob::HORIZONTAL
			}
			if smooth_dirs & glob::ADJ_E == glob::NONE {
				return glob::VERTICAL
			}
			if smooth_dirs & glob::ADJ_SE == glob::NONE  {
				return glob::CONCAVE
			}
			glob::FLAT
		},
		glob::SW_INDEX => {
			if smooth_dirs & glob::ADJ_S == glob::NONE {
				if smooth_dirs & glob::ADJ_W == glob::NONE {
					return glob::CONVEX
				}
				return glob::HORIZONTAL
			}
			if smooth_dirs & glob::ADJ_W == glob::NONE {
				return glob::VERTICAL
			}
			if smooth_dirs & glob::ADJ_SW == glob::NONE {
				return glob::CONCAVE
			}
			glob::FLAT
		},
		glob::NW_INDEX => {
			if smooth_dirs & glob::ADJ_N == glob::NONE {
				if smooth_dirs & glob::ADJ_W == glob::NONE {
					return glob::CONVEX
				}
				return glob::HORIZONTAL
			}
			if smooth_dirs & glob::ADJ_W == glob::NONE {
				return glob::VERTICAL
			}
			if smooth_dirs & glob::ADJ_NW == glob::NONE {
				return glob::CONCAVE
			}
			glob::FLAT
		},
		_ => panic!("smooth_dir_to_corner_type called with {}", corner)
	}
}

pub fn corner_to_string(corner_dir: u8, corner_type: u8) -> String {
	let dir_str = match corner_dir {
		glob::NE_INDEX => "ne",
		glob::SE_INDEX => "se",
		glob::SW_INDEX => "sw",
		glob::NW_INDEX => "nw",
		_ => panic!("corner_to_string called with corner_dir {}", corner_dir),
	};
	let type_str = match corner_type {
		glob::CONVEX => "conv",
		glob::CONCAVE => "conc",
		glob::HORIZONTAL => "hori",
		glob::VERTICAL => "vert",
		glob::FLAT => "flat",
		_ => panic!("corner_to_string called with corner_type {}", corner_type),
	};
	format!("{}-{}", dir_str, type_str)
}
