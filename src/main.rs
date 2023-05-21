#![allow(unused_mut)]
#![allow(dead_code)]
#![allow(unused_variables)]
use std::fs;
use std::path::PathBuf;
use rayon::prelude::*;
use aes_crypt;
use clap::{App, Arg};
use std::path::Path;
use std::process;


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
                aes_crypt::encrypt(path, epath, key); 
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
                aes_crypt::decrypt(epath, dpath, key); 
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

fn ls_dir(path: &PathBuf) -> Vec<PathBuf> {
    let mut contents: Vec<PathBuf> = Vec::new();
    let current_dir: fs::ReadDir = fs::read_dir(path).unwrap();

    for file in current_dir {
        contents.push(file.unwrap().path());
    }
    contents
}

fn check_arguments(matches: &clap::ArgMatches, key_path: &str, target_path: &str) {
    if matches.is_present("encrypt") && matches.is_present("decrypt") {
        eprintln!("Error: Both encrypt and decrypt flags are given, please provide only one.");
        process::exit(1);
    }

    if matches.is_present("decrypt") && !Path::new(&key_path).exists() {
        eprintln!("Error: The provided key file path does not exist.");
        process::exit(1);
    }

    if !Path::new(&target_path).exists() {
        eprintln!("Error: The provided target file or directory does not exist.");
        process::exit(1);
    }

    if !matches.is_present("encrypt") && !matches.is_present("decrypt") {
        eprintln!("Error: Either the encrypt or decrypt flag must be provided.");
        process::exit(1);
    }
}

fn main() {
    let matches = App::new("encrypter")
        .version("1.0")
        .about("Encrypts or Decrypts a target file or directory")
        .arg(
            Arg::with_name("encrypt")
                .short('e')
                .long("encrypt")
                .help("Encrypt the target file or directory"),
        )
        .arg(
            Arg::with_name("decrypt")
                .short('d')
                .long("decrypt")
                .help("Decrypt the target file or directory"),
        )
        .arg(
            Arg::with_name("target")
                .short('t')
                .long("target")
                .value_name("FILE OR DIRECTORY")
                .help("Specify the target file or directory to be encrypted or decrypted")
                .takes_value(true)
                .required(true),
        )
        .arg(
            Arg::with_name("key")
                .short('k')
                .long("key")
                .value_name("KEY FILE")
                .help("Specify the path where the key file will be saved or can be found")
                .takes_value(true)
                .required(true),
        )
        .get_matches();

    let key_path = matches.value_of("key").unwrap();
    let target_path = matches.value_of("target").unwrap();
    check_arguments(&matches, key_path, target_path);

    let mut node = Node::new(PathBuf::from(target_path));
    if matches.is_present("encrypt") {
        aes_crypt::gen_key(key_path);
        node.encrypt(key_path);
    } else {
        node.decrypt(key_path);
    }
}

/* 
fn main() {
    let key_path: &str = "./key.txt";
    //aes_crypt::gen_key(key_path);

    let root_path: PathBuf = PathBuf::from("/home/arbegla/projects/rust/binaries/encrypter_cpy");
    //let mut enode: Node = Node::new(root_path.clone());
    let mut dnode: Node = Node::new(root_path.clone());

    //enode.encrypt(key_path);
    dnode.decrypt(key_path);
}

*/
