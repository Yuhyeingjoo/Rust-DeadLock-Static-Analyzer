use std::io;
use std::sync::mpsc::channel;
use std::thread;

extern crate RsFile;

fn main() {
	let (sender, receiver) = channel();

    let mut file_vec : RsFile::FileVector = RsFile::FileVector::new(sender);
    let mut input_dir = String::new();
    let mut input_toml = String::new();
    io::stdin().read_line(&mut input_dir).expect("Failed to read line");
    input_dir = input_dir.trim().to_string();
    io::stdin().read_line(&mut input_toml).expect("Failed to read line");
    input_toml = input_toml.trim().to_string();

	thread::spawn(move || {

	});


    file_vec.traverse_dir(input_dir, input_toml);
    file_vec.show()
}

