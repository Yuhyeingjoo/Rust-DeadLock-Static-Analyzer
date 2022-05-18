#[derive(Debug)]
pub struct symbolTable {
    pub symbolVec: Vec<(String, String ,String)>,
}
static mut id : i32 = 0;
impl symbolTable {
    pub fn new ()-> Self {
        let  mtVector :Vec<(String,  String , String)> = Vec::new();
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
                    element.1 = id.to_string();
                }
                element.2 = _type.clone();
                break;
           }
        }
        let aVariable : (String, String, String);
        unsafe{
            id +=1;
            aVariable=(name, id.to_string(), _type);
        }

        self.symbolVec.push(aVariable);
    }
    pub fn get<'a>(&'a self, name : &'a str) -> (&'a str, &'a str,  &'a str) {
		//println!("CURRENT STATE OF ST: {:?}", self);
       for element in &self.symbolVec {
           if element.0.eq(name){
			   //println!("COMPARE PARAM : {} and EL : {}", name, element.0);
                return (&element.0.as_str(), &element.1.as_str(), &element.2.as_str());
                    
           }
       } 
       ("","-1","")
    }
    pub fn appendArc(&mut self, name : String, target : String) {
        for element in &self.symbolVec{
            if element.0.eq(&target.as_str()){
                unsafe{
                    self.symbolVec.push((name, element.1.clone(), element.2.clone()));
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
