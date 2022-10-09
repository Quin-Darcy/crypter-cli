#![allow(unused_mut)]
#![allow(dead_code)]
#![allow(unused_variables)]
use std::fs;
use shasher;
use std::path::{Path, PathBuf};


#[derive(Debug)]
struct Node {
    path: PathBuf,
    files: Vec<PathBuf>,
    hashes: Vec<String>,
    folders: Vec<Box<Node>>,
}

impl Node {
    pub fn new(path: PathBuf) -> Self {
        Node {
            path: path,
            files: Vec::new(),
            hashes: Vec::new(),
            folders: Vec::new(),
        }
    }

    pub fn is_leaf(&self) -> bool {
        if self.folders.len() == 0 {
            true
        } else {
            false
        }
    }
    
    pub fn burrow(&mut self) {
        let mut contents: Vec<PathBuf> = ls_dir(&self.path);

        for item in contents {
            if item.is_file() {
                let hashed_file: String = item.as_path().display().to_string();
                self.hashes.push(format!("{:0x}", shasher::get_hash(&102, &hashed_file[..])));
            } else if item.is_dir() {
                let mut new_folder = Node::new(item);
                new_folder.burrow();
                self.folders.push(Box::new(new_folder));
            }
        }
    }

    pub fn show(&self) {
        println!("PATH: {}", self.path.as_path().display());
        println!("HASHES: {:?}", self.hashes);
        for folder in &self.folders {
            folder.show();
        }
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
    let root_path: PathBuf = PathBuf::from("/home/runner/dircrawl");
    let contents: Vec<PathBuf> = ls_dir(&root_path);
    let mut node: Node = Node::new(root_path);
   
    node.burrow();
    node.show();
}