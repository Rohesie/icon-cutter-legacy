use image::imageops;
use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use yaml_rust::YamlLoader;

use super::error;
use super::glob;

#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct PrefHolder {
	pub file_to_open: Option<String>,
	pub output_name: Option<String>,
	pub base_icon_state: Option<String>,

	pub icon_size_x: u32,
	pub icon_size_y: u32,

	pub center_x: u32,
	pub center_y: u32,

	pub west_start: u32,
	pub west_step: u32,
	pub east_start: u32,
	pub east_step: u32,
	pub north_start: u32,
	pub north_step: u32,
	pub south_start: u32,
	pub south_step: u32,

	pub frames_per_state: u32,
	pub delay: Option<Vec<u32>>,

	pub produce_corners: bool,

	pub prefabs: Option<HashMap<u8, u32>>,
	pub prefab_overlays: Option<HashMap<u8, Vec<u32>>>,

	pub dmi_version: String,

	pub se_convex: u32,
	pub nw_convex: u32,
	pub ne_convex: u32,
	pub sw_convex: u32,

	pub se_concave: u32,
	pub nw_concave: u32,
	pub ne_concave: u32,
	pub sw_concave: u32,

	pub se_horizontal: u32,
	pub nw_horizontal: u32,
	pub ne_horizontal: u32,
	pub sw_horizontal: u32,

	pub se_vertical: u32,
	pub nw_vertical: u32,
	pub ne_vertical: u32,
	pub sw_vertical: u32,

	pub se_flat: Option<u32>,
	pub nw_flat: Option<u32>,
	pub ne_flat: Option<u32>,
	pub sw_flat: Option<u32>,

	pub is_diagonal: bool,
}

impl PrefHolder {
	pub fn build_corners_and_prefabs(
		&self,
		input: std::io::Cursor<Vec<u8>>,
		file_name: &str,
	) -> Result<
		(
			HashMap<u8, HashMap<u8, Vec<image::DynamicImage>>>,
			HashMap<u8, Vec<image::DynamicImage>>,
		),
		error::ReadError,
	> {
		let img =
			image::load(input, image::ImageFormat::Png).map_err(|x| error::ReadError::Image(x))?;

		let img_dimensions = match &img {
			image::DynamicImage::ImageLuma8(inner_img) => inner_img.dimensions(),
			image::DynamicImage::ImageLumaA8(inner_img) => inner_img.dimensions(),
			image::DynamicImage::ImageRgb8(inner_img) => inner_img.dimensions(),
			image::DynamicImage::ImageRgba8(inner_img) => inner_img.dimensions(),
			image::DynamicImage::ImageBgr8(inner_img) => inner_img.dimensions(),
			image::DynamicImage::ImageBgra8(inner_img) => inner_img.dimensions(),
			image::DynamicImage::ImageLuma16(inner_img) => inner_img.dimensions(),
			image::DynamicImage::ImageLumaA16(inner_img) => inner_img.dimensions(),
			image::DynamicImage::ImageRgb16(inner_img) => inner_img.dimensions(),
			image::DynamicImage::ImageRgba16(inner_img) => inner_img.dimensions(),
		};

		let width_in_frames = img_dimensions.0 / self.icon_size_x;
		let height_in_frames = img_dimensions.1 / self.icon_size_y;

		let max_total_frames = width_in_frames * height_in_frames;

		// Index defined by glob::CORNER_DIRS
		// corners -> corner_dir -> corner_type -> frames

		let corner_types: &[u8];
		let corners_length;
		if self.is_diagonal {
			corner_types = &glob::CORNER_TYPES_DIAGONAL;
			corners_length = glob::CORNER_TYPES_DIAGONAL.len() as u32;
		} else {
			corner_types = &glob::CORNER_TYPES_CARDINAL;
			corners_length = glob::CORNER_TYPES_CARDINAL.len() as u32;
		};

		let mut corners: HashMap<u8, HashMap<u8, Vec<image::DynamicImage>>> = HashMap::new();
		for corner_dir in glob::CORNER_DIRS.iter() {
			corners.insert(*corner_dir, HashMap::new());
			for corner_type in corner_types.iter() {
				let dir_map = corners.get_mut(corner_dir).unwrap();
				dir_map.insert(*corner_type, vec![]);
				for frame_offset in 0..self.frames_per_state {
					let frame_vec = dir_map.get_mut(corner_type).unwrap();
					let corner_params = self.get_corner_params(
						*corner_dir,
						*corner_type,
						frame_offset,
						width_in_frames,
						max_total_frames,
					)?;
					let corner_img = img.crop_imm(
						corner_params.0,
						corner_params.1,
						corner_params.2,
						corner_params.3,
					);
					frame_vec.push(corner_img);
				}
			}
		}

		if self.produce_corners {
			let mut corners_image = image::DynamicImage::new_rgba8(
				corners_length * self.icon_size_x,
				1 * self.icon_size_y,
			);
			let mut index = 0;
			for corner_type in corner_types.iter() {
				for frame in 0..self.frames_per_state {
					let frame_img = &corners
						.get_mut(&glob::NW_INDEX)
						.unwrap()
						.get_mut(corner_type)
						.unwrap()[frame as usize];
					imageops::replace(
						&mut corners_image,
						frame_img,
						(index * self.icon_size_x) + self.west_start,
						(0 * self.icon_size_y) + self.north_start,
					);
					let frame_img = &corners
						.get_mut(&glob::NE_INDEX)
						.unwrap()
						.get_mut(corner_type)
						.unwrap()[frame as usize];
					imageops::replace(
						&mut corners_image,
						frame_img,
						(index * self.icon_size_x) + self.east_start,
						(0 * self.icon_size_y) + self.north_start,
					);
					let frame_img = &corners
						.get_mut(&glob::SE_INDEX)
						.unwrap()
						.get_mut(corner_type)
						.unwrap()[frame as usize];
					imageops::replace(
						&mut corners_image,
						frame_img,
						(index * self.icon_size_x) + self.east_start,
						(0 * self.icon_size_y) + self.south_start,
					);
					let frame_img = &corners
						.get_mut(&glob::SW_INDEX)
						.unwrap()
						.get_mut(corner_type)
						.unwrap()[frame as usize];
					imageops::replace(
						&mut corners_image,
						frame_img,
						(index * self.icon_size_x) + self.west_start,
						(0 * self.icon_size_y) + self.south_start,
					);
					index += 1;
				}
			}
			let output_name = match &self.output_name {
				Some(thing) => format!("{}", thing),
				None => format!("{}-output", file_name),
			};
			corners_image
				.save(format!("{}-corners.png", output_name))
				.unwrap();
		};

		let mut prefabs: HashMap<u8, Vec<image::DynamicImage>> = HashMap::new();
		match &self.prefabs {
			Some(thing) => {
				for (signature, location) in thing {
					let mut frame_vector = vec![];
					for frame in 0..self.frames_per_state {
						let prefab_img = img.crop_imm(
							self.icon_size_x
								* self.icon_positition_to_x_coordinate(
									&format!("prefab {} icon_size_x", signature),
									*location,
									frame,
									width_in_frames,
									max_total_frames,
								)?,
							self.icon_size_y
								* self.icon_positition_to_y_coordinate(
									&format!("prefab {} icon_size_y", signature),
									*location,
									frame,
									width_in_frames,
									max_total_frames,
								)?,
							self.icon_size_x,
							self.icon_size_y,
						);
						frame_vector.push(prefab_img);
					}
					prefabs.insert(*signature, frame_vector); // End result: prefabs -> junction signature -> frame vector -> image
				}
			}
			None => (),
		};
		match &self.prefab_overlays {
			Some(thing) => {
				for (signature, location_vec) in thing {
					match prefabs.remove(signature) {
						Some(frame_vector) => {
							if frame_vector.len() as u32 != self.frames_per_state {
								return Err(error::ReadError::Generic(format!("Number of prefab overlays for signature {} does not match the frames per state ({}): {}. Aborting to avoid undefined behavior.", signature, self.frames_per_state, frame_vector.len())))
							}; // Sanity check, this should never happen unless the logic above was changed.
							let mut unoverlaid_vector = frame_vector;
							let mut overlaid_vector = vec![];
							for frame in 0..self.frames_per_state {
								let prefab_image = unoverlaid_vector.remove(0);
								let mut overlaid_prefab = prefab_image;
								let corner_img = img.crop_imm(
									self.icon_size_x * self.icon_positition_to_x_coordinate("prefab_overlays", location_vec[frame as usize], frame, width_in_frames, max_total_frames)?,
									self.icon_size_y * self.icon_positition_to_y_coordinate("prefab_overlays", location_vec[frame as usize], frame, width_in_frames, max_total_frames)?,
									self.icon_size_x,
									self.icon_size_y,
									);
								imageops::overlay(&mut overlaid_prefab, &corner_img, 0, 0);
								overlaid_vector.push(overlaid_prefab);
								};
							prefabs.insert(*signature, overlaid_vector);
						},
						None => return Err(error::ReadError::Generic(format!("Prefab overlay defined for inexistent prefab. Signature: {}. Overlays: {:?}.", signature, location_vec)))
					};
				}
			}
			None => (),
		};
		Ok((corners, prefabs))
	}

	pub fn get_corner_params(
		&self,
		corner_dir: u8,
		corner_type: u8,
		frame_offset: u32,
		width_in_frames: u32,
		max_total_frames: u32,
	) -> Result<(u32, u32, u32, u32), error::ReadError> {
		let corner_parameters = match corner_dir {
			glob::NE_INDEX => {
				let pos_name_and_value = match corner_type {
					glob::CONVEX => ("ne_convex", self.ne_convex),
					glob::CONCAVE => ("ne_concave", self.ne_concave),
					glob::HORIZONTAL => ("ne_horizontal", self.ne_horizontal),
					glob::VERTICAL => ("ne_vertical", self.ne_vertical),
					glob::FLAT => {
						let ne_flat = match self.ne_flat {
							Some(value) => value,
							None => {
								return Err(error::ReadError::Generic(format!(
									"get_corner_params -> NE_INDEX -> glob::FLAT -> {:?}",
									self.ne_flat
								)))
							}
						};
						("ne_flat", ne_flat)
					}
					_ => {
						return Err(error::ReadError::Generic(format!(
							"get_corner_params -> NE_INDEX -> {}",
							corner_type
						)))
					}
				};
				(
					pos_name_and_value.0,
					pos_name_and_value.1,
					self.east_start,
					self.north_start,
					self.east_step,
					self.north_step,
				)
			}
			glob::SE_INDEX => {
				let pos_name_and_value = match corner_type {
					glob::CONVEX => ("se_convex", self.se_convex),
					glob::CONCAVE => ("se_concave", self.se_concave),
					glob::HORIZONTAL => ("se_horizontal", self.se_horizontal),
					glob::VERTICAL => ("se_vertical", self.se_vertical),
					glob::FLAT => {
						let se_flat = match self.se_flat {
							Some(value) => value,
							None => {
								return Err(error::ReadError::Generic(format!(
									"get_corner_params -> SE_INDEX -> glob::FLAT -> {:?}",
									self.se_flat
								)))
							}
						};
						("se_flat", se_flat)
					}
					_ => {
						return Err(error::ReadError::Generic(format!(
							"get_corner_params -> SE_INDEX -> {}",
							corner_type
						)))
					}
				};
				(
					pos_name_and_value.0,
					pos_name_and_value.1,
					self.east_start,
					self.south_start,
					self.east_step,
					self.south_step,
				)
			}
			glob::SW_INDEX => {
				let pos_name_and_value = match corner_type {
					glob::CONVEX => ("sw_convex", self.sw_convex),
					glob::CONCAVE => ("sw_concave", self.sw_concave),
					glob::HORIZONTAL => ("sw_horizontal", self.sw_horizontal),
					glob::VERTICAL => ("sw_vertical", self.sw_vertical),
					glob::FLAT => {
						let sw_flat = match self.sw_flat {
							Some(value) => value,
							None => {
								return Err(error::ReadError::Generic(format!(
									"get_corner_params -> SW_INDEX -> glob::FLAT -> {:?}",
									self.sw_flat
								)))
							}
						};
						("sw_flat", sw_flat)
					}
					_ => {
						return Err(error::ReadError::Generic(format!(
							"get_corner_params -> SW_INDEX -> {}",
							corner_type
						)))
					}
				};
				(
					pos_name_and_value.0,
					pos_name_and_value.1,
					self.west_start,
					self.south_start,
					self.west_step,
					self.south_step,
				)
			}
			glob::NW_INDEX => {
				let pos_name_and_value = match corner_type {
					glob::CONVEX => ("nw_convex", self.nw_convex),
					glob::CONCAVE => ("nw_concave", self.nw_concave),
					glob::HORIZONTAL => ("nw_horizontal", self.nw_horizontal),
					glob::VERTICAL => ("nw_vertical", self.nw_vertical),
					glob::FLAT => {
						let nw_flat = match self.nw_flat {
							Some(value) => value,
							None => {
								return Err(error::ReadError::Generic(format!(
									"get_corner_params -> NW_INDEX -> glob::FLAT -> {:?}",
									self.nw_flat
								)))
							}
						};
						("nw_flat", nw_flat)
					}
					_ => {
						return Err(error::ReadError::Generic(format!(
							"get_corner_params -> NW_INDEX -> {}",
							corner_type
						)))
					}
				};
				(
					pos_name_and_value.0,
					pos_name_and_value.1,
					self.west_start,
					self.north_start,
					self.west_step,
					self.north_step,
				)
			}
			_ => {
				return Err(error::ReadError::Generic(format!(
					"get_corner_params -> {}",
					corner_dir
				)))
			}
		};
		let x_coordinate = self.icon_positition_to_x_coordinate(
			corner_parameters.0,
			corner_parameters.1,
			frame_offset,
			width_in_frames,
			max_total_frames,
		)?;
		let y_coordinate = self.icon_positition_to_y_coordinate(
			corner_parameters.0,
			corner_parameters.1,
			frame_offset,
			width_in_frames,
			max_total_frames,
		)?;
		Ok((
			self.icon_size_x * x_coordinate + corner_parameters.2,
			self.icon_size_x * y_coordinate + corner_parameters.3,
			corner_parameters.4,
			corner_parameters.5,
		))
	}

	pub fn icon_positition_to_x_coordinate(
		&self,
		var_name: &str,
		position: u32,
		frame_offset: u32,
		width_in_frames: u32,
		max_total_frames: u32,
	) -> Result<u32, error::ReadError> {
		let icon_position = position * self.frames_per_state + frame_offset;
		if icon_position > max_total_frames {
			return Err(error::ReadError::Generic(format!("Unlawful value for {} ({}), larger than the maximum amount of frames this image holds ({})", var_name, position, max_total_frames)));
		};
		Ok(icon_position % width_in_frames)
	}

	pub fn icon_positition_to_y_coordinate(
		&self,
		var_name: &str,
		position: u32,
		frame_offset: u32,
		width_in_frames: u32,
		max_total_frames: u32,
	) -> Result<u32, error::ReadError> {
		let icon_position = position * self.frames_per_state + frame_offset;
		if icon_position > max_total_frames {
			return Err(error::ReadError::Generic(format!("Unlawful value for {} ({}), larger than the maximum amount of frames this image holds ({})", var_name, position, max_total_frames)));
		};
		Ok(icon_position / width_in_frames) // This operation rounds towards zero, truncating any fractional part of the exact result, essentially a floor() function.
	}
}

pub fn read_some_u32_config(source: &yaml_rust::yaml::Yaml, index: &str) -> Option<u32> {
	let config = &source[index];
	if config.is_badvalue() {
		return None;
	}
	return match source[index].as_i64() {
		Some(thing) => Some(thing as u32),
		None => None,
	};
}

pub fn read_necessary_u32_config(
	source: &yaml_rust::yaml::Yaml,
	index: &str,
) -> Result<u32, error::ReadError> {
	let config = &source[index];
	if config.is_badvalue() {
		return Err(error::ReadError::Generic(format!("Undefined value for {}. This is a necessary config. Please check config.yaml in the examples folder for documentation.", index)));
	};
	return match source[index].as_i64() {
		Some(thing) => Ok(thing as u32),
		None => Err(error::ReadError::Generic(format!(
			"Unlawful value for {}, not a proper number: ({:?})",
			index, source[index]
		))),
	};
}

pub fn read_some_string_config(source: &yaml_rust::yaml::Yaml, index: &str) -> Option<String> {
	let config = &source[index];
	if config.is_badvalue() {
		return None;
	};
	return match source[index].as_str() {
		Some(thing) => Some(thing.to_string()),
		None => None,
	};
}

pub fn load_configs(caller_path: String) -> Result<PrefHolder, error::ReadError> {
	let config_path;
	let last_slash = caller_path.rfind(|c| c == '/' || c == '\\');
	if last_slash != None {
		config_path = caller_path[..last_slash.unwrap()].to_string();
	} else {
		config_path = ".".to_string();
	};
	let path = Path::new(&config_path).join("config.yaml");
	let mut file = File::open(path).map_err(|x| error::ReadError::Io(x))?;
	let mut contents = String::new();
	file.read_to_string(&mut contents)
		.map_err(|x| error::ReadError::Io(x))?;
	let docs = YamlLoader::load_from_str(&contents).unwrap();
	let doc = &docs[0];

	let file_to_open = read_some_string_config(&doc, "file_to_open");
	let output_name = read_some_string_config(&doc, "output_name");
	let base_icon_state = read_some_string_config(&doc, "base_icon_state");

	let icon_size_x = match read_some_u32_config(&doc, "icon_size_x") {
		Some(thing) => {
			if thing <= 0 {
				return Err(error::ReadError::Generic(format!(
					"Unlawful value for icon_size_x: {}",
					thing
				)));
			} else {
				thing
			}
		}
		None => 32,
	};
	let center_x = match read_some_u32_config(&doc, "center_x") {
		Some(thing) => {
			if thing > icon_size_x {
				return Err(error::ReadError::Generic(format!(
					"Unlawful value for center_x ({}), larger than icon_size_x ({})",
					thing, icon_size_x
				)));
			} else {
				thing
			}
		}
		None => 16,
	};
	let west_start = match read_some_u32_config(&doc, "west_start") {
		Some(thing) => {
			if thing > center_x {
				return Err(error::ReadError::Generic(format!(
					"Unlawful value for west_start ({}), larger than center_x ({})",
					thing, center_x
				)));
			} else {
				thing
			}
		}
		None => glob::ORIGIN_X,
	};
	let west_end = match read_some_u32_config(&doc, "west_end") {
		Some(thing) => {
			if thing > center_x || thing < west_start {
				return Err(error::ReadError::Generic(format!(
					"Unlawful value for west_end ({}), cannot be larger than center_x ({}) nor smaller than west_start ({})",
					thing, center_x, west_start
				)));
			} else {
				thing
			}
		}
		None => center_x,
	};
	let west_step =  west_end - west_start;
	let east_start = match read_some_u32_config(&doc, "east_start") {
		Some(thing) => {
			if thing > icon_size_x || thing < center_x {
				return Err(error::ReadError::Generic(format!(
					"Unlawful value for east_start ({}), cannot be larger than icon_size_x ({}) nor smaller than center_x ({})",
					thing, icon_size_x, center_x
				)));
			} else {
				thing
			}
		}
		None => center_x,
	};
	let east_end = match read_some_u32_config(&doc, "east_end") {
		Some(thing) => {
			if thing > icon_size_x || thing < east_start {
				return Err(error::ReadError::Generic(format!(
					"Unlawful value for east_end ({}), cannot be larger than icon_size_x ({}) nor smaller than east_start ({})",
					thing, icon_size_x, east_start
				)));
			} else {
				thing
			}
		}
		None => icon_size_x,
	};
	let east_step = east_end - east_start;

	let icon_size_y = match read_some_u32_config(&doc, "icon_size_y") {
		Some(thing) => {
			if thing <= 0 {
				return Err(error::ReadError::Generic(format!(
					"Unlawful value for icon_size_y: {}",
					thing
				)));
			} else {
				thing
			}
		}
		None => 32,
	};
	let center_y = match read_some_u32_config(&doc, "center_y") {
		Some(thing) => {
			if thing > icon_size_y {
				return Err(error::ReadError::Generic(format!(
					"Unlawful value for center_y ({}), larger than icon_size_y ({})",
					thing, icon_size_y
				)));
			} else {
				thing
			}
		}
		None => 16,
	};
	let north_start = match read_some_u32_config(&doc, "north_start") {
		Some(thing) => {
			if thing > center_y {
				return Err(error::ReadError::Generic(format!(
					"Unlawful value for north_start ({}), larger than center_y ({})",
					thing, center_y
				)));
			} else {
				thing
			}
		}
		None => glob::ORIGIN_Y,
	};
	let north_end = match read_some_u32_config(&doc, "north_end") {
		Some(thing) => {
			if thing > center_y || thing < north_start {
				return Err(error::ReadError::Generic(format!(
					"Unlawful value for north_end ({}), cannot be larger than center_y ({}) nor smaller than north_start ({})",
					thing, center_y, north_start
				)));
			} else {
				thing
			}
		}
		None => center_y,
	};
	let north_step =  north_end - north_start;
	let south_start = match read_some_u32_config(&doc, "south_start") {
		Some(thing) => {
			if thing > icon_size_y || thing < center_y {
				return Err(error::ReadError::Generic(format!(
					"Unlawful value for south_start ({}), cannot be larger than icon_size_y ({}) nor smaller than center_y ({})",
					thing, icon_size_y, center_y
				)));
			} else {
				thing
			}
		}
		None => center_y,
	};
	let south_end = match read_some_u32_config(&doc, "south_end") {
		Some(thing) => {
			if thing > icon_size_y || thing < south_start {
				return Err(error::ReadError::Generic(format!(
					"Unlawful value for south_end ({}), cannot be larger than icon_size_y ({}) nor smaller than south_start ({})",
					thing, icon_size_y, south_start
				)));
			} else {
				thing
			}
		}
		None => icon_size_y,
	};
	let south_step = south_end - south_start;

	let frames_per_state = match read_some_u32_config(&doc, "frames_per_state") {
		Some(thing) => {
			if thing <= 0 {
				return Err(error::ReadError::Generic(format!(
					"Unlawful value for frames_per_state: {}",
					thing
				)));
			} else {
				thing
			}
		}
		None => 1,
	};

	let delay;
	if frames_per_state == 1 {
		delay = None;
	} else {
		let mut delay_vec = vec![];
		if doc["delay"].is_badvalue() {
			for _frame in 0..frames_per_state {
				delay_vec.push(1) // List is empty, let's fill it with an arbitrary value.
			}
		} else {
			let yaml_delay;
			match doc["delay"].as_vec() {
					Some(thing) => yaml_delay = thing,
					None => return Err(error::ReadError::Generic(format!("Delay config improperly set. Please look at the example files for the proper format. Contents: {:?}", doc["delay"])))
				};
			for delay_value in yaml_delay.iter() {
				delay_vec.push(delay_value.as_i64().unwrap() as u32);
			}
			if delay_vec.len() as u32 > frames_per_state {
				return Err(error::ReadError::Generic(format!(
			"Higher number of entries in the delay input ({}) than the frames_per_state value ({}). delay entries: {:?}",
			delay_vec.len(), frames_per_state, delay_vec
		)));
			} else if (delay_vec.len() as u32) < frames_per_state {
				// Too few entries defined, we'll have to get creative and fill in the blanks.
				if delay_vec.len() == 0 {
					for _frame in 0..frames_per_state {
						delay_vec.push(1) // List is empty, let's fill it with an arbitrary value.
					}
				} else {
					let mut index = 0;
					for _frame in (delay_vec.len() as u32)..frames_per_state {
						delay_vec.push(delay_vec[index]); // We fill the list repeating the given pattern.
						index += 1;
					}
				};
			};
		};
		delay = Some(delay_vec);
	};

	let produce_corners;
	if doc["produce_corners"].is_badvalue() {
		produce_corners = false;
	} else {
		produce_corners = match doc["produce_corners"].as_bool() {
			Some(thing) => thing,
			None => false,
		};
	};

	let prefabs;
	if doc["prefabs"].is_badvalue() {
		prefabs = None;
	} else {
		let mut prefab_map: HashMap<u8, u32> = HashMap::new();
		let yaml_prefabs = match doc["prefabs"].as_hash() {
			Some(thing) => thing,
			None => {
				return Err(error::ReadError::Generic(format!(
					"prefabs value improperly setup: {:?}",
					doc["prefabs"]
				)))
			}
		};
		for (prefab_signature, position) in yaml_prefabs.iter() {
			let signature = match prefab_signature.as_i64() {
				Some(thing) => thing as u8,
				None => {
					return Err(error::ReadError::Generic(format!(
						"prefab signature value improperly: {:?}",
						prefab_signature
					)))
				}
			};
			let position = match position.as_i64() {
				Some(thing) => thing as u32,
				None => {
					return Err(error::ReadError::Generic(format!(
						"prefab value improperly set for {}: {:?}",
						signature, position
					)))
				}
			};
			prefab_map.insert(signature, position);
		}
		prefabs = Some(prefab_map);
	}

	let prefab_overlays;
	if doc["prefab_overlays"].is_badvalue() {
		prefab_overlays = None;
	} else {
		let mut overlays_map: HashMap<u8, Vec<u32>> = HashMap::new();
		let yaml_prefab_overlays =  match doc["prefab_overlays"].as_hash() {
			Some(thing) => thing,
			None => return Err(error::ReadError::Generic(format!("prefab_overlays defined with the wrong format. See the config.yaml in the example folder for a valid one. Read value: {:?}", doc["prefab_overlays"])))
		};
		for (overlay_signature, coords_list) in yaml_prefab_overlays.iter() {
			let signature = match overlay_signature.as_i64() {
				Some(thing) => thing as u8,
				None => return Err(error::ReadError::Generic(format!("prefab_overlays signature defined with the wrong format. See the config.yaml in the example folder for a valid one. Read value: {:?}", overlay_signature)))
			};
			let coords_list = match coords_list.as_vec() {
				Some(thing) => thing,
				None => {
					return Err(error::ReadError::Generic(format!(
						"prefab_overlays values for {} signature improperly set: {:?}",
						signature, coords_list
					)))
				}
			};
			let mut overlay_vec = vec![];
			for value in coords_list.iter() {
				let value = match value.as_i64() {
					Some(thing) => thing as u32,
					None => {
						return Err(error::ReadError::Generic(format!(
							"Improper value found in prefab_overlays for signature {}: {:?}",
							signature, value
						)))
					}
				};
				overlay_vec.push(value)
			}
			if overlay_vec.len() == 0 {
				return Err(error::ReadError::Generic(format!(
					"prefab_overlays values for {} empty, this is likely not intended.",
					signature
				)));
			};
			overlays_map.insert(signature, overlay_vec);
		}
		prefab_overlays = Some(overlays_map);
	};

	let dmi_version = match read_some_string_config(&doc, "file_to_open") {
		Some(thing) => thing.to_string(),
		None => "4.0".to_string(),
	};

	let se_convex = read_necessary_u32_config(&doc, "se_convex")?;
	let nw_convex = read_necessary_u32_config(&doc, "nw_convex")?;
	let ne_convex = read_necessary_u32_config(&doc, "ne_convex")?;
	let sw_convex = read_necessary_u32_config(&doc, "sw_convex")?;
	let se_concave = read_necessary_u32_config(&doc, "se_concave")?;
	let nw_concave = read_necessary_u32_config(&doc, "nw_concave")?;
	let ne_concave = read_necessary_u32_config(&doc, "ne_concave")?;
	let sw_concave = read_necessary_u32_config(&doc, "sw_concave")?;
	let se_horizontal = read_necessary_u32_config(&doc, "se_horizontal")?;
	let nw_horizontal = read_necessary_u32_config(&doc, "nw_horizontal")?;
	let ne_horizontal = read_necessary_u32_config(&doc, "ne_horizontal")?;
	let sw_horizontal = read_necessary_u32_config(&doc, "sw_horizontal")?;
	let se_vertical = read_necessary_u32_config(&doc, "se_vertical")?;
	let nw_vertical = read_necessary_u32_config(&doc, "nw_vertical")?;
	let ne_vertical = read_necessary_u32_config(&doc, "ne_vertical")?;
	let sw_vertical = read_necessary_u32_config(&doc, "sw_vertical")?;

	let se_flat = read_some_u32_config(&doc, "se_flat");
	let nw_flat = read_some_u32_config(&doc, "nw_flat");
	let ne_flat = read_some_u32_config(&doc, "ne_flat");
	let sw_flat = read_some_u32_config(&doc, "sw_flat");

	let is_diagonal = se_flat != None && nw_flat != None && ne_flat != None && sw_flat != None;

	return Ok(PrefHolder {
		file_to_open,
		output_name,
		base_icon_state,

		icon_size_x,
		center_x,
		west_start,
		west_step,
		east_start,
		east_step,
	
		icon_size_y,
		center_y,
		north_start,
		north_step,
		south_start,
		south_step,

		frames_per_state,
		delay,

		produce_corners,

		prefabs,
		prefab_overlays,

		dmi_version,

		se_convex,
		nw_convex,
		ne_convex,
		sw_convex,
		se_concave,
		nw_concave,
		ne_concave,
		sw_concave,
		se_horizontal,
		nw_horizontal,
		ne_horizontal,
		sw_horizontal,
		se_vertical,
		nw_vertical,
		ne_vertical,
		sw_vertical,

		se_flat,
		nw_flat,
		ne_flat,
		sw_flat,

		is_diagonal,
	});
}
