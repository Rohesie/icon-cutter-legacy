use std::fs::File;
use std::io::prelude::*;
use yaml_rust::YamlLoader;
use std::collections::HashMap;
use image::imageops;

use super::glob;

pub struct PrefHolder {
	pub file_to_open: String,
	pub output_name: String,

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

	pub produce_corners: bool,

	pub pure_horizontal: bool,
	pub pure_vertical: bool,
	pub pure_flat: bool,

	pub horizontal_x: u32,
	pub horizontal_y: u32,

	pub vertical_x: u32,
	pub vertical_y: u32,

	pub flat_x: u32,
	pub flat_y: u32,

	pub se_convex_x: u32,
	pub se_convex_y: u32,
	pub nw_convex_x: u32,
	pub nw_convex_y: u32,
	pub ne_convex_x: u32,
	pub ne_convex_y: u32,
	pub sw_convex_x: u32,
	pub sw_convex_y: u32,

	pub se_concave_x: u32,
	pub se_concave_y: u32,
	pub nw_concave_x: u32,
	pub nw_concave_y: u32,
	pub ne_concave_x: u32,
	pub ne_concave_y: u32,
	pub sw_concave_x: u32,
	pub sw_concave_y: u32,

	pub se_horizontal_x: u32,
	pub se_horizontal_y: u32,
	pub nw_horizontal_x: u32,
	pub nw_horizontal_y: u32,
	pub ne_horizontal_x: u32,
	pub ne_horizontal_y: u32,
	pub sw_horizontal_x: u32,
	pub sw_horizontal_y: u32,

	pub se_vertical_x: u32,
	pub se_vertical_y: u32,
	pub nw_vertical_x: u32,
	pub nw_vertical_y: u32,
	pub ne_vertical_x: u32,
	pub ne_vertical_y: u32,
	pub sw_vertical_x: u32,
	pub sw_vertical_y: u32,

	pub se_flat_x: u32,
	pub se_flat_y: u32,
	pub nw_flat_x: u32,
	pub nw_flat_y: u32,
	pub ne_flat_x: u32,
	pub ne_flat_y: u32,
	pub sw_flat_x: u32,
	pub sw_flat_y: u32,
}

impl PrefHolder {
	pub fn build_corners_and_prefabs(&self) -> (Vec<Vec<image::DynamicImage>>, HashMap<u8, image::DynamicImage> /*ArrayVec<[image::DynamicImage; 3]>*/) {
		let mut img = image::open(&self.file_to_open).unwrap();

		//Index defined by glob::CORNER_DIRS
		let mut corners: Vec<Vec<image::DynamicImage>> = vec![vec![], vec![], vec![], vec![]];

		for corner_dir in glob::CORNER_DIRS.iter() {
			for corner_type in glob::CORNER_TYPES.iter() {
				let corner_params = self.get_corner_params(*corner_dir, *corner_type);
				let corner_img = img.crop(corner_params.0, corner_params.1, corner_params.2, corner_params.3);
				corners[*corner_dir as usize].push(corner_img);
			}
		}

		if self.produce_corners {
			let x_length = glob::CORNER_TYPES.len() as u32;
			let mut corners_image = image::DynamicImage::new_rgba8(x_length * glob::TILE_SIZE, 1 * glob::TILE_SIZE);
			let mut index = 0;
			for corner_type in glob::CORNER_TYPES.iter() {
				imageops::replace(&mut corners_image, &corners[glob::NW_INDEX as usize][*corner_type as usize], (index * glob::TILE_SIZE) + self.west_start, (0 * glob::TILE_SIZE) + self.north_start);
				imageops::replace(&mut corners_image, &corners[glob::NE_INDEX as usize][*corner_type as usize], (index * glob::TILE_SIZE) + self.east_start, (0 * glob::TILE_SIZE) + self.north_start);
				imageops::replace(&mut corners_image, &corners[glob::SE_INDEX as usize][*corner_type as usize], (index * glob::TILE_SIZE) + self.east_start, (0 * glob::TILE_SIZE) + self.south_start);
				imageops::replace(&mut corners_image, &corners[glob::SW_INDEX as usize][*corner_type as usize], (index * glob::TILE_SIZE) + self.west_start, (0 * glob::TILE_SIZE) + self.south_start);
				index += 1;
			}
			corners_image.save(format!("{}-corners.png", &self.output_name)).unwrap();
		}

		let mut prefabs: HashMap<u8, image::DynamicImage> =  HashMap::new();
		if self.pure_horizontal {
			let corner_img = img.crop(glob::TILE_SIZE * self.horizontal_x, glob::TILE_SIZE * self.horizontal_y, glob::TILE_SIZE, glob::TILE_SIZE);
			prefabs.insert(glob::HORIZONTAL, corner_img);
		}
		if self.pure_vertical {
			let corner_img = img.crop(glob::TILE_SIZE * self.vertical_x, glob::TILE_SIZE * self.vertical_y, glob::TILE_SIZE, glob::TILE_SIZE);
			prefabs.insert(glob::VERTICAL, corner_img);
		}
		if self.pure_flat {
			let corner_img = img.crop(glob::TILE_SIZE * self.flat_x, glob::TILE_SIZE * self.flat_y, glob::TILE_SIZE, glob::TILE_SIZE);
			prefabs.insert(glob::FLAT, corner_img);
		}

		return (corners, prefabs);
	}
	pub fn get_corner_params(&self, corner_dir: u8, corner_type: u8) -> (u32, u32, u32, u32) {
		match corner_dir {
			glob::NE_INDEX => {
				match corner_type {
					glob::CONVEX => (glob::TILE_SIZE * self.ne_convex_x + self.east_start, glob::TILE_SIZE * self.ne_convex_y + self.north_start, self.east_step, self.north_step),
					glob::CONCAVE => (glob::TILE_SIZE * self.ne_concave_x + self.east_start, glob::TILE_SIZE * self.ne_concave_y + self.north_start, self.east_step, self.north_step),
					glob::HORIZONTAL => (glob::TILE_SIZE * self.ne_horizontal_x + self.east_start, glob::TILE_SIZE * self.ne_horizontal_y + self.north_start, self.east_step, self.north_step),
					glob::VERTICAL => (glob::TILE_SIZE * self.ne_vertical_x + self.east_start, glob::TILE_SIZE * self.ne_vertical_y + self.north_start, self.east_step, self.north_step),
					glob::FLAT => (glob::TILE_SIZE * self.ne_flat_x + self.east_start, glob::TILE_SIZE * self.ne_flat_y + self.north_start, self.east_step, self.north_step),
					_ => panic!("get_corner_params -> NE_INDEX -> {}", corner_type)
				}
			},
			glob::SE_INDEX => {
				match corner_type {
					glob::CONVEX => (glob::TILE_SIZE * self.se_convex_x + self.east_start, glob::TILE_SIZE * self.se_convex_y + self.south_start, self.east_step, self.south_step),
					glob::CONCAVE => (glob::TILE_SIZE * self.se_concave_x + self.east_start, glob::TILE_SIZE * self.se_concave_y + self.south_start, self.east_step, self.south_step),
					glob::HORIZONTAL => (glob::TILE_SIZE * self.se_horizontal_x + self.east_start, glob::TILE_SIZE * self.se_horizontal_y + self.south_start, self.east_step, self.south_step),
					glob::VERTICAL => (glob::TILE_SIZE * self.se_vertical_x + self.east_start, glob::TILE_SIZE * self.se_vertical_y + self.south_start, self.east_step, self.south_step),
					glob::FLAT => (glob::TILE_SIZE * self.se_flat_x + self.east_start, glob::TILE_SIZE * self.se_flat_y + self.south_start, self.east_step, self.south_step),
					_ => panic!("get_corner_params -> SE_INDEX -> {}", corner_type)
				}
			},
			glob::SW_INDEX => {
				match corner_type {
					glob::CONVEX => (glob::TILE_SIZE * self.sw_convex_x + self.west_start, glob::TILE_SIZE * self.sw_convex_y + self.south_start, self.west_step, self.south_step),
					glob::CONCAVE => (glob::TILE_SIZE * self.sw_concave_x + self.west_start, glob::TILE_SIZE * self.sw_concave_y + self.south_start, self.west_step, self.south_step),
					glob::HORIZONTAL => (glob::TILE_SIZE * self.sw_horizontal_x + self.west_start, glob::TILE_SIZE * self.sw_horizontal_y + self.south_start, self.west_step, self.south_step),
					glob::VERTICAL => (glob::TILE_SIZE * self.sw_vertical_x + self.west_start, glob::TILE_SIZE * self.sw_vertical_y + self.south_start, self.west_step, self.south_step),
					glob::FLAT => (glob::TILE_SIZE * self.sw_flat_x + self.west_start, glob::TILE_SIZE * self.sw_flat_y + self.south_start, self.west_step, self.south_step),
					_ => panic!("get_corner_params -> SW_INDEX -> {}", corner_type)
				}
			},
			glob::NW_INDEX => {
				match corner_type {
					glob::CONVEX => (glob::TILE_SIZE * self.nw_convex_x + self.west_start, glob::TILE_SIZE * self.nw_convex_y + self.north_start, self.west_step, self.north_step),
					glob::CONCAVE => (glob::TILE_SIZE * self.nw_concave_x + self.west_start, glob::TILE_SIZE * self.nw_concave_y + self.north_start, self.west_step, self.north_step),
					glob::HORIZONTAL => (glob::TILE_SIZE * self.nw_horizontal_x + self.west_start, glob::TILE_SIZE * self.nw_horizontal_y + self.north_start, self.west_step, self.north_step),
					glob::VERTICAL => (glob::TILE_SIZE * self.nw_vertical_x + self.west_start, glob::TILE_SIZE * self.nw_vertical_y + self.north_start, self.west_step, self.north_step),
					glob::FLAT => (glob::TILE_SIZE * self.nw_flat_x + self.west_start, glob::TILE_SIZE * self.nw_flat_y + self.north_start, self.west_step, self.north_step),
					_ => panic!("get_corner_params -> NW_INDEX -> {}", corner_type)
				}
			},
			_ => panic!("get_corner_params -> {}", corner_dir)
		}
	}
}

pub fn load_configs() -> PrefHolder {
	let mut file = File::open("./config.yaml").expect("Unable to open config file.");
	let mut contents = String::new();
	file.read_to_string(&mut contents).expect("Unable to read config file.");
	let docs = YamlLoader::load_from_str(&contents).unwrap();
	let doc = &docs[0];
	let conf_center_x = doc["center_x"].as_i64().unwrap() as u32;
	let conf_center_y = doc["center_y"].as_i64().unwrap() as u32;
	return PrefHolder {
		file_to_open: doc["file_to_open"].as_str().unwrap().to_string(),
		output_name: doc["output_name"].as_str().unwrap().to_string(),

		center_x: conf_center_x,
		center_y: conf_center_y,

		//Derivatives
		west_start: glob::ORIGIN_X,
		west_step: conf_center_x,
		east_start: conf_center_x,
		east_step: glob::TILE_SIZE - conf_center_x,
		north_start: glob::ORIGIN_Y,
		north_step: conf_center_y,
		south_start: conf_center_y,
		south_step: glob::TILE_SIZE - conf_center_y,

		produce_corners: doc["produce_corners"].as_bool().unwrap(),

		pure_horizontal: doc["pure_horizontal"].as_bool().unwrap(),
		pure_vertical: doc["pure_vertical"].as_bool().unwrap(),
		pure_flat: doc["pure_flat"].as_bool().unwrap(),

		horizontal_x: doc["horizontal_x"].as_i64().unwrap() as u32,
		horizontal_y: doc["horizontal_y"].as_i64().unwrap() as u32,
		vertical_x: doc["vertical_x"].as_i64().unwrap() as u32,
		vertical_y: doc["vertical_y"].as_i64().unwrap() as u32,
		flat_x: doc["flat_x"].as_i64().unwrap() as u32,
		flat_y: doc["flat_y"].as_i64().unwrap() as u32,

		se_convex_x: doc["se_convex_x"].as_i64().unwrap() as u32,
		se_convex_y: doc["se_convex_y"].as_i64().unwrap() as u32,
		nw_convex_x: doc["nw_convex_x"].as_i64().unwrap() as u32,
		nw_convex_y: doc["nw_convex_y"].as_i64().unwrap() as u32,
		ne_convex_x: doc["ne_convex_x"].as_i64().unwrap() as u32,
		ne_convex_y: doc["ne_convex_y"].as_i64().unwrap() as u32,
		sw_convex_x: doc["sw_convex_x"].as_i64().unwrap() as u32,
		sw_convex_y: doc["sw_convex_y"].as_i64().unwrap() as u32,
	
		se_concave_x: doc["se_concave_x"].as_i64().unwrap() as u32,
		se_concave_y: doc["se_concave_y"].as_i64().unwrap() as u32,
		nw_concave_x: doc["nw_concave_x"].as_i64().unwrap() as u32,
		nw_concave_y: doc["nw_concave_y"].as_i64().unwrap() as u32,
		ne_concave_x: doc["ne_concave_x"].as_i64().unwrap() as u32,
		ne_concave_y: doc["ne_concave_y"].as_i64().unwrap() as u32,
		sw_concave_x: doc["sw_concave_x"].as_i64().unwrap() as u32,
		sw_concave_y: doc["sw_concave_y"].as_i64().unwrap() as u32,
	
		se_horizontal_x: doc["se_horizontal_x"].as_i64().unwrap() as u32,
		se_horizontal_y: doc["se_horizontal_y"].as_i64().unwrap() as u32,
		nw_horizontal_x: doc["nw_horizontal_x"].as_i64().unwrap() as u32,
		nw_horizontal_y: doc["nw_horizontal_y"].as_i64().unwrap() as u32,
		ne_horizontal_x: doc["ne_horizontal_x"].as_i64().unwrap() as u32,
		ne_horizontal_y: doc["ne_horizontal_y"].as_i64().unwrap() as u32,
		sw_horizontal_x: doc["sw_horizontal_x"].as_i64().unwrap() as u32,
		sw_horizontal_y: doc["sw_horizontal_y"].as_i64().unwrap() as u32,
	
		se_vertical_x: doc["se_vertical_x"].as_i64().unwrap() as u32,
		se_vertical_y: doc["se_vertical_y"].as_i64().unwrap() as u32,
		nw_vertical_x: doc["nw_vertical_x"].as_i64().unwrap() as u32,
		nw_vertical_y: doc["nw_vertical_y"].as_i64().unwrap() as u32,
		ne_vertical_x: doc["ne_vertical_x"].as_i64().unwrap() as u32,
		ne_vertical_y: doc["ne_vertical_y"].as_i64().unwrap() as u32,
		sw_vertical_x: doc["sw_vertical_x"].as_i64().unwrap() as u32,
		sw_vertical_y: doc["sw_vertical_y"].as_i64().unwrap() as u32,
	
		se_flat_x: doc["se_flat_x"].as_i64().unwrap() as u32,
		se_flat_y: doc["se_flat_y"].as_i64().unwrap() as u32,
		nw_flat_x: doc["nw_flat_x"].as_i64().unwrap() as u32,
		nw_flat_y: doc["nw_flat_y"].as_i64().unwrap() as u32,
		ne_flat_x: doc["ne_flat_x"].as_i64().unwrap() as u32,
		ne_flat_y: doc["ne_flat_y"].as_i64().unwrap() as u32,
		sw_flat_x: doc["sw_flat_x"].as_i64().unwrap() as u32,
		sw_flat_y: doc["sw_flat_y"].as_i64().unwrap() as u32,
	}
}
