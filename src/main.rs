use std::{io::{stdout, Write}, time::{Instant, Duration}};
use crossterm::{
	execute, Result, 
	terminal::{self, enable_raw_mode, ClearType, LeaveAlternateScreen, EnterAlternateScreen},
	cursor, 
	event::{poll, read, Event, KeyEvent, KeyCode},
	style::{self, Color}
};
use rand::{thread_rng, seq::SliceRandom};

const WIDTH : usize = 8;
const HEIGHT : usize = 22;

#[derive(Debug, Copy, Clone)]
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

enum KeyAction {
	None, Drop(bool), Move(bool), Exit 
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

fn rotate(board : &[[bool; WIDTH]; HEIGHT], block : &mut Block, clockwise : bool) -> bool {
	block.rotation = if clockwise {(block.rotation + 1) % 4} else {(block.rotation + 3) % 4};
	let kicks = match block.name {
		BlockType::O => [(0, 0), (0, 0), (0, 0), (0, 0), (0, 0)],
		BlockType::I => match (clockwise, block.rotation) {
			(true, 1) | (false, 2) => [(0, 0), (-2, 0), (1, 0), (-2, 1), (1, -2)],
			(true, 2) | (false, 3) => [(0, 0), (-1, 0), (2, 0), (-1, -2), (2, 1)],
			(true, 3) | (false, 0) => [(0, 0), (2, 0), (-1, 0), (2, -1), (-1, 2)],
			(true, 0) | (false, 1) => [(0, 0), (1, 0), (-2, 0), (1, 2), (-2, -1)],
			_ => unreachable!()
		},
		_ => match (clockwise, block.rotation) {
			(true, 1) | (false, 1) => [(0, 0), (-1, 0), (-1, -1), (0, 2), (-1, 2)],
			(true, 2) | (false, 0)=> [(0, 0), (1, 0), (1, 1), (0, -2), (1, -2)],
			(true, 3) | (false, 3)=> [(0, 0), (1, 0), (1, -1), (0, 2), (1, 2)],
			(true, 0) | (false, 2) => [(0, 0), (-1, 0), (-1, 1), (0, -2), (-1, -2)],
			_ => unreachable!()
		}
	};
	let x = block.x_pos;
	let y = block.y_pos;
	for kick in kicks {
		block.x_pos = x + kick.0;
		block.y_pos = y + kick.1;
		if !overlapps(board, block) {
			return true;
		}
	}
	
	block.x_pos = x;
	block.y_pos = y;
	block.rotation = if clockwise {(block.rotation + 3) % 4} else {(block.rotation + 1) % 4};
	return false;
}
fn move_block(board : &[[bool; WIDTH]; HEIGHT], block : &mut Block, dx : isize, dy : isize) -> bool {
	block.x_pos += dx;
	block.y_pos += dy;
	if overlapps(board, block) {
		block.x_pos -= dx;
		block.y_pos -= dy;
		return false;
	}
	return true;
}

fn in_bounds(block : &Block, x : usize, y : usize) -> bool {
	let x = block.x_pos + x as isize;
	let y = block.y_pos + y as isize;
	return x >= 0 && y >= 0 && (x as usize) < WIDTH && (y as usize) < HEIGHT;
}

fn overlapps(board : &[[bool; WIDTH]; HEIGHT], block : &Block) -> bool {
	for y in 0..block.tiles[block.rotation].len() {
		for x in 0..block.tiles[block.rotation][y].len() {
			if block.tiles[block.rotation][y][x] {
				if !in_bounds(block, x, y) || board[(y as isize + block.y_pos) as usize][(x as isize + block.x_pos) as usize] {
					return true;
				}
			}
		}
	}
	false
}

fn freeze(board : &mut [[bool; WIDTH]; HEIGHT], block : &Block) -> usize {
	for y in 0..block.tiles[block.rotation].len() {
		for x in 0..block.tiles[block.rotation][y].len() {
			if !in_bounds(&block, x, y) {
				continue;
			}
			if block.tiles[block.rotation][y][x] {
				board[(y as isize + block.y_pos) as usize][(x as isize + block.x_pos) as usize] = true;
			}
		}
	}
	let mut cleared_rows = 0;
	let mut index = board.len() - 1;
	while {
		if board[index].iter().all(|tile| *tile) {
			for i in (1..(index + 1)).rev() {
				board.copy_within((i - 1)..(i), i);
			}
			board[0].fill(false);
			cleared_rows += 1;
		} else {
			index -= 1;
		}
		index > 0
	}{}
	cleared_rows
}

fn add_score(rows : usize, level : &mut usize, score : &mut usize, cleared_rows : &mut usize, delay : &mut Duration) {
	if rows == 0 { 
		return; 
	}
	
	*score += *level * match rows {
		1 => 100,
		2 => 300,
		3 => 500,
		4 => 800,
		_ => unreachable!("Not possible to clear more than 4 rows")
	};
	*cleared_rows += rows;
	let target_rows = match *level {
		1 => 10,
		2 => 30,
		3 => 70,
		4 => 120,
		5 => 180,
		6 => 250,
		7 => 330,
		8 => 420,
		9 => 520,
		10 => 620,
		11 => 720,
		12 => 820,
		13 => 920,
		14 => 1020,
		15 => 1120,
		16 => 1230,
		17 => 1350,
		18 => 1480,
		19 => 1620,
		20 => 1770,
		21 => 1930,
		22 => 2100,
		23 => 2280,
		24 => 2470,
		25 => 2670,
		26 => 2870,
		27 => 3070,
		28 => 3270,
		29 => usize::MAX,
		_ => unreachable!("Invalid level")
	};
	if *cleared_rows >= target_rows {
		*level += 1;
		*delay = Duration::from_millis(827 - 27 * (*level as u64));
	}
}

fn print_board(board : &[[bool; WIDTH]; HEIGHT], block : &Option<Block>) -> Result<()>{
	execute!(stdout(), cursor::MoveTo(0, 0))?;
	println!("##################");
	for (y, row) in board.iter().enumerate() {
		print!("#");
		for (x, c) in row.iter().enumerate() {
			if let Some(b) = block {
				let xi = x as isize;
				let yi = y as isize;
				let tiles = &b.tiles[b.rotation];
				if 
					yi >= b.y_pos && yi < b.y_pos + tiles.len() as isize &&
					xi >= b.x_pos && xi < b.x_pos + tiles[0].len() as isize &&
					tiles[(yi - b.y_pos) as usize][(xi - b.x_pos) as usize]
				{
					print!("▓▓");
					continue;
				}
			}
			print!("{}", if *c {"██"} else {"  "});
		}
		println!("#");
	}
	println!("##################");
	Ok(())
}

fn print_ui(block : &Block, score : usize, rows : usize, level : usize, delay : Duration) -> Result<()> {
	execute!(stdout(), cursor::MoveTo(20, 0))?;
	print!("NEXT:");
	execute!(stdout(), cursor::MoveTo(18, 2))?;
	for _ in 0..5{
		print!("                 ");
		execute!(stdout(), cursor::MoveToColumn(18), cursor::MoveDown(1))?;
	}
	execute!(stdout(), cursor::MoveTo(18, 2))?;
	for row in &block.tiles[block.rotation] {
		print!("  ");
		for col in row {
			print!("{}", if *col {"▓▓"} else {"  "}); 
		}
		execute!(stdout(), cursor::MoveToColumn(18), cursor::MoveDown(1))?;
	}
	execute!(stdout(), cursor::MoveTo(19, 8))?;
	print!("Score: {}", score);
	execute!(stdout(), cursor::MoveTo(19, 9))?;
	print!("Level: {}", level);
	execute!(stdout(), cursor::MoveTo(19, 10))?;
	print!("Lines: {}", rows);
	execute!(stdout(), cursor::MoveTo(19, 11))?;
	print!("Delay: {:?}", delay);
	Ok(())
}

fn print_game_over(score : usize, level : usize) -> Result<bool> {
	execute!(stdout(), terminal::Clear(ClearType::All), cursor::MoveTo(10, 3))?;
	print!("Game Over");
	execute!(stdout(), cursor::MoveTo(10, 5))?;
	print!("Score : {}", score);
	execute!(stdout(), cursor::MoveTo(10, 7))?;
	print!("Level : {}", level);
	execute!(stdout(), cursor::MoveTo(6, 9))?;
	print!("Press R to play again");
	execute!(stdout(), cursor::MoveTo(7, 11))?;
	print!("Press esc to exit");
	stdout().flush()?;
	let res = loop {
		match read() {
			Ok(Event::Key(KeyEvent {
				code : KeyCode::Esc, ..
			})) => break Ok(false),
			Ok(Event::Key(KeyEvent {
				code : KeyCode::Char('r'), ..
			})) => break Ok(true),
			_ => ()
		}
	};
	execute!(stdout(), terminal::Clear(ClearType::All))?;
	res
}

fn handle_key(board : &[[bool; WIDTH]; HEIGHT], block : &mut Option<Block>) -> KeyAction {
	if let Ok(true) = poll(Duration::from_millis(1)) {
		let event = read();
		match event {
			Ok(Event::Key(KeyEvent {
				code : keycode, ..
			})) => match (keycode, block.as_mut()) {
					(KeyCode::Esc, _) => KeyAction::Exit,
					(KeyCode::Up, Some(block)) => KeyAction::Move(rotate(&board, block, true)),
					(KeyCode::Char('z'), Some(block)) => KeyAction::Move(rotate(&board, block, false)),
					(KeyCode::Down, _) => KeyAction::Drop(false),
					(KeyCode::Left, Some(block)) => KeyAction::Move(move_block(&board, block, -1, 0)),
					(KeyCode::Right, Some(block)) => KeyAction::Move(move_block(&board, block, 1, 0)),
					(KeyCode::Char(' '), Some(b)) => {
						while move_block(board, b, 0, 1) {}
						KeyAction::Drop(true)
					},
					_ => KeyAction::None
			},
			_ => KeyAction::None
		}
	} else {
		KeyAction::None
	}
}


fn main() -> Result<()> {
	let mut rng = thread_rng();
	enable_raw_mode()?;
	execute!(stdout(), 
		EnterAlternateScreen,
		cursor::Hide,
		terminal::Clear(ClearType::All)
	)?;
	let mut bag = [BlockType::I, BlockType::J, BlockType::L, BlockType::O, BlockType::S, BlockType::T, BlockType::Z];
	let mut bag_index = 1;
	bag.shuffle(&mut rng);

	let mut board : [[bool; WIDTH]; HEIGHT] = [[false; WIDTH]; HEIGHT];
	
	let mut block = Some(create_block(bag[0]));
	let mut next_block = create_block(bag[1]);

	let mut delay = Duration::from_millis(800);
	let lock_delay = Duration::from_millis(500);
	let soft_drop_delay = Duration::from_millis(100);
	let mut active_delay = delay;

	let mut time;

	let mut running = true;
	let mut soft_drop = false;
	let mut soft_drop_instant = Instant::now();
	let soft_drop_duration = Duration::from_millis(100);
	
	let mut score = 0;
	let mut line_clears = 0;
	let mut level = 1;

	let mut lock_actions = None;
	print_ui(&next_block, score, line_clears, level, delay)?;
	while running {
		print_board(&board, &block)?;
		time = Instant::now();
		if soft_drop && soft_drop_instant.elapsed() >= soft_drop_duration { // This does not work very well..
			soft_drop = false;
			if let Some(_) = lock_actions {
				active_delay = lock_delay;
			} else {
				active_delay = delay;
			}
		}
		while time.elapsed() < active_delay {
			match handle_key(&board, &mut block) {
				KeyAction::Exit => {
					running = false;
					break;
				},
				KeyAction::Move(true) => {
					if let Some(actions_left) = lock_actions{
						if actions_left > 0 {
							lock_actions = Some(actions_left - 1);
							time = Instant::now();
						}
					}
					print_board(&board, &block)?;
				},
				KeyAction::Drop(false) => {
					active_delay = soft_drop_delay;
					soft_drop_instant = Instant::now();
					soft_drop = true;
				},
				KeyAction::Drop(true) => {
					lock_actions = None;
					let rows = freeze(&mut board, &block.unwrap());
					add_score(rows, &mut level, &mut score, &mut line_clears, &mut delay);
					block = None;
					break;
				}
				_ => ()
			};
		}
		if let Some(_) = lock_actions {
			if !move_block(&board, block.as_mut().unwrap(), 0, 1) {
				let rows = freeze(&mut board, &block.unwrap());
				add_score(rows, &mut level, &mut score, &mut line_clears, &mut delay);
				block = None;
			}
			if !soft_drop {
				active_delay = delay;
			}
			lock_actions = None;
		} else if let Some(b) = block.as_mut() {
			if !move_block(&board, b, 0, 1) { // Maybe move back?
				lock_actions = Some(15);
				if !soft_drop {
					active_delay = lock_delay;
				}
			}
		} else {
			bag_index += 1;
			if bag_index == bag.len() {
				bag_index = 0;
				bag.shuffle(&mut rng);
			}
			if overlapps(&board, &next_block) {
				if let Ok(true) = print_game_over(score, level) {
					bag.shuffle(&mut rng);
					next_block = create_block(bag[0]);
					block = Some(create_block(bag[1]));
					bag_index = 1;
					score = 0;
					level = 1;
					line_clears = 0;
					delay = Duration::from_millis(800);
					soft_drop = false;
					active_delay = delay;
					board = [[false; WIDTH]; HEIGHT];
					print_ui(&next_block, score, line_clears, level, delay)?;
					continue;
				} else {
					break;
				}
			}
			block = Some(next_block);
			next_block = create_block(bag[bag_index]);
			print_ui(&next_block, score, line_clears, level, delay)?;
		}	
	}
	execute!(stdout(), LeaveAlternateScreen, cursor::Show)?;
	Ok(())
}
