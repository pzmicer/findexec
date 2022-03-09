use std::{fs::{self, DirEntry}, collections::{HashMap, VecDeque}, path::{PathBuf}};
use std::os::linux::fs::MetadataExt;
// use std::os::unix::fs::PermissionsExt;
// use users::{Users, Groups, UserCache};
// use serde_json::{Result, Value, json};
use serde::{Serialize, Deserialize};
use clap::{Parser, ArgEnum};
// use walkdir::{WalkDir, IntoIter};


#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ArgEnum)]
enum OutputType {
    JSON,
}


/// Simple program to greet a person
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct FindexecArgs {
    ///Target directory
    target: String,

    #[clap(short, long)]
    recursively: bool,

    /// Exclude
    #[clap(long)]
    exclude: Option<String>,

    ///Exclude directories starting with <EXCLUDE_DIRS>
    #[clap(long)]
    exclude_dirs: Option<String>,

    #[clap(long)]
    exclude_files: Option<String>,

    #[clap(long)]
    exclude_owner: Option<String>,

    #[clap(short, long, arg_enum)]
    output: Option<OutputType>,
}

#[derive(Serialize, Deserialize)]
struct Owner {
    uid: u32,
    name: String,
    files: Vec<PathBuf>,
}

fn list_dir(args: &FindexecArgs) -> Vec<DirEntry> {
    let mut dirs = VecDeque::new();
    dirs.push_back(PathBuf::from(&args.target));

    let mut result = Vec::<DirEntry>::new();
    while !dirs.is_empty() {
        let elem = dirs.pop_front().expect("BUG: Shouldn't be empty!");
        let contents = match fs::read_dir(elem) {
            Ok(contents) => contents,
            Err(_) => continue,
        };

        for entry in contents.into_iter().filter_map(|e| e.ok()) {
            if entry.path().is_symlink() {
                continue;
            }
            
            if let Some(ref exclude) = args.exclude {
                if entry.file_name().to_str().map(|s| s.contains(exclude)).unwrap_or(false) {
                    continue;
                }
            }

            if args.recursively && entry.path().is_dir() {
                dirs.push_back(entry.path());
            }

            if is_executable(&entry) && entry.path().is_file() {
                result.push(entry);      
            }
        }
    }

    result
}

fn name_starts_with(str: &str, entry: &DirEntry) -> bool {
    entry.file_name()
        .to_str()
        .map(|s| s.starts_with(str))
        .unwrap_or(false)
}

fn is_dir(entry: &DirEntry) -> bool {
    entry.metadata()
        .unwrap()
        .is_dir()
}

fn is_executable(entry: &DirEntry) -> bool {
    entry.metadata().unwrap().st_mode() & 0o100 != 0
}

fn to_json() -> String {
    // JSON Serialization
    let mut for_json: Vec<Owner> = Vec::new();
    for (key, value) in &grouped_by_onwer {
        let files: Vec<PathBuf> = value.iter().map(|&x| x.path()).collect();
        for_json.push(Owner { 
            uid: *key, 
            name: String::new(), 
            files: files,
        });
    }

    for_json.sort_by(|a, b| b.files.len().cmp(&a.files.len()));

    // println!("{}", serde_json::to_string(&for_json).unwrap());
}

fn main() {
    println!("{:o}", fs::metadata("./src").unwrap().st_mode());
    println!("{:o}", fs::metadata("./src/main.rs").unwrap().st_mode());
    println!("{}", fs::metadata("./src/main.rs").unwrap().st_mode() & 0o100);
    
    // for entry in WalkDir::new("/etc").into_iter() {
    //     println!("{}", entry.unwrap().path().display());
    // }

   let app = FindexecArgs::parse();

    println!("{}", app.target);

    let contents = list_dir(&app);

    for file in &contents {
        println!("{}", file.path().display());
    }

    let mut grouped_by_onwer: HashMap<u32, Vec<&DirEntry>> = HashMap::new();
    
    for entry in &contents {
        let metadata = entry.metadata().unwrap();
        grouped_by_onwer.entry(metadata.st_uid()).or_insert(Vec::new()).push(entry);
    }
    
    
}
