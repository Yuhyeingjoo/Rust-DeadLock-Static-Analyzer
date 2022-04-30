use std::sync::mpsc::{Sender};
use std::path::Path;
use std::fs::read_to_string;
use walkdir::{DirEntry, WalkDir};
use tree_sitter::Parser;
use tree_sitter::{Tree,Node};
use tree_sitter_traversal::{traverse, Order};

#[path = "symbol_table.rs"] mod symbol_table;

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
    Func(String, String),
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
                let ret_node = node.child_by_field_name("return_type");
                let mut func_return = String::from("");
                match ret_node {
                    Some(ch) => {
                        func_return = code[ch.start_byte()..ch.end_byte()].to_string();
                    },
                    None =>{}
                }    
                if func_return.eq("Self") {
                    let impl_node = node.parent().unwrap().parent().unwrap().child(1).unwrap();
                    func_return =  code[impl_node.start_byte()..impl_node.end_byte()].to_string();
                }
                let argu_node = node.child_by_field_name("parameters");
                let mut func_argu = String::from("");
    
				//argument 처리하는 로직인것같은데 이걸 어디에 저장하는건지??
				match argu_node {
                    Some(ch) => {
                        let para_name = node.child_by_field_name("parameter");
                        match para_name {
                            Some(par) =>{
                                func_argu = code[par.start_byte()..par.end_byte()].to_string();
                                //println!("zzzzzzzzzzzzz{}", func_argu);
                            },
                            None =>{}
                        }
                    },
                    None => {
                        //println!("zzzzzzzzzzzzzzz");
                    }
                }
                
                let mut func_name = String::from("");
                match  node.child_by_field_name("name"){
                        Some(ch) =>{
                                func_name = code[ch.start_byte()..ch.end_byte()].to_string();
                        },
                        None=>{}
                }

            
		/*
		if parent.ne("") {	
			func_name = format!("{}::{}",parent,func_name);
			if func_return.ne("") {
				func_return = format!("{}::{}", parent, func_return);
			}
		}
		*/
        
        ItemType::
             Func(func_name,func_return)

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
				self.traverse_block(&block, &code, 0, "0".to_string(), String::from(""), Vec::new());
			}
			_ => {
				panic!("Couldn't find main block");
			}
		}
	}

	fn traverse_block(&self, node: &tree_sitter::Node, code: &str, tid: i32, block: String, upper_idtf: String, arguments :Vec<(String, i32, String)>) {
        let mut symbol_table = symbol_table::symbolTable::new();

		/*
		//여기서 append 사용하지 않은 이유가 있는지?
		//id는 symbol table 내부에서 관리하는걸로 알고있는데 직접 넣어주는 이유는?
		for i in 0..arguments.len() {
			symbol_table.symbolVec.push(arguments[i].clone());
        }
		*/
		for arg in arguments {
			let (name, id, _type) = arg;
			symbol_table.append(name, _type);
		}

		let mut limit = 0;
		let mut preorder: Vec<Node<'_>> = traverse(node.walk(), Order::Pre).collect::<Vec<_>>();
		for x in &preorder {
			let kind = x.kind();
			if kind.eq("call_expression") {

				let call_node = x.child(0).unwrap();
				let mut key = &code[call_node.start_byte()..call_node.end_byte()];
				let idtf = key.clone();

				println!("CALL EXPR : {}", key);

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
					self.traverse_block(&t_block, &code, new_tid, format!("{}-{}",block.clone(), new_bc), upper_idtf.clone(), symbol_table.symbolVec.clone()); 
				}
		
				if call_node.end_byte() < limit {
					continue;
				}

                //println!("{}",key);
				if key.contains(".") {
					let split: Vec<&str> = key.split(".").collect();
                    //let  mut _type   = String::from("");
                    //let mut symbol_id  = 0;

					/* symbolVec 이 private이라 접근 안됨
                    for element in &symbol_table.symbolVec{
                        if element.0.eq(split[0]){
                            _type = element.2.clone(); 
                            symbol_id  = element.1;
                        }
					}
					*/
					println!("IN ST : {:?}", symbol_table);
					println!("******get -> {:?}", split[0]);
					let (_, symbol_id, _type) = symbol_table.get(split[0]);

					let type_split: Vec<&str> = _type.split("::").collect();
                    println!("type split {:?} {:?}", _type, split);
                    for element in & * self.file_vec{
                        if let LibType::Name(extern_name) = &element.lib_name{
                            if type_split[0].to_string().eq(extern_name.as_str()) {
                                let mut arg : Vec<(String,i32,String)>  = Vec::new();
                                arg.push(("self".to_string(), symbol_id , _type.to_string()));
                                println!("method {:?} {:?}", key, arg);
                                self.search(&element, &element.item_list, &key[type_split[0].len()+1 ..] , tid, block.clone(), upper_idtf.clone(), arg);
                            }
                        }
                    }

				    //key = split.last().unwrap();
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
                if  key.contains("::"){

					let split: Vec<&str> = key.split("::").collect();
                    let lib_name = split[0];
                    let mut key  = String::from("");
                    for i in 1 .. split.len() {
                        key.push_str(split[i]);
                    }
                    
                    for element in & * self.file_vec { 
                        if let LibType::Name(extern_name) = &element.lib_name{
                            if lib_name.to_string().eq(extern_name.as_str()) {
                                let mut key  = String::from("");
                                for i in 1 .. split.len() {
                                    key.push_str(split[i]);
                                    if i != split.len()-1{
                                        key.push_str("::");
                                    }
                                }
                                self.search(&element, &element.item_list, &key , tid, block.clone(), idtf.clone(), Vec::new());
                            }
                            
                        }
                    }
                }
                
			}
			else if kind.eq("let_declaration") {
				println!("LET DCLR : {}",  &code[x.start_byte()..x.end_byte()]);
				self.store_symbol(&x, &code, &mut symbol_table);
				println!("AFTER UPDATING SYMBOL TABLE : {:?}", symbol_table);
			}

		}
	}
	fn search(&self, file: &File, list: &Vec<ItemType>, key: &str, tid: i32, block_id: String, idtf: String, arguments : Vec<(String,i32,String)>) {
        if key.contains("::") {
			let split: Vec<&str> = key.split("::").collect();
            for item in list {
                match item {
                    ItemType::ImplFunc(new_list, name) => {
                        if split[0].eq(name){
                            let mut key  = String::from("");
                            for i in 1 .. split.len() {
                                key.push_str(split[i]);
                                if i !=split.len()-1{
                                    key.push_str("::");
                                }
                            }
			                self.search(&file, &new_list, &key, tid, block_id.clone(), idtf.clone(), Vec::new());	
                        }
                    },
                    _ => {},
                }
            }


        }
        else {
            for item in list {
                match item {
                    ItemType::Func(name, ret) => {
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
                                    self.traverse_block(&block, &file.code, tid, format!("{}-{}", block_id.clone(), bc), idtf.clone(), Vec::new());
                                }
                                _ => {
                                    //println!("KEY -> {}",block_key);
                                    panic!("Couldn't find target block");
                                }
                            }	
                        }
                    }
                    ItemType::ImplFunc(new_list, name) => { //ImplFun(new_list, Counter)   key : Counter::new()  
                        self.search(&file, &new_list, &key, tid, block_id.clone(), idtf.clone(), Vec::new());	
                    }
                    _ => {}
                }
            }
        }
	}
	fn store_symbol (&self, node: &tree_sitter::Node, code: &str, symbol_table: &mut symbol_table::symbolTable) {
		let var = node.child(1).unwrap();
		let mut value = node.child_by_field_name("value").unwrap();

		let mut var_str = &code[var.start_byte()..var.end_byte()];
		let mut value_str = &code[value.start_byte()..value.end_byte()];

		if value_str.starts_with("Arc::new(") || value_str.starts_with("Arc::clone(") {
			match value.child_by_field_name("arguments").unwrap().child(1) {
				Some(argu) => {
					if argu.kind().eq("call_expression") {
						value = argu.child(0).unwrap();
					}
					else {
						value = argu;
					}
				}
				None => {
					println!("Should not reach here");
				}
			}
			value_str = &code[value.start_byte()..value.end_byte()];
			if value_str.starts_with("&") {
				value_str = &value_str[1..];
				//symbol_table.appendArc(value_str.to_string(), var_str.to_string());
				symbol_table.appendArc(var_str.to_string(), value_str.to_string());
				return;
			}
		} 
		else {
			if value_str.ends_with("()") {
				value_str = &value_str[0..value_str.len() -2];
			}
		}
		//println!("symbol name : {} , symbol type key : {}", var_str, value_str);

		let mut symbol_type = String::from("");
		if value_str.contains("::"){
			let split: Vec<&str> = value_str.split("::").collect();
			let lib_name = split[0];
			for element in & * self.file_vec { 
				if let LibType::Name(extern_name) = &element.lib_name{
					if lib_name.to_string().eq(extern_name.as_str()) {
						symbol_type = self.find_return_type(&element, &element.item_list, &value_str[lib_name.len()+2 ..]);
					}
				}
			}
		}
		//println!("return type : {}", symbol_type);
		//println!("");

		symbol_table.append(var_str.to_string(), symbol_type);
		//println!("{:?}", symbol_table);
	}

	fn find_return_type(&self, file: &File, list: &Vec<ItemType>, key: &str) -> String {
		let mut return_value = String::from("");

		for item in list {
			match item {
				ItemType::Func(name, ret) => {
					//println!("key : {} vs name : {}", key, name);
					if key.eq(name) {
						return_value = ret.to_string();
					}
				}
				ItemType::ImplFunc(new_list, name) => {
					let re = self.find_return_type(&file, &new_list, &key);
					if re.ne("") {
						return_value = re;
					}
				}
				_ => {
				}
			}
		}
		return_value
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

