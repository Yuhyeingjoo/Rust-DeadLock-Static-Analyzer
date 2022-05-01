#[derive(Debug)]
pub struct symbolTable {
    pub symbolVec: Vec<(String, i32,String)>,
}
static mut id : i32 = 0;
impl symbolTable {
    pub fn new ()-> Self {
        let  mtVector :Vec<(String,  i32, String)> = Vec::new();
        Self{
            symbolVec : mtVector,
        }
    }

    pub fn append(&mut self,  name : String, _type : String ){
        for element in &mut self.symbolVec {
           if element.0.eq(&name.as_str()){
                element.0 = name.clone();
                unsafe {
                    id +=1;
                    element.1 = id;
                }
                element.2 = _type.clone();
                break;
           }
        }
        let aVariable : (String, i32, String);
        unsafe{
            id +=1;
            aVariable=(name, id, _type);
        }

        self.symbolVec.push(aVariable);
    }
    pub fn get<'a>(&'a self, name : &'a str) -> (&'a str, i32,  &'a str) {
		//println!("CURRENT STATE OF ST: {:?}", self);
       for element in &self.symbolVec {
           if element.0.eq(name){
			   //println!("COMPARE PARAM : {} and EL : {}", name, element.0);
                return (&element.0.as_str(), element.1, &element.2.as_str());
                    
           }
       } 
       ("",-1,"")
    }
    pub fn appendArc(&mut self, name : String, target : String) {
        for element in &self.symbolVec{
            if element.0.eq(&target.as_str()){
                unsafe{
                    self.symbolVec.push((name, element.1, element.2.clone()));
                }
                break;
            }
        }
    }
}
   

/*
fn main(){
    let mut ve = symbolTable::new();
    ve.append("t1".to_string(), "Chan".to_string());
    ve.append("t2".to_string(), "Chan".to_string());
    ve.appendArc("t3".to_string(), "t1".to_string());


    println!("{:?}",ve);
    let t1 = "t3";
    println!("get {:?}", ve.get(t1));

}
*/
