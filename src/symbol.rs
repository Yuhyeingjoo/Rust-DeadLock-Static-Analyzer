struct symbolTable {
    symbolVec: Vec<(String, i32,String)>,
}
static mut id : i32 = 0;
impl symbolTable {
    fn new ()-> Self {
        let mut mtVector :Vec<(String,  i32, String)> = Vec::new();
        Self{
            symbolVec : mtVector,
        }
    }

    fn append(&mut self,  name : String, _type : String ){
        for element in self {
           if element.0.eq(name.to_str()){
                element.0 = name;
                unsafe {
                    id +=1;
                    element.1 = id;
                }
                element.2 = _type;
                break;
           } 
        }
       self. 
    }
}
