use std::{fs::{self, DirEntry, File}, collections::{HashMap, VecDeque}, path::{PathBuf}, io::Read};
use std::os::linux::fs::MetadataExt;
use users::{get_user_by_uid, get_user_by_name};
use serde::{Serialize, Deserialize};
use clap::{Parser, ArgEnum};


#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ArgEnum)]
enum OutputType {
    JSON,
}

/// This application lists ELF files in specified directory and groups them by user
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct FindexecArgs {
    /// Target directory
    target: String,

    /// Recursively list directory
    #[clap(short, long)]
    recursively: bool,

    /// Exclude files which contains specified string
    #[clap(long)]
    exclude: Option<String>,

    /// Exclude files by username
    #[clap(long)]
    exclude_user: Option<String>,

    /// Output type
    #[clap(short, long, arg_enum)]
    output: Option<OutputType>,
}

#[derive(Serialize, Deserialize)]
struct Owner {
    uid: u32,
    username: String,
    files: Vec<PathBuf>,
    amount: usize,
    size: u64,
}

fn list_dir(args: &FindexecArgs) ->Vec<DirEntry> {
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

            if let Some(ref exclude_user) = args.exclude_user {
                let user = get_user_by_name(exclude_user);
                if user.is_some() && entry.metadata().unwrap().st_uid() == user.unwrap().uid() {
                    continue;
                }
            }

            if args.recursively && entry.path().is_dir() {
                dirs.push_back(entry.path());
            }

            if entry.path().is_file() && is_elf(entry.path()) { //is_executable(&entry)
                result.push(entry);      
            }
        }
    }

    result
}

fn is_executable(entry: &DirEntry) -> bool {
    entry.metadata().unwrap().st_mode() & 0o100 != 0
}

fn is_elf(path: PathBuf) -> bool {
    let mut file = match File::open(&path) {
        Ok(file) => file, 
        Err(_) => {
            println!("Can't open file {}", path.display());
            return false;
        }
    };

    let amount = 4;
    let mut buffer = vec![0u8; amount as usize];
    match file.read(&mut buffer) {
        Ok(n) => {
            if n < amount {
                return false;
            }
        },
        Err(_) => {
            return false;
        }
    }

    &buffer[1..] == vec![0x45u8, 0x4cu8, 0x46u8] // ELF
}

fn get_owners(contents: &Vec<DirEntry>) -> Vec<Owner> {
    let mut grouped_by_onwer: HashMap<u32, Vec<&DirEntry>> = HashMap::new();
    
    for entry in contents {
        let metadata = entry.metadata().unwrap();
        grouped_by_onwer.entry(metadata.st_uid()).or_insert(Vec::new()).push(entry);
    }

    let mut owners: Vec<Owner> = Vec::new();
    for (key, value) in &grouped_by_onwer {
        let files: Vec<PathBuf> = value.iter().map(|&x| x.path()).collect();
        let size = files.iter().map(|e| e.metadata().unwrap().st_size()).sum();
        let amount = files.len();

        let owner = match get_user_by_uid(*key) {
            Some(owner) => owner,
            None => continue,
        };

        owners.push(Owner { 
            uid: owner.uid(), 
            username: owner.name().to_str().unwrap().to_owned(), 
            files,
            amount,
            size,
        });
    }

    owners
}

fn main() {
    let app = FindexecArgs::parse();
    
    let contents = list_dir(&app);

    let mut owners = get_owners(&contents);

    owners.sort_by(|a, b| b.files.len().cmp(&a.files.len()));

    if let Some(output) = app.output {
        match output {
            OutputType::JSON => {
                println!("{}", serde_json::to_string(&owners).unwrap());
            }
        }
    } else {
        for owner in &owners {
            println!("{}: {:?}, amount = {}, size = {};", owner.username, owner.files, owner.amount, owner.size);
        }
    }
}
