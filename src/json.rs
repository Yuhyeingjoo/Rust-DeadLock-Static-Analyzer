use std::fs::File;
use std::io::prelude::*;
extern crate serde;
use serde::{Serialize, Deserialize};
pub fn write_json(bug_list : Vec<(String, usize)>) {
	let mut file = File::create("bugs.json").unwrap();
	let sch = serde_json::to_string_pretty(&bug_list).unwrap();
	file.write_all(sch.as_bytes());
}
