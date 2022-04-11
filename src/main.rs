use std::io;
use std::sync::mpsc::channel;
use std::thread;


mod rs_file;
mod dd;

fn main() {
	let (sender, receiver) = channel();

    let mut file_vec : rs_file::FileVector = rs_file::FileVector::new(sender);
    let mut input_dir = String::new();
    let mut input_toml = String::new();
    io::stdin().read_line(&mut input_dir).expect("Failed to read line");
    input_dir = input_dir.trim().to_string();
    io::stdin().read_line(&mut input_toml).expect("Failed to read line");
    input_toml = input_toml.trim().to_string();
	let handle =thread::spawn(move || {
		let mut graph = dd::GraphMaker::new(receiver);
		graph.run();	
	});	


    file_vec.traverse_dir(input_dir, input_toml);

	file_vec.show();
	file_vec.start();

	handle.join();
}

