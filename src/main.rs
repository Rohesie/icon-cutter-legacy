#![allow(dead_code)]

//Internal modules.
mod glob;
mod config;
mod helpers;
mod dmi;

//To do the image manipulation.
use image::imageops;

//To export the string dmi metadata signature.
use std::fs::File;
use std::path::Path;
use std::io::prelude::*;

fn main() {
	build_walls();
}

fn build_walls() {
	let prefs = config::load_configs();
	let corners_and_prefabs = prefs.build_corners_and_prefabs();
	let corners = corners_and_prefabs.0;
	let prefabs = corners_and_prefabs.1;
	let possible_walls = prepare_walls();

	let number_of_walls = possible_walls.len() as u32;
	assert!(number_of_walls > 0, "prepare_walls() produced {} results", number_of_walls);
	let max_index = (possible_walls.len() as f64).sqrt().ceil() as u32;

	let mut new_wall = image::DynamicImage::new_rgba8(max_index * glob::TILE_SIZE, max_index * glob::TILE_SIZE);
	let mut dmi_signature = format!("# BEGIN DMI\nversion = 4.0\n\twidth = {}\n\theight = {}\n", glob::TILE_SIZE, glob::TILE_SIZE);
	let mut index_x = 0;
	let mut index_y = 0;

	let output_name = prefs.output_name.to_string();
	for wall_signature in possible_walls.iter() {
		match wall_signature {
			&glob::ADJ_N_S if prefs.pure_vertical => imageops::replace(&mut new_wall, &prefabs[&glob::VERTICAL], (index_x * glob::TILE_SIZE) + prefs.west_start, (index_y * glob::TILE_SIZE) + prefs.north_start),
			&glob::ADJ_E_W if prefs.pure_horizontal => imageops::replace(&mut new_wall, &prefabs[&glob::HORIZONTAL], (index_x * glob::TILE_SIZE) + prefs.west_start, (index_y * glob::TILE_SIZE) + prefs.north_start),
			&glob::ADJ_ALL if prefs.pure_flat => imageops::replace(&mut new_wall, &prefabs[&glob::FLAT], (index_x * glob::TILE_SIZE) + prefs.west_start, (index_y * glob::TILE_SIZE) + prefs.north_start),
			_ => {
				imageops::replace(&mut new_wall, &corners[glob::NW_INDEX as usize][helpers::smooth_dir_to_corner_type(glob::NW_INDEX, *wall_signature) as usize], (index_x * glob::TILE_SIZE) + prefs.west_start, (index_y * glob::TILE_SIZE) + prefs.north_start);
				imageops::replace(&mut new_wall, &corners[glob::NE_INDEX as usize][helpers::smooth_dir_to_corner_type(glob::NE_INDEX, *wall_signature) as usize], (index_x * glob::TILE_SIZE) + prefs.east_start, (index_y * glob::TILE_SIZE) + prefs.north_start);
				imageops::replace(&mut new_wall, &corners[glob::SE_INDEX as usize][helpers::smooth_dir_to_corner_type(glob::SE_INDEX, *wall_signature) as usize], (index_x * glob::TILE_SIZE) + prefs.east_start, (index_y * glob::TILE_SIZE) + prefs.south_start);
				imageops::replace(&mut new_wall, &corners[glob::SW_INDEX as usize][helpers::smooth_dir_to_corner_type(glob::SW_INDEX, *wall_signature) as usize], (index_x * glob::TILE_SIZE) + prefs.west_start, (index_y * glob::TILE_SIZE) + prefs.south_start);
			}
		}
		let string_signature = format!("state = \"{}-{}\"\n\tdirs = 1\n\tframes = 1\n", &output_name, wall_signature);
		dmi_signature.push_str(&string_signature);
		if index_x > max_index - 2 {
			index_x = 0;
			index_y += 1;
		}
		else {
			index_x += 1;
		}
	}
	dmi_signature += "# END DMI\n";

	let png_path = format!("{}.png", output_name);
	new_wall.save(png_path).unwrap();

	let png_path = format!("{}.png", output_name);
	let path = Path::new(&png_path);
	let mut png_file = File::open(&path).expect("Failed to open created png.");

	let mut image_bytes: Vec<u8> = vec![];
	png_file.read_to_end(&mut image_bytes).expect("Error while reading the created png file.");

	let mut dmi = dmi::dmi_from_vec(&image_bytes).unwrap();
	dmi.write_ztxt_chunk(dmi_signature).unwrap();

	let dmi_path = format!("{}.dmi", output_name);
	dmi.save(dmi_path).unwrap();

	println!("Number of wall sprites produced: {}", number_of_walls);

}

fn prepare_walls() -> Vec<u8> {
	let mut wall_variations: Vec<u8> = vec![];
	for smooth_dirs in glob::NONE ..= glob::ADJ_ALL {
		let combination_key = helpers::smooth_dir_to_combination_key(smooth_dirs);
		if !wall_variations.contains(&combination_key) {
			wall_variations.push(combination_key);
		}
	}
	wall_variations.sort();
	return wall_variations;
}
