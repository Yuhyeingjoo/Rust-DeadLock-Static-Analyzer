use std::io;
use std::sync::mpsc::channel;
use std::thread;
use std::env;

mod rs_file;
mod dd;

fn main() {
    let args: Vec<String> = env::args().collect();
	let (sender, receiver) = channel();

    let mut file_vec : rs_file::FileVector = rs_file::FileVector::new(sender);
    let mut input_dir = args[1].clone();
    let mut input_toml = args[2].clone();
	let handle =thread::spawn(move || {
		let mut graph = dd::GraphMaker::new(receiver);
		graph.run();	
	});	


    file_vec.traverse_dir(input_dir, input_toml);

	//file_vec.show();
	file_vec.start();

	handle.join();
}

