use std::fs::File;
use std::io::prelude::*;
extern crate serde;

use serde::{Serialize, Deserialize};
use serde_json::json;

pub fn write_json(bug_list : Vec<(String, usize)>) {
	let mut file = File::create("bugs.json").unwrap();



    let mut bug_info = Vec::new();

    for file in &bug_list {
        println!("{} : {}", file.0, file.1);
        bug_info.push(json!(
                                {
                                    "file_name" : file.0,
                                    "line_number" : file.1
                                }));

    }

    let bug_infos = json!(
                            {
                                "bug_infos" : bug_info
                            });
    
	let sch = serde_json::to_string_pretty(&bug_infos).unwrap();
	
    file.write_all(&sch.into_bytes());
}
