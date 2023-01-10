const CLEAR_STRING : &str = "\x1b[2J\x1b[3J\x1b[1;1H";

const WIDTH : usize = 8;
const HEIGHT : usize = 22;

enum BlockType {
	I, J, L, O, S, T, Z
}


struct Block {
	tiles : [Vec<Vec<bool>>; 4],
	rotation : usize,
	x_pos : isize,
	y_pos : isize,
	name : BlockType
}

fn create_block(name : BlockType) -> Block {
	let tiles = match name {
		BlockType::I => [
			vec![vec![false, false, false, false], vec![true, true, true, true],
				vec![false, false, false, false], vec![false, false, false, false]],
			vec![vec![false, false, true, false], vec![false, false, true, false],
				vec![false, false, true, false], vec![false, false, true, false]],
			vec![vec![false, false, false, false], vec![false, false, false, false],
				vec![true, true, true, true], vec![false, false, false, false]],
			vec![vec![false, true, false, false], vec![false, true, false, false],
				vec![false, true, false, false], vec![false, true, false, false]]
		],
		BlockType::J => [
			vec![vec![true, false, false], vec![true, true, true], vec![false, false, false]],
			vec![vec![false, true, true], vec![false, true, false], vec![false, true, false]],
			vec![vec![false, false, false], vec![true, true, true], vec![false, false, true]],
			vec![vec![false, true, false], vec![false, true, false], vec![true, true, false]]
		],
		BlockType::L =>  [
			vec![vec![false, false, true], vec![true, true, true], vec![false, false, false]],
			vec![vec![false, true, false], vec![false, true, false], vec![false, true, true]],
			vec![vec![false, false, false], vec![true, true, true], vec![true, false, false]],
			vec![vec![true, true, false], vec![false, true, false], vec![false, true, false]]
		],
		BlockType::O => [
			vec![vec![true, true], vec![true, true]],
			vec![vec![true, true], vec![true, true]],
			vec![vec![true, true], vec![true, true]],
			vec![vec![true, true], vec![true, true]]
		],
		BlockType::S => [
			vec![vec![false, true, true], vec![true, true, false], vec![false, false, false]],
			vec![vec![false, true, false], vec![false, true, true], vec![false, false, true]],
			vec![vec![false, false, false], vec![false, true, true], vec![true, true, false]],
			vec![vec![true, false, false], vec![true, true, false], vec![false, true, false]]
		],
		BlockType::T =>  [
			vec![vec![false, true, false], vec![true, true, true], vec![false, false, false]],
			vec![vec![false, true, false], vec![false, true, true], vec![false, true, false]],
			vec![vec![false, false, false], vec![true, true, true], vec![false, true, false]],
			vec![vec![false, true, false], vec![true, true, false], vec![false, true, false]]
		],
		BlockType::Z => [
			vec![vec![true, true, false], vec![false, true, true], vec![false, false, false]],
			vec![vec![false, false, true], vec![false, true, true], vec![false, true, false]],
			vec![vec![false, false, false], vec![true, true, false], vec![false, true, true]],
			vec![vec![false, true, false], vec![true, true, false], vec![true, false, false]]
		]
	};
	let x = (WIDTH - tiles[0].len()) / 2;
	Block {tiles, rotation : 0, x_pos : x as isize, y_pos : 0, name}
}

fn rotate(board : &[[bool; WIDTH]; HEIGHT], block : &mut Block, clockwise : bool) {
	block.rotation = if clockwise {(block.rotation + 1) % 4} else {(block.rotation + 3) % 4};
	let kicks = match block.name {
		BlockType::O => [(0, 0), (0, 0), (0, 0), (0, 0), (0, 0)],
		BlockType::I => match (clockwise, block.rotation) {
			(true, 1) | (false, 2) => [(0, 0), (-2, 0), (1, 0), (-2, -1), (1, 2)],
			(true, 2) | (false, 3) => [(0, 0), (-1, 0), (2, 0), (-1, 2), (2, -1)],
			(true, 3) | (false, 0) => [(0, 0), (2, 0), (-1, 0), (2, 1), (-1, -2)],
			(true, 0) | (false, 1) => [(0, 0), (1, 0), (-2, 0), (1, -2), (-2, 1)],
			_ => unreachable!()
		},
		_ => match (clockwise, block.rotation) {
			(true, 1) | (false, 1) => [(0, 0), (-1, 0), (-1, 1), (0, -2), (-1, -2)],
			(true, 2) | (false, 0)=> [(0, 0), (1, 0), (1, -1), (0, 2), (1, 2)],
			(true, 3) | (false, 3)=> [(0, 0), (1, 0), (1, 1), (0, -2), (1, -2)],
			(true, 0) | (false, 2) => [(0, 0), (-1, 0), (-1, -1), (0, 2), (-1, 2)],
			_ => unreachable!()
		}
	};
	let x = block.x_pos;
	let y = block.y_pos;
	for kick in kicks {
		block.x_pos = x + kick.0;
		block.y_pos = y + kick.1;
		if !overlapps(board, block) {
			return;
		}
	}
	block.x_pos = x;
	block.y_pos = y;
	block.rotation = if clockwise {(block.rotation + 3) % 4} else {(block.rotation + 1) % 4};
}

fn down(board : &mut [[bool; WIDTH]; HEIGHT], block : &mut Block) -> bool {
	block.y_pos += 1;
	if overlapps(board, block) {
		block.y_pos -= 1;
		freeze(board, block);
		return true;
	}
	return false;
}

fn overlapps(board : &[[bool; WIDTH]; HEIGHT], block : &Block) -> bool {
	if block.x_pos < 0 || block.y_pos < 0 || // NOT WORKING
		block.x_pos as usize + block.tiles[block.rotation][0].len() > WIDTH || 
		block.y_pos as usize + block.tiles[block.rotation].len() > HEIGHT
	{
		return true;
	}
	for y in 0..block.tiles[block.rotation].len() {
		for x in 0..block.tiles[block.rotation][y].len() {
			if block.tiles[block.rotation][y][x] && board[y + block.y_pos as usize][x + block.x_pos as usize] {
				return true;
			}
		}
	}
	false
}

fn freeze(board : &mut [[bool; WIDTH]; HEIGHT], block : &Block) {
	for y in 0..block.tiles[block.rotation].len() {
		for x in 0..block.tiles[block.rotation][y].len() {
			board[y + block.y_pos as usize][x + block.x_pos as usize] = block.tiles[block.rotation][y][x];
		}
	}
}


fn print_board(board : &[[bool; WIDTH]; HEIGHT], block : Option<&Block>) {
	//print!("{}", CLEAR_STRING);
	println!("################");
	for (y, row) in board.iter().enumerate() {
		for (x, c) in row.iter().enumerate() {
			if let Some(b) = block { 
				let y_pos = b.y_pos as usize;
				let x_pos = b.x_pos as usize;
				let tiles = &b.tiles[b.rotation];
				if y >= y_pos && y < y_pos + tiles.len() && x >= x_pos && x < x_pos + tiles[y - y_pos].len() {
					print!("{}", if tiles[y - y_pos][x - x_pos] {"▓▓"} else {"  "});
					continue;
				}
			}
			print!("{}", if *c {"██"} else {"  "});
		}
		println!("");
	}
	println!("################");
}


fn main() {
	let mut board : [[bool; WIDTH]; HEIGHT] = [[false; WIDTH]; HEIGHT];
	//freeze(&mut board, &create_block(BlockType::I));
	let mut block = create_block(BlockType::Z);
	print_board(&board, Some(&block));
	while !down(&mut board, &mut block) {
		print_board(&board, Some(&block));
	}
	print_board(&board, None);
}
