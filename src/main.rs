#![allow(dead_code)]

//Internal modules.
mod config;
mod dmi;
mod error;
mod glob;
mod helpers;

//To do the image manipulation.
use image::imageops;

//To export the string dmi metadata signature.
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

	match &prefs.file_to_open {
		Some(thing) => args.push(thing.clone()),
		None => (),
	};

	let mut icons_built = 0;
	for image_path_string in args.iter() {
		//println!("Debug: image_path_string: {}", image_path_string);
		let path = Path::new(&image_path_string);
		let mut file;
		match File::open(&path) {
			Ok(f) => file = f,
			Err(e) => {
				println!("Wrong file path: {:?}", e);
				dont_disappear::any_key_to_continue::default();
				return;
			}
		};
		let mut contents = Vec::new();
		if let Err(e) = file.read_to_end(&mut contents) {
			println!("Unable to read file: {:?}", e);
			dont_disappear::any_key_to_continue::default();
			return;
		};
		let cursor = Cursor::new(contents);

		let mut formatted_file_name = image_path_string.clone();

		let dot_offset = image_path_string
			.find('.')
			.unwrap_or(image_path_string.len());
		formatted_file_name = formatted_file_name.drain(..dot_offset).collect(); //Here we remove everything after the dot. Whether .dmi or .png is the same for us.

		let building_return = build_icons(cursor, formatted_file_name, &prefs, icons_built);
		match building_return {
			Ok(_x) => println!("Icons built successfully."),
			Err(x) => println!("Error building icon: {}", x),
		};
		dont_disappear::any_key_to_continue::default();
		icons_built += 1;
	}

	println!("Program finished.");
	dont_disappear::any_key_to_continue::default();
}

fn build_icons(
	input: std::io::Cursor<Vec<u8>>,
	file_string_path: String,
	prefs: &config::PrefHolder,
	icons_built: u32,
) -> Result<bool, error::ReadError> {
	let corners_and_prefabs = prefs.build_corners_and_prefabs(input, &*file_string_path)?;
	let corners = corners_and_prefabs.0;
	let mounted_prefabs = corners_and_prefabs.1;

	let possible_icon_states = prepare_icon_states(prefs.is_diagonal);

	let number_of_icon_states = possible_icon_states.len() as u32;
	assert!(
		number_of_icon_states > 0,
		"prepare_icon_states() produced {} results",
		number_of_icon_states
	);
	let max_index = ((possible_icon_states.len() as u32 * prefs.frames_per_state) as f64)
		.sqrt()
		.ceil() as u32;

	let width = max_index * prefs.output_icon_size_x;
	let height = max_index * prefs.output_icon_size_y;
	let mut new_icon = image::DynamicImage::new_rgba8(width, height);
	let mut dmi_signature = format!(
		"# BEGIN DMI\nversion = {}\n\twidth = {}\n\theight = {}\n",
		prefs.dmi_version, prefs.output_icon_size_x, prefs.output_icon_size_y
	);
	let mut index_x = 0;
	let mut index_y = 0;

	let output_name;
	match &prefs.output_name {
		Some(thing) => {
			if icons_built == 0 {
				output_name = format!("{}", &thing)
			} else {
				output_name = format!("{}({})", &thing, icons_built + 1)
			};
		}
		None => {
			if file_string_path.len() == 0 {
				if icons_built == 0 {
					output_name = "output".to_string()
				} else {
					output_name = format!("output({})", icons_built + 1)
				};
			} else {
				output_name = format!("{}-output", helpers::trim_path_before_last_slash(file_string_path.clone()));
			};
		}
	};
	let icon_state_name;
	icon_state_name = match &prefs.base_icon_state {
		Some(thing) => thing.clone(),
		None => "icon".to_string(),
	};

	for icon_signature in possible_icon_states.iter() {
		if mounted_prefabs.contains_key(icon_signature) {
			for frame in 0..prefs.frames_per_state {
				imageops::replace(
					&mut new_icon,
					&mounted_prefabs[icon_signature][frame as usize],
					(index_x * prefs.output_icon_size_x) + prefs.output_west_start,
					(index_y * prefs.output_icon_size_y) + prefs.output_north_start,
				);
				if index_x > max_index - 2 {
					index_x = 0;
					index_y += 1;
				} else {
					index_x += 1;
				};
			};
		} else {
			for frame in 0..prefs.frames_per_state {
				let frame_img = &corners
					.get(&glob::NW_INDEX)
					.unwrap()
					.get(&helpers::smooth_dir_to_corner_type(
						glob::NW_INDEX,
						*icon_signature,
					))
					.unwrap()[frame as usize];
				imageops::overlay(
					&mut new_icon,
					frame_img,
					(index_x * prefs.output_icon_size_x) + prefs.output_west_start,
					(index_y * prefs.output_icon_size_y) + prefs.output_north_start,
				);
				let frame_img = &corners
					.get(&glob::NE_INDEX)
					.unwrap()
					.get(&helpers::smooth_dir_to_corner_type(
						glob::NE_INDEX,
						*icon_signature,
					))
					.unwrap()[frame as usize];
				imageops::overlay(
					&mut new_icon,
					frame_img,
					(index_x * prefs.output_icon_size_x) + prefs.output_east_start,
					(index_y * prefs.output_icon_size_y) + prefs.output_north_start,
				);
				let frame_img = &corners
					.get(&glob::SE_INDEX)
					.unwrap()
					.get(&helpers::smooth_dir_to_corner_type(
						glob::SE_INDEX,
						*icon_signature,
					))
					.unwrap()[frame as usize];
				imageops::overlay(
					&mut new_icon,
					frame_img,
					(index_x * prefs.output_icon_size_x) + prefs.output_east_start,
					(index_y * prefs.output_icon_size_y) + prefs.output_south_start,
				);
				let frame_img = &corners
					.get(&glob::SW_INDEX)
					.unwrap()
					.get(&helpers::smooth_dir_to_corner_type(
						glob::SW_INDEX,
						*icon_signature,
					))
					.unwrap()[frame as usize];
				imageops::overlay(
					&mut new_icon,
					frame_img,
					(index_x * prefs.output_icon_size_x) + prefs.output_west_start,
					(index_y * prefs.output_icon_size_y) + prefs.output_south_start,
				);
				if index_x > max_index - 2 {
					index_x = 0;
					index_y += 1;
				} else {
					index_x += 1;
				};
			}
		};
		let string_signature;
		if prefs.frames_per_state == 1 {
			string_signature = format!(
				"state = \"{}-{}\"\n\tdirs = 1\n\tframes = 1\n",
				&icon_state_name, icon_signature
			)
		} else {
			let delay_vec = match &prefs.delay {
				Some(thing) => thing,
				None => {return Err(error::ReadError::Generic("Error while trying to read the delay preference, no value found. This shouldn't happen.".to_string()))}
				};
			let mut delay_signature = vec![];
			for delay_value in delay_vec.iter() {
				delay_signature.push(delay_value.to_string())
			}
			let delay_signature = delay_signature.join(",");
			string_signature = format!(
				"state = \"{}-{}\"\n\tdirs = 1\n\tframes = {}\n\tdelay = {}\n",
				&icon_state_name, icon_signature, prefs.frames_per_state, delay_signature
			)
		};
		dmi_signature.push_str(&string_signature);
	}
	dmi_signature += "# END DMI\n";

	let mut image_bytes: Vec<u8> = vec![];
	new_icon
		.write_to(&mut image_bytes, image::ImageOutputFormat::Png)
		.map_err(|x| error::ReadError::Image(x))?;

	let mut dmi = dmi::dmi_from_vec(&image_bytes)?;
	dmi.write_ztxt_chunk(dmi_signature)?;

	let output_name = format!("{}.dmi", output_name);
	let dmi_path = Path::new(&output_name);
	dmi.write(dmi_path)?;

	println!(
		"{} icon states produced, with {} frames each, for a total of {} frames.",
		number_of_icon_states,
		prefs.frames_per_state,
		number_of_icon_states * prefs.frames_per_state
	);
	Ok(true)
}

fn prepare_icon_states(is_diagonal: bool) -> Vec<u8> {
	let mut icon_variations: Vec<u8> = vec![];
	for smooth_dirs in glob::NONE..=glob::ADJ_ALL {
		let combination_key = helpers::smooth_dir_to_combination_key(smooth_dirs, is_diagonal);
		if icon_variations.contains(&combination_key) {
			continue;
		};
		icon_variations.push(combination_key);
	}
	icon_variations.sort();
	return icon_variations;
}
