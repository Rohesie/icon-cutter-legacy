#![allow(dead_code)]

//Internal modules.
mod config;
mod dmi;
mod glob;
mod helpers;

//To do the image manipulation.
use image::imageops;

//To export the string dmi metadata signature.
use dmi::error::DmiReadError;
use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::io::Cursor;
use std::path::Path;

fn main() {
	let mut args: Vec<String> = env::args().collect();

	let self_path = args.remove(0);

	let prefs;
	match config::load_configs(self_path.clone()) {
		Ok(thing) => prefs = thing,
		Err(e) => {
			println!("Failed to load configs: {}", e);
			dont_disappear::any_key_to_continue::default();
			return;
		}
	};

	for image_path_string in args.iter() {
		let path = Path::new(&image_path_string);
		let mut file;
		match File::open(&path) {
			Ok(f) => file = f,
			Err(e) => {
				println!("{:?}", e);
				dont_disappear::any_key_to_continue::default();
				return;
			}
		}
		let mut contents = Vec::new();
		if let Err(e) = file.read_to_end(&mut contents) {
			println!("{:?}", e);
			dont_disappear::any_key_to_continue::default();
			return;
		};
		let cursor = Cursor::new(contents);

		let mut formatted_file_name = image_path_string.clone();

		let dot_offset = image_path_string
			.find('.')
			.unwrap_or(image_path_string.len());
		formatted_file_name = formatted_file_name.drain(..dot_offset).collect(); //Here we remove everything after the dot. Whether .dmi or .png is the same for us.

		let building_return = build_walls(cursor, formatted_file_name, &prefs);
		match building_return {
			Ok(_x) => println!("Wall built successfully."),
			Err(x) => println!("Error building wall: {}", x),
		}
		dont_disappear::any_key_to_continue::default();
	}

	println!("Program finished.");
	dont_disappear::any_key_to_continue::default();
}

fn build_walls(
	input: std::io::Cursor<Vec<u8>>,
	file_string_path: String,
	prefs: &config::PrefHolder,
) -> Result<bool, DmiReadError> {
	let corners_and_prefabs = prefs.build_corners_and_prefabs(input, &*file_string_path);
	let corners = corners_and_prefabs.0;
	let mounted_prefabs = corners_and_prefabs.1;
	let prefab_keys = &prefs.prefab_keys;

	let possible_walls = prepare_walls(prefs.is_diagonal);

	let number_of_walls = possible_walls.len() as u32;
	assert!(
		number_of_walls > 0,
		"prepare_walls() produced {} results",
		number_of_walls
	);
	let max_index = (possible_walls.len() as f64).sqrt().ceil() as u32;

	let width = max_index * prefs.icon_size_x;
	let height = max_index * prefs.icon_size_y;
	let mut new_wall = image::DynamicImage::new_rgba8(width, height);
	let mut dmi_signature = format!(
		"# BEGIN DMI\nversion = {}\n\twidth = {}\n\theight = {}\n",
		prefs.dmi_version, prefs.icon_size_x, prefs.icon_size_y
	);
	let mut index_x = 0;
	let mut index_y = 0;

	let output_name = match &prefs.output_name {
		Some(thing) => thing.clone(),
		None => {
			let mut file_name = file_string_path.clone();
			file_name = helpers::trim_path_before_last_slash(file_name);
			file_name
		}
	};
	//let output_name = prefs.output_name.to_string();
	for wall_signature in possible_walls.iter() {
		let has_prefab = match prefab_keys {
			None => false,
			Some(map) => {
				if map.contains(wall_signature) {
					true
				} else {
					false
				}
			}
		};
		if has_prefab {
			imageops::replace(
				&mut new_wall,
				&mounted_prefabs[wall_signature],
				(index_x * prefs.icon_size_x) + prefs.west_start,
				(index_y * prefs.icon_size_y) + prefs.north_start,
			)
		} else {
			imageops::overlay(
				&mut new_wall,
				&corners[glob::NW_INDEX as usize]
					[helpers::smooth_dir_to_corner_type(glob::NW_INDEX, *wall_signature) as usize],
				(index_x * prefs.icon_size_x) + prefs.west_start,
				(index_y * prefs.icon_size_y) + prefs.north_start,
			);
			imageops::overlay(
				&mut new_wall,
				&corners[glob::NE_INDEX as usize]
					[helpers::smooth_dir_to_corner_type(glob::NE_INDEX, *wall_signature) as usize],
				(index_x * prefs.icon_size_x) + prefs.east_start,
				(index_y * prefs.icon_size_y) + prefs.north_start,
			);
			imageops::overlay(
				&mut new_wall,
				&corners[glob::SE_INDEX as usize]
					[helpers::smooth_dir_to_corner_type(glob::SE_INDEX, *wall_signature) as usize],
				(index_x * prefs.icon_size_x) + prefs.east_start,
				(index_y * prefs.icon_size_y) + prefs.south_start,
			);
			imageops::overlay(
				&mut new_wall,
				&corners[glob::SW_INDEX as usize]
					[helpers::smooth_dir_to_corner_type(glob::SW_INDEX, *wall_signature) as usize],
				(index_x * prefs.icon_size_x) + prefs.west_start,
				(index_y * prefs.icon_size_y) + prefs.south_start,
			);
		}
		let string_signature = format!(
			"state = \"{}-{}\"\n\tdirs = 1\n\tframes = 1\n",
			&output_name, wall_signature
		);
		dmi_signature.push_str(&string_signature);
		if index_x > max_index - 2 {
			index_x = 0;
			index_y += 1;
		} else {
			index_x += 1;
		}
	}
	dmi_signature += "# END DMI\n";

	let mut image_bytes: Vec<u8> = vec![];
	new_wall
		.write_to(&mut image_bytes, image::ImageOutputFormat::Png)
		.map_err(|x| DmiReadError::Image(x))?;

	let mut dmi = dmi::dmi_from_vec(&image_bytes)?;
	dmi.write_ztxt_chunk(dmi_signature)
		.map_err(|x| DmiReadError::Io(x))?;

	let path_name = format!("{}-output.dmi", file_string_path);
	let dmi_path = Path::new(&path_name);
	dmi.write(dmi_path)?;

	println!("Number of wall sprites produced: {}", number_of_walls);
	Ok(true)
}

fn prepare_walls(is_diagonal: bool) -> Vec<u8> {
	let mut wall_variations: Vec<u8> = vec![];
	for smooth_dirs in glob::NONE..=glob::ADJ_ALL {
		let combination_key = helpers::smooth_dir_to_combination_key(smooth_dirs, is_diagonal);
		if !wall_variations.contains(&combination_key) {
			wall_variations.push(combination_key);
		}
	}
	wall_variations.sort();
	return wall_variations;
}
