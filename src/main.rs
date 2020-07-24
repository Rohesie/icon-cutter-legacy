//Internal modules.
mod glob;
mod config;
mod helpers;

//To do the image manipulation.
use image::imageops;

//To export the string dmi metadata signature.
use std::fs::File;
use std::path::Path;
use std::io::prelude::*;

fn main() {
	//let mut img = image::open("./wall_brick_final.png").unwrap();
	//println!("dimensions {:?}", img.dimensions());
	//println!("color {:?}", img.color());
	/*
	let prefs = config::load_configs();
	println!("{:?}\n{:?}\n{:?}\n{:?}\n{:?}\n{:?}\n{:?}\n{:?}\n{:?}\n{:?}\n{:?}\n{:?}\n{:?}\n{:?}\n{:?}\n{:?}\n{:?}\n{:?}\n{:?}\n{:?}\n{:?}\n{:?}\n{:?}\n{:?}\n{:?}\n{:?}\n
	{:?}\n{:?}\n{:?}\n{:?}\n{:?}\n{:?}\n{:?}\n{:?}\n{:?}\n{:?}\n{:?}\n{:?}\n{:?}\n{:?}\n{:?}\n{:?}\n{:?}\n{:?}\n{:?}\n{:?}\n{:?}\n{:?}\n{:?}\n{:?}\n{:?}\n{:?}\n{:?}\n",
	prefs.file_to_open,
	prefs.output_name,
	prefs.center_x,
	prefs.center_y,

	prefs.west_start,
	prefs.west_step,
	prefs.east_start,
	prefs.east_step,
	prefs.north_start,
	prefs.north_step,
	prefs.south_start,
	prefs.south_step,

	prefs.produce_corners,

	prefs.se_convex_x,
	prefs.se_convex_y,
	prefs.nw_convex_x,
	prefs.nw_convex_y,
	prefs.ne_convex_x,
	prefs.ne_convex_y,
	prefs.sw_convex_x,
	prefs.sw_convex_y,

	prefs.se_concave_x,
	prefs.se_concave_y,
	prefs.nw_concave_x,
	prefs.nw_concave_y,
	prefs.ne_concave_x,
	prefs.ne_concave_y,
	prefs.sw_concave_x,
	prefs.sw_concave_y,

	prefs.se_horizontal_x,
	prefs.se_horizontal_y,
	prefs.nw_horizontal_x,
	prefs.nw_horizontal_y,
	prefs.ne_horizontal_x,
	prefs.ne_horizontal_y,
	prefs.sw_horizontal_x,
	prefs.sw_horizontal_y,

	prefs.se_vertical_x,
	prefs.se_vertical_y,
	prefs.nw_vertical_x,
	prefs.nw_vertical_y,
	prefs.ne_vertical_x,
	prefs.ne_vertical_y,
	prefs.sw_vertical_x,
	prefs.sw_vertical_y,

	prefs.se_flat_x,
	prefs.se_flat_y,
	prefs.nw_flat_x,
	prefs.nw_flat_y,
	prefs.ne_flat_x,
	prefs.ne_flat_y,
	prefs.sw_flat_x,
	prefs.sw_flat_y,
	);
	*/
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

	let mut new_wall = image::DynamicImage::new_rgba8(number_of_walls * glob::TILE_SIZE, glob::TILE_SIZE);
	let mut dmi_signature = format!("# BEGIN DMI\nversion = 4.0\n\twidth = {}\n\theight = {}\n", glob::TILE_SIZE, glob::TILE_SIZE);
	let mut index = 0;

	let output_name = prefs.output_name.to_string();
	for wall_signature in possible_walls.iter() {
		match wall_signature {
			[2, 2, 2, 2] if prefs.pure_horizontal => imageops::replace(&mut new_wall, &prefabs[&glob::HORIZONTAL], index * glob::TILE_SIZE, prefs.north_start),
			[3, 3, 3, 3] if prefs.pure_vertical => imageops::replace(&mut new_wall, &prefabs[&glob::VERTICAL], index * glob::TILE_SIZE, prefs.north_start),
			[4, 4, 4, 4] if prefs.pure_flat => imageops::replace(&mut new_wall, &prefabs[&glob::FLAT], index * glob::TILE_SIZE, prefs.north_start),
			_ => {
				imageops::replace(&mut new_wall, &corners[glob::NW_INDEX as usize][wall_signature[glob::NW_INDEX as usize] as usize], (index * glob::TILE_SIZE) + prefs.west_start, prefs.north_start);
				imageops::replace(&mut new_wall, &corners[glob::NE_INDEX as usize][wall_signature[glob::NE_INDEX as usize] as usize], (index * glob::TILE_SIZE) + prefs.east_start, prefs.north_start);
				imageops::replace(&mut new_wall, &corners[glob::SE_INDEX as usize][wall_signature[glob::SE_INDEX as usize] as usize], (index * glob::TILE_SIZE) + prefs.east_start, prefs.south_start);
				imageops::replace(&mut new_wall, &corners[glob::SW_INDEX as usize][wall_signature[glob::SW_INDEX as usize] as usize], (index * glob::TILE_SIZE) + prefs.west_start, prefs.south_start);
			}
		}
		let string_signature = format!("state = \"{}-{}-{}-{}-{}\"\n\tdirs = 1\n\tframes = 1\n", &output_name, wall_signature[glob::NE_INDEX as usize], wall_signature[glob::SE_INDEX as usize], wall_signature[glob::SW_INDEX as usize], wall_signature[glob::NW_INDEX as usize]);
		dmi_signature.push_str(&string_signature);
		index += 1;
	}

	new_wall.save(format!("{}.png", output_name)).unwrap();
	dmi_signature += "# END DMI\n";

	let path = format!("{}_dmi_signature.txt", &prefs.output_name);
	let path = Path::new(&path);
	let display = path.display();
	let mut file = match File::create(&path) {
		Err(why) => panic!("couldn't create {}: {}", display, why),
		Ok(file) => file,
	};
	match file.write_all(dmi_signature.as_bytes()) {
		Err(why) => panic!("couldn't write to {}: {}", display, why),
		Ok(_) => println!("successfully wrote to {}", display),
	}

	println!("Number of wall sprites produced: {}", number_of_walls);

}

fn prepare_walls() -> Vec<[u8; 4]> {
	let mut wall_variations: Vec<[u8; 4]> = vec![];
	for smooth_dirs in glob::NONE ..= glob::ADJ_ALL {
		let mut four_corners = [glob::NONE, glob::NONE, glob::NONE, glob::NONE];
		for corner_index in glob::CORNER_DIRS.iter() {
			four_corners[*corner_index as usize] = helpers::smooth_dir_to_corner_type(*corner_index, smooth_dirs)
		}
		if !wall_variations.contains(&four_corners) {
			wall_variations.push(four_corners);
		}
	}
	wall_variations.sort();
	return wall_variations;
}
