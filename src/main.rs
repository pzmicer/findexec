use std::{fs::{self, DirEntry}, collections::HashMap, path::PathBuf};
use std::os::linux::fs::MetadataExt;
// use users::{Users, Groups, UserCache};
use serde_json::{Result, Value, json};
use serde::{Serialize, Deserialize};


#[derive(Serialize, Deserialize)]
struct Owner {
    uid: u32,
    name: String,
    files: Vec<PathBuf>,
}

fn main() {
    let dir: Vec<DirEntry> = fs::read_dir("/etc").unwrap().map(|x| x.unwrap()).collect();
    for file in &dir {
        // println!("{}", file.path().display());
    }

    let mut grouped_by_onwer: HashMap<u32, Vec<&DirEntry>> = HashMap::new();
    println!("\n\n\n");
    for entry in &dir {
        // let p = entry.path();
        if let Ok(metadata) = entry.metadata() {
            // println!("{:?}", metadata.st_uid());
            // println!("{:?}", metadata.permissions());
            
            grouped_by_onwer.entry(metadata.st_uid()).or_insert(Vec::new()).push(entry);
        } else {
            println!("Can't read metadata");
        }
    }

    let mut for_json: Vec<Owner> = Vec::new();

    for (key, value) in &grouped_by_onwer {
        // println!("{}\n{:?}", key, value);
        let files = value.iter().map(|x| x.path()).collect();
        for_json.push(Owner { 
            uid: *key, 
            name: String::new(), 
            files: files,
        });
    }

    println!("{}", serde_json::to_string(&for_json).unwrap());
}
