use std::sync::mpsc::{Receiver};
use petgraph::algo::is_cyclic_directed;
use petgraph::{Graph,algo};
pub struct GraphMaker {
    recv : Receiver <(i32, String,String,String)>,
    graph : Graph<GNode, Edge>,
}
#[derive(Debug)]
struct GNode {
    lockName: String,
    tidBlock : Vec<(i32,String)>,
    primitive : String,
}
struct Edge{

}


impl GraphMaker {
    pub fn new(rec : Receiver<(i32,String,String, String)>)->Self{
        let mut Graph = Graph::<GNode,Edge>::new();
        Self{
            recv: rec,
            graph : Graph,
        }
    }
    pub fn run(&mut self) {
        loop {
            let received : (i32, String, String, String) = self.recv.recv().unwrap();
            //println!("RECEIVED {:?}",received);
            let mut tidVec : Vec<(i32,String)>  = Vec::new();
            tidVec.push((received.0, received.2));
            let gnode_bowl = GNode {
                lockName : received.1,
                tidBlock : tidVec,
                primitive : received.3,
            };
            self.make_graph(gnode_bowl);

        }

    }
    
    
    
    
    
    fn make_graph(&mut self, gnode : GNode){
        let new_lock_name = gnode.lockName.clone();
        self.add_to_graph(gnode);
        if is_cyclic_directed(&self.graph){
            println!("Deadlock! on {}",new_lock_name);
        }
    }

    fn add_to_graph(&mut self, gnode:GNode) {
        let add_or_not : bool = true;
        let SameNode = self.graph
            .node_indices().rev().find(|i| self.graph[*i].lockName == gnode.lockName);
        let mut exist : GNode;
        match SameNode {
            Some(existing) =>{
                if GraphMaker::compare_block(&gnode.tidBlock[0], &self.graph.node_weight(existing).unwrap().tidBlock){
                    //println!("self lock {}",&gnode.lockName);
                    self.graph.add_edge(existing, existing, Edge{});

                }
                else if GraphMaker::compare_TID(&gnode.tidBlock[0], &self.graph.node_weight(existing).unwrap().tidBlock){
                    let iterNode = self.graph.node_indices();
                    let gnodeTup = gnode.tidBlock[0].clone();
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
							 self.graph.add_edge(element,existing, Edge{});
							
                        }

                    }
                    self.graph.node_weight_mut(existing).unwrap().tidBlock.push(gnode.tidBlock[0].clone());
                }
                else{

                    let gnodeTup = gnode.tidBlock[0].clone();
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
							self.graph.add_edge(element,existing, Edge{});

                        }
                    
                   }
                    self.graph.node_weight_mut(existing).unwrap().tidBlock.push(gnode.tidBlock[0].clone());
                    //println!("pushed {:?}", self.graph.node_weight_mut(existing).unwrap());
                }
            },
            None => {
                let gnodeTup = gnode.tidBlock[0].clone();
                let iterNode = self.graph.node_indices();
                //println!("Added {:?}", gnode);
                let added_node = self.graph.add_node(gnode);
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
                        self.graph.add_edge(element,added_node, Edge{});
                    }
                
               }
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
