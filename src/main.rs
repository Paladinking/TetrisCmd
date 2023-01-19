mod tetris;

fn main() {
	match tetris::start() {
		Ok(_) => (),
		Err(e) => println!("{:?}", e)
	}
}