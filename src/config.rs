use image::imageops;
use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;
use yaml_rust::YamlLoader;

use super::glob;

pub struct PrefHolder {
	pub file_to_open: String,
	pub output_name: String,

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

	pub produce_corners: bool,

	pub prefabs: Option<HashMap<u8, [u32; 2]>>,
	pub tg_corners: Option<HashMap<u8, [u32; 4]>>,

	pub dmi_version: String,

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

	pub se_flat_x: Option<u32>,
	pub se_flat_y: Option<u32>,
	pub nw_flat_x: Option<u32>,
	pub nw_flat_y: Option<u32>,
	pub ne_flat_x: Option<u32>,
	pub ne_flat_y: Option<u32>,
	pub sw_flat_x: Option<u32>,
	pub sw_flat_y: Option<u32>,

	pub is_diagonal: bool,
}

impl PrefHolder {
	pub fn build_corners_and_prefabs(
		&self,
	) -> (
		Vec<Vec<image::DynamicImage>>,
		HashMap<u8, image::DynamicImage>,
	) {
		let mut img = image::open(&self.file_to_open).expect("Failed to open input png file.");

		//Index defined by glob::CORNER_DIRS
		let mut corners: Vec<Vec<image::DynamicImage>> = vec![vec![], vec![], vec![], vec![]];

		let corner_types: &[u8];
		let corners_length;
		if self.is_diagonal {
			corner_types = &glob::CORNER_TYPES_DIAGONAL;
			corners_length = glob::CORNER_TYPES_DIAGONAL.len() as u32;
		} else {
			corner_types = &glob::CORNER_TYPES_CARDINAL;
			corners_length = glob::CORNER_TYPES_CARDINAL.len() as u32;
		}

		for corner_dir in glob::CORNER_DIRS.iter() {
			for corner_type in corner_types.iter() {
				let corner_params = self.get_corner_params(*corner_dir, *corner_type);
				let corner_img = img.crop(
					corner_params.0,
					corner_params.1,
					corner_params.2,
					corner_params.3,
				);
				corners[*corner_dir as usize].push(corner_img);
			}
		}

		if self.produce_corners {
			let mut corners_image = image::DynamicImage::new_rgba8(
				corners_length * self.icon_size_x,
				1 * self.icon_size_y,
			);
			let mut index = 0;
			for corner_type in corner_types.iter() {
				imageops::replace(
					&mut corners_image,
					&corners[glob::NW_INDEX as usize][*corner_type as usize],
					(index * self.icon_size_x) + self.west_start,
					(0 * self.icon_size_y) + self.north_start,
				);
				imageops::replace(
					&mut corners_image,
					&corners[glob::NE_INDEX as usize][*corner_type as usize],
					(index * self.icon_size_x) + self.east_start,
					(0 * self.icon_size_y) + self.north_start,
				);
				imageops::replace(
					&mut corners_image,
					&corners[glob::SE_INDEX as usize][*corner_type as usize],
					(index * self.icon_size_x) + self.east_start,
					(0 * self.icon_size_y) + self.south_start,
				);
				imageops::replace(
					&mut corners_image,
					&corners[glob::SW_INDEX as usize][*corner_type as usize],
					(index * self.icon_size_x) + self.west_start,
					(0 * self.icon_size_y) + self.south_start,
				);
				index += 1;
			}
			corners_image
				.save(format!("{}-corners.png", &self.output_name))
				.unwrap();
		}

		let mut prefabs: HashMap<u8, image::DynamicImage> = HashMap::new();
		if self.tg_corners != None {
			for (signature, location) in self.tg_corners.as_ref().unwrap() {
				let corner_img = img.crop(
					self.icon_size_x * location[0],
					self.icon_size_y * location[1],
					self.icon_size_x,
					self.icon_size_y,
				);
				let mut big_img = img.crop(
					self.icon_size_x * location[2],
					self.icon_size_y * location[3],
					self.icon_size_x,
					self.icon_size_y,
				);
				imageops::overlay(&mut big_img, &corner_img, 0, 0);
				prefabs.insert(*signature, big_img);
			}
		}
		if self.prefabs != None {
			for (signature, location) in self.prefabs.as_ref().unwrap() {
				if prefabs.contains_key(signature) {
					continue; //Already handled by tg_corners config.
				}
				let prefab_img = img.crop(
					self.icon_size_x * location[0],
					self.icon_size_y * location[1],
					self.icon_size_x,
					self.icon_size_y,
				);
				prefabs.insert(*signature, prefab_img);
			}
		}

		return (corners, prefabs);
	}
	pub fn get_corner_params(&self, corner_dir: u8, corner_type: u8) -> (u32, u32, u32, u32) {
		match corner_dir {
			glob::NE_INDEX => match corner_type {
				glob::CONVEX => (
					self.icon_size_x * self.ne_convex_x + self.east_start,
					self.icon_size_y * self.ne_convex_y + self.north_start,
					self.east_step,
					self.north_step,
				),
				glob::CONCAVE => (
					self.icon_size_x * self.ne_concave_x + self.east_start,
					self.icon_size_y * self.ne_concave_y + self.north_start,
					self.east_step,
					self.north_step,
				),
				glob::HORIZONTAL => (
					self.icon_size_x * self.ne_horizontal_x + self.east_start,
					self.icon_size_y * self.ne_horizontal_y + self.north_start,
					self.east_step,
					self.north_step,
				),
				glob::VERTICAL => (
					self.icon_size_x * self.ne_vertical_x + self.east_start,
					self.icon_size_y * self.ne_vertical_y + self.north_start,
					self.east_step,
					self.north_step,
				),
				glob::FLAT => (
					self.icon_size_x * self.ne_flat_x.unwrap() + self.east_start,
					self.icon_size_y * self.ne_flat_y.unwrap() + self.north_start,
					self.east_step,
					self.north_step,
				),
				_ => panic!("get_corner_params -> NE_INDEX -> {}", corner_type),
			},
			glob::SE_INDEX => match corner_type {
				glob::CONVEX => (
					self.icon_size_x * self.se_convex_x + self.east_start,
					self.icon_size_y * self.se_convex_y + self.south_start,
					self.east_step,
					self.south_step,
				),
				glob::CONCAVE => (
					self.icon_size_x * self.se_concave_x + self.east_start,
					self.icon_size_y * self.se_concave_y + self.south_start,
					self.east_step,
					self.south_step,
				),
				glob::HORIZONTAL => (
					self.icon_size_x * self.se_horizontal_x + self.east_start,
					self.icon_size_y * self.se_horizontal_y + self.south_start,
					self.east_step,
					self.south_step,
				),
				glob::VERTICAL => (
					self.icon_size_x * self.se_vertical_x + self.east_start,
					self.icon_size_y * self.se_vertical_y + self.south_start,
					self.east_step,
					self.south_step,
				),
				glob::FLAT => (
					self.icon_size_x * self.se_flat_x.unwrap() + self.east_start,
					self.icon_size_y * self.se_flat_y.unwrap() + self.south_start,
					self.east_step,
					self.south_step,
				),
				_ => panic!("get_corner_params -> SE_INDEX -> {}", corner_type),
			},
			glob::SW_INDEX => match corner_type {
				glob::CONVEX => (
					self.icon_size_x * self.sw_convex_x + self.west_start,
					self.icon_size_y * self.sw_convex_y + self.south_start,
					self.west_step,
					self.south_step,
				),
				glob::CONCAVE => (
					self.icon_size_x * self.sw_concave_x + self.west_start,
					self.icon_size_y * self.sw_concave_y + self.south_start,
					self.west_step,
					self.south_step,
				),
				glob::HORIZONTAL => (
					self.icon_size_x * self.sw_horizontal_x + self.west_start,
					self.icon_size_y * self.sw_horizontal_y + self.south_start,
					self.west_step,
					self.south_step,
				),
				glob::VERTICAL => (
					self.icon_size_x * self.sw_vertical_x + self.west_start,
					self.icon_size_y * self.sw_vertical_y + self.south_start,
					self.west_step,
					self.south_step,
				),
				glob::FLAT => (
					self.icon_size_x * self.sw_flat_x.unwrap() + self.west_start,
					self.icon_size_y * self.sw_flat_y.unwrap() + self.south_start,
					self.west_step,
					self.south_step,
				),
				_ => panic!("get_corner_params -> SW_INDEX -> {}", corner_type),
			},
			glob::NW_INDEX => match corner_type {
				glob::CONVEX => (
					self.icon_size_x * self.nw_convex_x + self.west_start,
					self.icon_size_y * self.nw_convex_y + self.north_start,
					self.west_step,
					self.north_step,
				),
				glob::CONCAVE => (
					self.icon_size_x * self.nw_concave_x + self.west_start,
					self.icon_size_y * self.nw_concave_y + self.north_start,
					self.west_step,
					self.north_step,
				),
				glob::HORIZONTAL => (
					self.icon_size_x * self.nw_horizontal_x + self.west_start,
					self.icon_size_y * self.nw_horizontal_y + self.north_start,
					self.west_step,
					self.north_step,
				),
				glob::VERTICAL => (
					self.icon_size_x * self.nw_vertical_x + self.west_start,
					self.icon_size_y * self.nw_vertical_y + self.north_start,
					self.west_step,
					self.north_step,
				),
				glob::FLAT => (
					self.icon_size_x * self.nw_flat_x.unwrap() + self.west_start,
					self.icon_size_y * self.nw_flat_y.unwrap() + self.north_start,
					self.west_step,
					self.north_step,
				),
				_ => panic!("get_corner_params -> NW_INDEX -> {}", corner_type),
			},
			_ => panic!("get_corner_params -> {}", corner_dir),
		}
	}
}

pub fn read_some_u32_config(source: &yaml_rust::yaml::Yaml, index: &str) -> Option<u32> {
	let config = &source[index];
	if config.is_badvalue() {
		return None;
	}
	Some(source[index].as_i64().unwrap() as u32)
}

pub fn load_configs() -> PrefHolder {
	let mut file = File::open("./config.yaml").expect("Unable to open config file.");
	let mut contents = String::new();
	file.read_to_string(&mut contents)
		.expect("Unable to read config file.");
	let docs = YamlLoader::load_from_str(&contents).unwrap();
	let doc = &docs[0];

	let icon_size_x = doc["icon_size_x"].as_i64().unwrap() as u32;
	let icon_size_y = doc["icon_size_y"].as_i64().unwrap() as u32;
	let center_x = doc["center_x"].as_i64().unwrap() as u32;
	let center_y = doc["center_y"].as_i64().unwrap() as u32;

	let prefabs;
	if doc["prefabs"].is_badvalue() {
		prefabs = None;
	} else {
		let mut prefab_map: HashMap<u8, [u32; 2]> = HashMap::new();
		let yaml_prefabs = doc["prefabs"].as_hash().unwrap();
		for (prefab_signature, x_and_y_hash) in yaml_prefabs.iter() {
			let x_and_y = [
				x_and_y_hash["x"].as_i64().unwrap() as u32,
				x_and_y_hash["y"].as_i64().unwrap() as u32,
			];
			prefab_map.insert(prefab_signature.as_i64().unwrap() as u8, x_and_y);
		}
		prefabs = Some(prefab_map);
	}

	let tg_corners;
	if doc["tg_corners"].is_badvalue() {
		tg_corners = None;
	} else {
		let mut tg_corners_map: HashMap<u8, [u32; 4]> = HashMap::new();
		let yaml_tg_corners = doc["tg_corners"].as_hash().unwrap();
		for (tg_corners_signature, corner_coords_hash) in yaml_tg_corners.iter() {
			let corner_coords = [
				corner_coords_hash["corner_x"].as_i64().unwrap() as u32,
				corner_coords_hash["corner_y"].as_i64().unwrap() as u32,
				corner_coords_hash["big_x"].as_i64().unwrap() as u32,
				corner_coords_hash["big_y"].as_i64().unwrap() as u32,
			];
			tg_corners_map.insert(tg_corners_signature.as_i64().unwrap() as u8, corner_coords);
		}
		tg_corners = Some(tg_corners_map);
	}

	let produce_corners;
	if doc["produce_corners"].is_badvalue() {
		produce_corners = false;
	} else {
		produce_corners = doc["produce_corners"].as_bool().unwrap();
	}

	let se_flat_x = read_some_u32_config(&doc, "se_flat_x");
	let se_flat_y = read_some_u32_config(&doc, "se_flat_y");
	let nw_flat_x = read_some_u32_config(&doc, "nw_flat_x");
	let nw_flat_y = read_some_u32_config(&doc, "nw_flat_y");
	let ne_flat_x = read_some_u32_config(&doc, "ne_flat_x");
	let ne_flat_y = read_some_u32_config(&doc, "ne_flat_y");
	let sw_flat_x = read_some_u32_config(&doc, "sw_flat_x");
	let sw_flat_y = read_some_u32_config(&doc, "sw_flat_y");

	let is_diagonal = se_flat_y != None
		&& se_flat_y != None
		&& nw_flat_x != None
		&& nw_flat_y != None
		&& ne_flat_x != None
		&& ne_flat_y != None
		&& sw_flat_x != None
		&& sw_flat_y != None;

	return PrefHolder {
		file_to_open: doc["file_to_open"].as_str().unwrap().to_string(),
		output_name: doc["output_name"].as_str().unwrap().to_string(),

		icon_size_x,
		icon_size_y,

		center_x,
		center_y,

		//Derivatives
		west_start: glob::ORIGIN_X,
		west_step: center_x,
		east_start: center_x,
		east_step: icon_size_x - center_x,
		north_start: glob::ORIGIN_Y,
		north_step: center_y,
		south_start: center_y,
		south_step: icon_size_y - center_y,

		produce_corners,

		prefabs,
		tg_corners,

		dmi_version: doc["dmi_version"].as_str().unwrap().to_string(),

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
		se_flat_x,
		se_flat_y,
		nw_flat_x,
		nw_flat_y,
		ne_flat_x,
		ne_flat_y,
		sw_flat_x,
		sw_flat_y,
		is_diagonal,
	};
}
