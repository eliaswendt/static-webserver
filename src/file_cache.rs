use std::fs;
use std::path;
use std::io::{Read, BufReader};
use std::collections::{LinkedList, HashMap};

pub struct File {
    content_type: String,
    
}

pub struct FileCache {
    pub files: HashMap<String,Vec<u8>>
}

impl FileCache {
    pub fn from_root_dir(root_dir: &str) -> FileCache {

        let len_root_dir = root_dir.len();
        let file_path_list = get_file_path_list(root_dir);

        let mut files: HashMap<String, Vec<u8>> = HashMap::new();
        let mut buf = Vec::new();

        for file_path in file_path_list {
            match fs::File::open(&file_path) {
                Ok(file) => {
                    
                    buf.clear();
                    match BufReader::new(file).read_to_end(&mut buf) {
                        Ok(_) => { // all fine, continue with next file
                            files.insert(String::from(&file_path.to_str().unwrap()[len_root_dir..]), buf.to_vec());
                        },
                        Err(e) => println!("Could not read file \"{}\": {}", file_path.to_str().unwrap(), e)
                    }
                },
                Err(e) => println!("Could not open file \"{}\": {}", file_path.to_str().unwrap(), e)
            };
        }

        FileCache {
            files
        }
    }

    pub fn get_file(&self, filepath: &str) -> &[u8] {

        let filepath = if filepath.ends_with("/") {
            format!("{}index.html", filepath)
        } else {
            String::from(filepath)
        };

        match self.files.get(&filepath) {
            Some(file) => file,
            None => "Not Found".as_bytes()
        }
    }
}


fn get_file_path_list(root_dir: &str) -> LinkedList<path::PathBuf> {

    let mut file_path_list: LinkedList<path::PathBuf> = LinkedList::new();

    // list with paths to 
    let mut path_queue: LinkedList<path::PathBuf> = LinkedList::new();
    path_queue.push_back(path::Path::new(root_dir).to_path_buf());

    loop {
        let dir = match path_queue.pop_front() {
            Some(path) => match fs::read_dir(path) {
                Ok(dir) => dir,
                Err(e) => {
                    println!("Could not read dir: {} -> continue with next dir", e);
                    continue;
                }
            },
            None => break // done with all paths
        };

        for entry_result in dir {
            let entry = match entry_result {
                Ok(entry) => entry,
                Err(e) => {
                    println!("Could not read entry: {} -> continue with next entry", e);
                    continue;
                }
            };


            let path = entry.path();

            let metadata = match fs::metadata(&path) {
                Ok(metadata) => metadata,
                Err(e) => {
                    println!("Could not read entrie's metadata: {} -> continue with next entry", e);
                    continue;
                }
            };

            if metadata.is_dir() {
                path_queue.push_back(path);
            } else if metadata.is_file() {
                file_path_list.push_back(path);
            }

        }
    }

    file_path_list
}
