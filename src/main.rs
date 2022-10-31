#![allow(unused_mut)]
#![allow(dead_code)]
#![allow(unused_variables)]
use std::fs;
use std::env;
use shasher;
use std::io::Write;
use std::fs::File;
use std::path::PathBuf;
use std::fs::OpenOptions;
use rayon::prelude::*;
use std::collections::HashMap;


const MAX_DEPTH: u32 = 6;

#[derive(Debug)]
struct Node {
    path: PathBuf,
    depth: u32,
    files: Vec<String>,
    hashes: Vec<String>,
    folders: Vec<Box<Node>>,
}

impl Node {
    pub fn new(path: PathBuf) -> Self {
        Node {
            path: path,
            depth: 0,
            files: Vec::new(),
            hashes: Vec::new(),
            folders: Vec::new(),
        }
    }
    
    pub fn burrow(&mut self) {
        let mut data = ls_dir(&self.path);
        
        self.files = data.par_iter().filter(|x| x.is_file()).map(|y| y.as_path().display().to_string()).collect();
        
        self.hashes.par_extend(
            data.par_iter()
            .filter(|x| x.is_file())
            .map(|y| {
                let hashed_file: String = y.as_path().display().to_string();
                format!("{:0x}", shasher::get_hash(&102, &hashed_file[..]))
            }));
        
        self.folders.par_extend(
            data.par_iter()
            .filter(|x| x.is_dir() && self.depth < MAX_DEPTH)
            .map(|y| {
                let mut new_node = Node::new((*y).to_path_buf());
                new_node.depth = self.depth+1;
                new_node.burrow();
                Box::new(new_node)
            }));
    } 
}

fn write_file_hashes(file_hashmap: HashMap<&String, &String>, path: &str) {
    let mut file = OpenOptions::new()
        .write(true)
        .append(true)
        .open(path)
        .unwrap();
        
    for (key, value) in &file_hashmap {
        if let Err(e) = writeln!(file, "{}:{}", key, value) {
            eprintln!("Could not write to file: {}", e);
        }
    }
} 

fn get_key_value_pairs<'a>(node: &'a Node, key_value_pairs: &mut Vec<(&'a String, &'a String)>) {
    let keys = &node.files;
    let values = &node.hashes;
    let zipped: Vec<(&String, &String)> = keys.par_iter().zip(values).collect::<Vec<(&String, &String)>>();

    key_value_pairs.extend(zipped);
    key_value_pairs.to_vec();

    // Make this loop faster!!
    for n in &node.folders {
        get_key_value_pairs(&n, key_value_pairs);
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
    let args: Vec<String> = env::args().collect();
    let root_path: PathBuf = PathBuf::from(&args[1]);
    let hash_path: &str = &args[2];

    let mut node: Node = Node::new(root_path);
    let mut key_value_pairs: Vec<(&String, &String)> = Vec::new();
    let mut file_hashmap: HashMap<&String, &String>;

    node.burrow();
    get_key_value_pairs(&node, &mut key_value_pairs);
    file_hashmap = key_value_pairs.iter().map(|kv| {return *kv}).collect();
    
    let mut to_file: File = match File::create(hash_path) {
        Ok(_file) => _file,
        Err(_e) => panic!("Error creating file {}", hash_path),
    };
    
    write_file_hashes(file_hashmap, hash_path);
}
