use petgraph::Outgoing;
use std::sync::mpsc::{Receiver};
use petgraph::algo::is_cyclic_directed;
use petgraph::{Graph,algo};
use petgraph::graph::NodeIndex;
use petgraph::visit::Dfs;
pub struct GraphMaker {
    recv : Receiver <(i32, String,String,String, String, usize)>,
    graph : Graph<GNode, Edge>,
}
#[derive(Debug)]
struct GNode {
    lockName: String,
    tidBlock : Vec<(i32,String)>,
    primitive : Vec<String>,
    visit : i32,
    file_name : String,
    line_num : usize,
}
#[derive(Debug)]
struct Edge{
    rw : EdgeInfo
}
#[derive(Debug)]
enum EdgeInfo {
   None,
   lock_info(GNode),
}

impl GNode {
    fn clone(&self) -> Self {
        Self {
            lockName : self.lockName.clone(),
            tidBlock : self.tidBlock.clone(),
            primitive : self.primitive.clone(),
            visit : self.visit,
            file_name : self.file_name.clone(),
            line_num : self.line_num,
        }
    }
}
impl GraphMaker {
    pub fn new(rec : Receiver<(i32,String,String, String, String, usize)>)->Self{
        let mut Graph = Graph::<GNode,Edge>::new();
        Self{
            recv: rec,
            graph : Graph,
        }
    }
    pub fn run(&mut self) {
        loop {
            let received : (i32, String, String, String,String,usize) = self.recv.recv().unwrap();
            println!("**************RECEIVED {:?}",received);
            let mut tidVec : Vec<(i32,String)>  = Vec::new();
            tidVec.push((received.0, received.2));
            let mut prim_vec : Vec<String> = Vec::new();
            prim_vec.push(received.3);
            let gnode_bowl = GNode {
                lockName : received.1,
                tidBlock : tidVec,
                primitive : prim_vec,
                visit : 0,
                file_name : received.4,
                line_num : received.5

            };
            self.make_graph(gnode_bowl);
            
        }

    }
    
    
    fn dfs(&mut self, n : NodeIndex, mut  path : Vec<(String, bool)>, mut lock_position : Vec<(String, usize)>) -> bool{
        let mut  file_name = String::new();
        let mut line_num : usize = 0;
        {
            let cur  = self.graph.node_weight_mut(n).unwrap();
            //println!("From {:?}",cur);
            cur.visit = 1;
            if !&cur.primitive[0].eq("lock"){
                for element in &cur.primitive {
                    let mut is_write : bool = false;
                    if element.eq("write"){
                        is_write = true;
                    }
                        path.push((cur.lockName.clone(), is_write ));
                }
            }
            file_name = cur.file_name.clone();
            line_num = cur.line_num;
            if !lock_position.contains(&(file_name.clone(), line_num)) {
                lock_position.push((file_name, line_num));
            }
        }

        //println!("Vector {:?}",path);
        let mut ret_val = false;
        let mut nodes = self.graph.neighbors_directed(n, Outgoing).detach();
        while let Some(node) = nodes.next_node(&self.graph){
            let mut cur_visit =0;
            let mut lock_name  = String::from("");
            let mut lock_primitive = Vec::new();
            {
            let cur  = self.graph.node_weight_mut(node).unwrap();
            //println!("To {:?}",cur);
            lock_name = cur.lockName.clone();
            cur_visit = cur.visit;
            lock_primitive = cur.primitive.clone();
            }
            if lock_primitive[0].eq("lock"){
                if cur_visit==1 {
                    println!("Deadlock on {} {:?}" ,lock_name, lock_position);
                    return true;
                }          
                else if cur_visit ==0{
                    ret_val = self.dfs(node, path.clone(), lock_position.clone());
                }

            }
            else {
                    let edge=self.graph.find_edge(n,node).unwrap();
                    let mut is_write : bool = false;
                    let mut arg_path = path.clone();
                    if let EdgeInfo::lock_info(e_gnode) = &self.graph.edge_weight(edge).unwrap().rw{
                        //println!("Edge :: {:?}", e_gnode);
                        if e_gnode.primitive[0].eq("write"){
                            is_write = true;
                        }
                        arg_path.push((e_gnode.lockName.clone(), is_write));
                    }
                if cur_visit ==0 {
                        ret_val = self.dfs(node, arg_path.clone(), lock_position.clone());
                    }
                else {
                    //println!("*******************************************cycle!!\n{:?}",path);
                    if GraphMaker::check(&path){
                        println!("Deadlock on {}",lock_name);
                        return true;
                    }
                }
            
            }

        }
        {
            let cur  = self.graph.node_weight_mut(n).unwrap();
            cur.visit = 0;
        }
        return ret_val;

    }
    fn check( path :& Vec<(String,bool)>) -> bool{
            let mut rw_list : Vec<(String,bool)> = Vec::new();
            for element in path {
                let mut has = false;
                for l_element in &mut rw_list {
                    if element.0.eq(l_element.0.as_str()){
                        if !l_element.1 && element.1 {
                            l_element.1 = true;
                        }
                        has = true;
                        break;    
                    }
                }

                if !has {
                    rw_list.push(element.clone());
                    //println!("rw_list {:?}",rw_list);
                }
            }
            
            let mut ret_val = true;

            for element in rw_list {
                if !element.1 {
                    ret_val =false;
                }    
            }
            ret_val
    }
    fn search(&mut self) {
        let end = self.graph.node_count() ;
        //let cur_visit  = self.graph.node_weight_mut(NodeIndex::new(end)).unwrap().visit;
        for n in 0 .. end {
            println!("graph : {:?}", self.graph[NodeIndex::new(n)]);
        }

    }
    
    fn make_graph(&mut self, gnode : GNode){
        let new_lock_name = gnode.lockName.clone();
        let node_index = self.add_to_graph(gnode);
        /*
        if is_cyclic_directed(&self.graph){
            println!("Deadlock! on {}",new_lock_name);
        }
        */
        //self.search();
        self.dfs(node_index, Vec::new(), Vec::new());
    }

    fn add_to_graph(&mut self, gnode:GNode)->NodeIndex {
        let add_or_not : bool = true;
        let SameNode = self.graph
            .node_indices().rev().find(|i| self.graph[*i].lockName == gnode.lockName);
        let mut exist : GNode;
        match SameNode {
            Some(existing) =>{
                if GraphMaker::compare_block(&gnode.tidBlock[0], &self.graph.node_weight(existing).unwrap().tidBlock){
                    //println!("self lock {}",&gnode.lockName);
                    if gnode.primitive[0].eq("lock") {
                        self.graph.add_edge(existing, existing, 
                                            Edge{
                                                rw : EdgeInfo::None,
                                            });
                    }
                    else {
                        let gnode_prim = gnode.primitive[0].clone();
                        self.graph.add_edge(existing, existing, 
                                            Edge{
                                                rw : EdgeInfo::lock_info(gnode),
                                            });
                        self.graph.node_weight_mut(existing).unwrap().primitive.push(gnode_prim);
                    }

                }
                else if GraphMaker::compare_TID(&gnode.tidBlock[0], &self.graph.node_weight(existing).unwrap().tidBlock){
                    let iterNode = self.graph.node_indices();
                    let gnodeTup = gnode.tidBlock[0].clone();
                    let gnode_prim = gnode.primitive[0].clone();

                    //println!("same lock different block {}", &gnode.lockName);
                    //println!("Added {:?}", gnode);
                    //let added_node = self.graph.add_node(gnode);
                    for element in iterNode {
                        if GraphMaker::compare_block(&gnodeTup, &self.graph.node_weight(element).unwrap().tidBlock){
							/*
                            println!("add edge {} to {}",
                                     &self.graph.node_weight(element).unwrap().lockName,
                                     &self.graph.node_weight(existing).unwrap().lockName);
                            println!("add edge {:?} to {:?}",
                                     &self.graph.node_weight(element).unwrap().tidBlock,
                                     &self.graph.node_weight(existing).unwrap().tidBlock);
                            */
                            let gnode_primitive = gnode.primitive[0].clone();
                            if gnode_primitive.eq("lock") {
                                self.graph.add_edge(element, existing, 
                                                    Edge{
                                                        rw : EdgeInfo::None,
                                                    });
                                break;
                            }
                            else {
                                self.graph.add_edge(element, existing, 
                                                    Edge{
                                                        rw : EdgeInfo::lock_info(gnode),
                                                    });
                                break;
                            }
							
                        }

                    }
                    self.graph.node_weight_mut(existing).unwrap().tidBlock.push(gnodeTup);
                    self.graph.node_weight_mut(existing).unwrap().primitive.push(gnode_prim);
                }
                else{

                    println!("here");
                    let gnodeTup = gnode.tidBlock[0].clone();
                    let gnode_prim = gnode.primitive[0].clone();
                    let iterNode = self.graph.node_indices();
                    for element in iterNode {
                        if GraphMaker::compare_block(&gnodeTup, &self.graph.node_weight(element).unwrap().tidBlock){
							/*
                            println!("add edge {} to {}",
                                     &self.graph.node_weight(element).unwrap().lockName,
                                     &self.graph.node_weight(existing).unwrap().lockName);
                            println!("add edge {:?} to {:?}",
                                     &self.graph.node_weight(element).unwrap().tidBlock,
                                     &self.graph.node_weight(existing).unwrap().tidBlock);
							*/
                            let gnode_primitive = gnode.primitive[0].clone();
                            if gnode_primitive.eq("lock") {
                                self.graph.add_edge(element, existing, 
                                                    Edge{
                                                        rw : EdgeInfo::None,
                                                    });
                                break;
                            }
                            else {
                                self.graph.add_edge(element, existing, 
                                                    Edge{
                                                        rw : EdgeInfo::lock_info(gnode),
                                                    });
                                break;
                            }

                        }
                    
                   }
                    self.graph.node_weight_mut(existing).unwrap().tidBlock.push(gnodeTup);
                    self.graph.node_weight_mut(existing).unwrap().primitive.push(gnode_prim);
                    //println!("pushed {:?}", self.graph.node_weight_mut(existing).unwrap());
                }
                existing
            },
            None => {
                let gnodeTup = gnode.tidBlock[0].clone();
                let iterNode = self.graph.node_indices();
                //println!("Added {:?}", gnode);
                let added_node = self.graph.add_node(gnode.clone());
                for element in iterNode {
                    if GraphMaker::compare_block(&gnodeTup, &self.graph.node_weight(element).unwrap().tidBlock){
							/*
                            println!("add edge {} to {}",
                                     &self.graph.node_weight(element).unwrap().lockName,
                                     &self.graph.node_weight(added_node).unwrap().lockName);
                            println!("add edge {:?} to {:?}",
                                     &self.graph.node_weight(element).unwrap().tidBlock,
                                     &self.graph.node_weight(added_node).unwrap().tidBlock);
							*/
                            if gnode.primitive[0].eq("lock") {
                                self.graph.add_edge(element, added_node.clone(), 
                                                    Edge{
                                                        rw : EdgeInfo::None,
                                                    });
                                break;
                            }
                            else {
                                self.graph.add_edge(element, added_node.clone(), 
                                                    Edge{
                                                        rw : EdgeInfo::lock_info(gnode),
                                                    });
                                break;
                            }
                    }
                
               }
                added_node
            },
        }
    }
    fn tidVecFind(tidVec :&Vec<(i32,String)>, gnodeVec : &(i32,String)) ->bool{
        let mut tf = false;
        for element in tidVec{
            if element.0 == gnodeVec.0 && element.1 ==gnodeVec.1{
                return true;
            }
        }
        tf
    }
    fn compare_TID(gnode:&(i32, String), graph_node : &Vec<(i32,String)>) -> bool{
        let gnode_tid = gnode.0;
        for element in graph_node {
            let graph_tid = element.0;
            if gnode_tid == graph_tid{
                return true;
            }
        }
        return false;
    }
    fn compare_block(gnode:&(i32,String), graph_node : &Vec<(i32,String)>) ->bool {
        let gnodeBlock : Vec<&str> = gnode.1.split("-").collect();
        for element in graph_node {
            let graphBlock : Vec<&str> = element.1.split("-").collect();
            if gnodeBlock.len() <= graphBlock.len(){
                for i in 0..gnodeBlock.len(){
                    if gnodeBlock[i] !=graphBlock[i]{
                        break;
                    }
                    if i == gnodeBlock.len()-1{
                        return true;
                    }
                }
            }
            else {
                for i in 0..graphBlock.len(){
                    if graphBlock[i] != gnodeBlock[i]{
                        break;
                    }
                    if i == graphBlock.len()-1{
                        return true;
                    }
                }
            }
        }
        return false;
        /*
        let graphBlock : Vec<&str> = graph_node.split(":").collect();

        if gnodeBlock.len() <= graphBlock.len(){
            for i in (0..gnodeBlock.len()) {
                if gnodeBlock[i] !=graphBlock[i]{
                    return false;
                }
            }
            return true;
        }
        else {
            for i in (0..graphBlock.len()){
                if graphBlock[i]!=gnodeBlock[i] {
                    return false;
                }
            }
            return true;
        }
        */
    }

}
