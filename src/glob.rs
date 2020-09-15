	/*
	* * GLOBAL DEFINITIONS
	*/
	
	//In number of pixels
	pub const TILE_SIZE:u32 = 32;
	pub const ORIGIN_X:u32 = 0;
	pub const ORIGIN_Y:u32 = 0;

	pub const NE_INDEX:u8 = 0;
	pub const SE_INDEX:u8 = 1;
	pub const SW_INDEX:u8 = 2;
	pub const NW_INDEX:u8 = 3;

	pub const CORNER_DIRS: [u8; 4] = [NE_INDEX, SE_INDEX, SW_INDEX, NW_INDEX];

	pub const CONVEX:u8 = 0;
	pub const CONCAVE:u8 = 1;
	pub const HORIZONTAL:u8 = 2;
	pub const VERTICAL:u8 = 3;
	pub const FLAT:u8 = 4;

	pub const CORNER_TYPES: [u8; 5] = [CONVEX, CONCAVE, HORIZONTAL, VERTICAL, FLAT];

	//Dirs
	pub const NONE:u8 = 0;

	//Adjacency dirs
	pub const ADJ_N:u8 = 1<<0;
	pub const ADJ_S:u8 = 1<<1;
	pub const ADJ_E:u8 = 1<<2;
	pub const ADJ_W:u8 = 1<<3;
	pub const ADJ_NE:u8 = 1<<4;
	pub const ADJ_SE:u8 = 1<<5;
	pub const ADJ_SW:u8 = 1<<6;
	pub const ADJ_NW:u8 = 1<<7;

	pub const ADJ_CARDINALS: [u8; 4] = [ADJ_N, ADJ_E, ADJ_S, ADJ_W];

	pub const ADJ_N_S:u8 = ADJ_N | ADJ_S;
	pub const ADJ_E_W:u8 = ADJ_E | ADJ_W;
	
	pub const ADJ_ALL:u8 = !0;
