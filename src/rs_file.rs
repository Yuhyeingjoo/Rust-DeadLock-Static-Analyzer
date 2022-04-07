use std::sync::mpsc::{Sender, channel};
use std::path::Path;
use std::fs::read_to_string;
use walkdir::{DirEntry, WalkDir};
use tree_sitter::Parser;
use tree_sitter::{Tree,Node};
use tree_sitter_traversal::{traverse, Order};


pub struct FileVector{
    file_vec: Vec<File>,
	sender: Sender<(i32, String, String, String)>,
}

#[derive(Debug)]
struct File{
    path : String,
    code : String,
    ast : Tree,
    lib_name : LibType,
    item_list : Vec<ItemType>
}

#[derive(Debug)]
enum ItemType{
    ModFunc(Vec<ItemType>, String),
    ImplFunc(Vec<ItemType>, String),
    Func(String),
    None,
}
#[derive(Debug,Clone)]
enum LibType{
    Name (String),
    Main (String),
    MT,
}


impl ItemType {
    fn new_vec(node: &Node<'_>, code: &String) -> Vec<ItemType>{
        
        let mut _item_list :Vec<ItemType> = Vec::new();
		let preorder: Vec<Node<'_>> = node.children(&mut node.walk()).collect::<Vec<_>>();
        for element in &preorder {
            if element.kind().eq("function_item"){
                let item_type = ItemType::new(element, code);
                _item_list.push(item_type);
            }
			if element.kind().eq("impl_item") {
				let name = match element.child(1) {
					Some(ch) => {
						code[ch.start_byte()..ch.end_byte()].to_string()
					}
					None => { panic!("impl item error") }
				};
				_item_list.push(ItemType::ImplFunc(ItemType::new_vec(&element.child(2).unwrap(), &code), name));
			}
        }
        _item_list
    }
    fn new(node : &Node<'_>, code : &String) ->ItemType {
                let mut func_name = String::from("");
                match  node.child_by_field_name("name"){
                        Some(ch) =>{
                                func_name = code[ch.start_byte()..ch.end_byte()].to_string();
                        },
                        None=>{}
                }
        ItemType::
             Func(func_name)

    }
}

impl File {
    fn new(dir: &DirEntry) -> File {
        let path = Path::new(dir.path());
        let code = read_to_string(path).unwrap();
        let mut parser = Parser::new();
        parser.set_language(tree_sitter_rust::language()).expect("Error loading Rust grammar");
        let parsed = parser.parse(&code, None);
        let ast = parsed.unwrap();

        let str_path = path.to_str().unwrap();
		let mut _item_list;
		let root_node = ast.root_node();
		 
		_item_list  = ItemType::new_vec(&root_node , &code);
        
        Self{
            path : str_path.to_string(),
            code,
            ast : ast,
            lib_name : LibType::MT,
            item_list : _item_list,
        }
    }
}
impl FileVector {
    pub fn new(sender: Sender<(i32, String, String, String)>) ->Self {
        let file_vec : Vec<File> = Vec::new();

        Self{
            file_vec,
			sender,
        }
    }
    pub fn show(&self){
        for element in &self.file_vec {
            print!("{:?}",element.path);
            println!("{:?}",element.lib_name);
        }
    }

    pub fn traverse_dir(&mut self, input_dir : String, toml : String){
        for entry in WalkDir::new(input_dir)
                    .into_iter()
                    .filter_entry(|e| is_dir_or_rs(e))
        {
            let dir = match entry{
                Ok(_dir) =>{
                    _dir
                },
                _ =>{panic!("directory errpr");},
            };

            if is_rust(&dir){
                self.file_vec.push(File::new(&dir));
            }
        }
        self.read_toml(toml);
        self.find_main();
    }
    fn find_main(&mut self){
		let mut index: Option<usize> = None;
		for (i, element) in self.file_vec.iter().enumerate() {
			match element.lib_name {
				LibType::Main(_) => {
					index = Some(i);
					break;
				}
				_ => {}
			}
		}

		if let None = index {
			for (i, element) in self.file_vec.iter().enumerate() {
				if (element.path.contains("main.rs")) {
					index = Some(i);
					break;
				}
			}
		}

		let main = self.file_vec.remove(index.unwrap());
		self.file_vec.insert(0, main);
    }
    fn read_toml(&mut self, toml_dir: String) {
        let toml = read_to_string(&toml_dir).unwrap();
        let mut name_flag = 0;
        let mut path_flag = 0;
        let mut _name = String::new();
        let mut _path  = String::new();
        for line in toml.lines(){
            if line.eq("[[bin}}"){
                name_flag = 3;
                path_flag = 3;
            }
            if line.eq("[lib]"){
                name_flag = 1;
                path_flag = 1;
            }
            if  line.starts_with("name"){
                if name_flag ==1 {
                    name_flag  = 2;
                }
                else {
                    name_flag = 4;
                }
                    let name_split_vec: Vec<&str> =line.split("\"").collect();
                    _name = name_split_vec[1].to_string();
            }
            else if line.starts_with("path"){
                if path_flag ==1{
                    path_flag = 2;
                }
                else {
                    path_flag = 4;
                }
                    _path = String::from("");
                    match toml_dir.rsplit_once("Cargo.toml"){
                        Some((base_dir,_ )) =>{
                            let end_dir: Vec<&str> =line.split("\"").collect();
                            base_dir.to_string().push_str(end_dir[1]);
                            _path.push_str(base_dir);
                            _path.push_str(end_dir[1]);
                        },
                        _=> {},
                    };
            }

            if path_flag==2 && name_flag ==2{
                for  element in &mut *  self.file_vec {
                    if element.path.eq(&_path){
                        element.lib_name = LibType::Name(_name);
                        _path= String::from("");
                        _name = String::from("");
                    }
                }
                path_flag = 0;
                name_flag = 0;
            }
            if path_flag==4 && name_flag == 4{
                for element in &mut *  self.file_vec {
                    if element.path.eq(&_path){
                        println!("mainr: {} {}", element.path, _name); 
                        element.lib_name = LibType::Main(_path);
                        _path= String::from("");
                        _name = String::from("");
                    }
                }
                path_flag = 0;
                name_flag = 0;
            }
            
        }
    }
}

fn is_dir_or_rs(entry: &DirEntry) -> bool {
        return (entry.file_name()
             .to_str()
              .map(|s| s.ends_with(".rs"))
               .unwrap_or(false) || entry.file_type().is_dir())
            && !entry.file_name()
             .to_str()
              .map(|s| s.starts_with("."))
               .unwrap_or(false)
}

fn is_rust(entry:&DirEntry) -> bool{
    return entry.file_name().to_str().unwrap().ends_with(".rs")
}
