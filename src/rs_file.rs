use std::sync::mpsc::{Sender};
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

static mut block_count: i32 = 0;
static mut tid_count: i32 = 0;

impl ItemType {
    fn new_vec(node: &Node<'_>, code: &String, parent: String) -> Vec<ItemType>{
        
        let mut _item_list :Vec<ItemType> = Vec::new();
		let preorder: Vec<Node<'_>> = node.children(&mut node.walk()).collect::<Vec<_>>();
        for element in &preorder {
            if element.kind().eq("function_item"){
                let item_type = ItemType::new(element, code, parent.clone());
                _item_list.push(item_type);
            }
			if element.kind().eq("impl_item") {
				let name = match element.child(1) {
					Some(ch) => {
						code[ch.start_byte()..ch.end_byte()].to_string()
					}
					None => { panic!("impl item error") }
				};
				_item_list.push(ItemType::ImplFunc(ItemType::new_vec(&element.child(2).unwrap(), &code, name.clone()), name));
			}
        }
        _item_list
    }
    fn new(node : &Node<'_>, code : &String, parent: String) ->ItemType {
                let mut func_name = String::from("");
                match  node.child_by_field_name("name"){
                        Some(ch) =>{
                                func_name = code[ch.start_byte()..ch.end_byte()].to_string();
                        },
                        None=>{}
                }

		if parent.ne("") {
			//
			func_name = format!("{}::{}",parent,func_name);
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
		 
		_item_list  = ItemType::new_vec(&root_node , &code, "".to_string());
        
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
            println!("path : {:?}",element.path);
			println!("libname : {:?}",element.lib_name);
            println!("item : {:?}",element.item_list);		
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
				if element.path.contains("main.rs") {
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
						let _lib_name = _name.clone();

                        element.lib_name = LibType::Name(_name);
                        _path= String::from("");
                        _name = String::from("");
						
						//	
						match element.item_list.get_mut(0).unwrap() {
							ItemType::ImplFunc(vec, s) => {
								for func in &mut * vec {
									let mut new_name = String::from("");
									match func {
										ItemType::Func(name) => {
											new_name = format!("{}::{}", _lib_name, name);
										},
										_ => {},
									}
									*func = ItemType::Func(new_name);
								}								
							},
							_ => {},
						}
						
                    }
                }
                path_flag = 0;
                name_flag = 0;
            }
            if path_flag==4 && name_flag == 4{
                for element in &mut *  self.file_vec {
                    if element.path.eq(&_path){
                        println!("main: {} {}", element.path, _name); 
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

	pub fn find_block<'a>(&self, tree: &'a tree_sitter::Tree, target: &str, code: &str) -> Option<tree_sitter::Node<'a>> {		
		let root = tree.root_node();
		let preorder: Vec<Node<'_>> = traverse(root.walk(), Order::Pre).collect::<Vec<_>>();

		let mut target_block = Option::None;

		for x in &preorder {
			let line = &code[x.start_byte()..x.end_byte()];
			if line.eq(target) && x.kind().eq("identifier") && x.parent().unwrap().kind().eq("function_item"){
				target_block = x.child_by_field_name("body");

				match target_block {
					Some(_) => {
						break;
					}
					None => {
						let parent = x.parent().unwrap();
						target_block = parent.child_by_field_name("body");
					}
				}
			}
		}
		target_block
	}

	pub fn start(&self) {
		
		let main_tree = &(self.file_vec.get(0).unwrap().ast);
		let code = &(self.file_vec.get(0).unwrap().code);
	
		match self.find_block(&main_tree, "main", &code) {
			Some(block) => {
				self.traverse_block(&block, &code, 0, "0".to_string(), String::from(""));
			}
			_ => {
				panic!("Couldn't find main block");
			}
		}
	}

	fn traverse_block(&self, node: &tree_sitter::Node, code: &str, tid: i32, block: String, upper_idtf: String) {
		let mut limit = 0;
		let mut preorder: Vec<Node<'_>> = traverse(node.walk(), Order::Pre).collect::<Vec<_>>();
		for x in &preorder {
			let kind = x.kind();
			if kind.eq("call_expression") {
				let call_node = x.child(0).unwrap();
				let mut key = &code[call_node.start_byte()..call_node.end_byte()];
				let idtf = key.clone();

				if key.eq("thread::spawn") {
					let t_block = x.child(1).unwrap();
					limit = t_block.end_byte();
					
					let mut new_tid = 0;
					let mut new_bc = 0;
					unsafe {
						tid_count = tid_count + 1;
						new_tid = tid_count;
					}	
					unsafe {
						block_count = block_count + 1;
						new_bc = block_count;
					}
					self.traverse_block(&t_block, &code, new_tid, format!("{}-{}",block.clone(), new_bc), upper_idtf.clone()); 
				}
		
				if call_node.end_byte() < limit {
					continue;
				}


				println!("{}", key);
				if key.contains(".") {
					let split: Vec<&str> = key.split(".").collect();
					key = split.last().unwrap();
				}
				let mut idtf = str::replace(idtf, &format!(".{}",key), "");

				if idtf.contains("self") {
					idtf = idtf.replace("self", &upper_idtf);
				}

				if key.eq("lock") {
					println!("lock");
					println!("tid : {} {}.{} {}",tid, idtf,key, block);
					self.sender.send((tid, idtf.to_string(), block.clone(), key.to_string()));
				}

				for element in & * self.file_vec {
					self.search(&element, &element.item_list, key, tid, block.clone(), idtf.clone());
				}
			}
			else if kind.eq("let_declaration") {
				let mut cursor = x.walk();
				for child in x.children(&mut cursor) {
					if child.kind().eq("call_expression") {
						//println!("child : {:?}", &code[child.start_byte()..child.end_byte()]);
					}
				}
			}

		}
	}
	fn search(&self, file: &File, list: &Vec<ItemType>, key: &str, tid: i32, block_id: String, idtf: String) {
		for item in list {
			match item {
				ItemType::Func(name) => {
					if key.eq(name) {
						//println!("matched name : {}", name);
						let mut block_key = key.clone();
						if key.contains("::") {
							let split: Vec<&str> = key.split("::").collect();
							block_key = split.last().unwrap();
						}		
						//println!("search -> {} ", block_key);
						match self.find_block(&file.ast, block_key, &file.code) {
							Some(block) => {
								let mut bc = -1;
								unsafe {
									block_count = block_count + 1;
									bc = block_count;
								}
								self.traverse_block(&block, &file.code, tid, format!("{}-{}", block_id.clone(), bc), idtf.clone());
							}
							_ => {
								//println!("KEY -> {}",block_key);
								panic!("Couldn't find target block");
							}
						}	
					}
				}
				ItemType::ImplFunc(new_list, name) => {
					self.search(&file, &new_list, &key, tid, block_id.clone(), idtf.clone());	
				}
				_ => {}
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

