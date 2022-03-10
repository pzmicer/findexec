use std::{fs::{self, DirEntry}, collections::{HashMap, VecDeque}, path::{PathBuf}};
use std::os::linux::fs::MetadataExt;
use users::{get_user_by_uid};
use serde::{Serialize, Deserialize};
use clap::{Parser, ArgEnum};


#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ArgEnum)]
enum OutputType {
    JSON,
}


#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct FindexecArgs {
    ///Target directory
    target: String,

    #[clap(short, long)]
    recursively: bool,

    /// Exclude files which contains specified string
    #[clap(long)]
    exclude: Option<String>,

    #[clap(short, long, arg_enum)]
    output: Option<OutputType>,
}

#[derive(Serialize, Deserialize)]
struct Owner {
    uid: u32,
    name: String,
    files: Vec<PathBuf>,
    size: u64,
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

fn is_executable(entry: &DirEntry) -> bool {
    entry.metadata().unwrap().st_mode() & 0o100 != 0
}

fn main() {
   let app = FindexecArgs::parse();
    let contents = list_dir(&app);

    let mut grouped_by_onwer: HashMap<u32, Vec<&DirEntry>> = HashMap::new();
    
    for entry in &contents {
        let metadata = entry.metadata().unwrap();
        grouped_by_onwer.entry(metadata.st_uid()).or_insert(Vec::new()).push(entry);
    }
    
    // Serialization

    let mut owners: Vec<Owner> = Vec::new();
    for (key, value) in &grouped_by_onwer {
        let files: Vec<PathBuf> = value.iter().map(|&x| x.path()).collect();
        let size = files.iter().map(|e| e.metadata().unwrap().st_size()).sum();

        let owner = if let Some(owner) = get_user_by_uid(*key) {
                owner
        } else {
            println!("Can't find user!");
            continue
        };

        owners.push(Owner { 
            uid: owner.uid(), 
            name: owner.name().to_str().unwrap().to_owned(), 
            files: files,
            size: size,
        });
    }

    owners.sort_by(|a, b| b.files.len().cmp(&a.files.len()));


    if let Some(output) = app.output {
        match output {
            OutputType::JSON => {
                println!("{}", serde_json::to_string(&owners).unwrap());
            }
        }
    } else {
        for owner in &owners {
            println!("{}: {:?}", owner.uid, owner.files);
        }
    }
}
