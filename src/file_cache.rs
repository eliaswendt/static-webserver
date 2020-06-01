use std::fs;
use std::path;
use std::io::{Read, BufReader};
use std::collections::{LinkedList, HashMap};
use crate::mime_types;


pub struct File {
    relative_path: String,
    pub mime_type: String,
    pub payload: Vec<u8>
}

pub struct FileCache {
    pub files: HashMap<String, File>
}

impl FileCache {
    pub fn from_root_dir(root_dir: &str) -> FileCache {

        let extension_to_mime_type_map = mime_types::mime_types::get_extension_to_mime_type_map();

        let len_root_dir = root_dir.len();
        let file_path_list = get_file_path_list(root_dir);

        let mut files: HashMap<String, File> = HashMap::new();

        let mut buf = Vec::new();

        for file_path in file_path_list {
            match fs::File::open(&file_path) {
                Ok(file) => {
                    
                    buf.clear();
                    match BufReader::new(file).read_to_end(&mut buf) {
                        Ok(_) => { // all fine, read file, insert file into cache

                            let relative_path = &file_path.to_str().unwrap()[len_root_dir..];

                            // very messy code to get file extension of current file 
                            // we first need to get the file_name by splitting away paths
                            // then we need to split the file_name by the dot char and take the last result as extension
                            let mut file_extension = ""; // default file extension
                            let path_splits: Vec<&str> = relative_path.split("/").collect();
                            if path_splits.len() > 0 {
                                let file_name = path_splits[path_splits.len()-1];
                                let dot_splits: Vec<&str> = file_name.split(".").collect();
                                if dot_splits.len() > 0 {
                                    file_extension = dot_splits[dot_splits.len()-1]
                                }
                            }

                            // last step is to lookup if we can translate this extension to mime_type
                            let mime_type = match extension_to_mime_type_map.get(file_extension) {
                                Some(mime_type) => mime_type, // found mime_type
                                None => {
                                    "application/octet-stream"
                                }
                            };

                            files.insert(
                                String::from(&file_path.to_str().unwrap()[len_root_dir..]), 
                                File {
                                    relative_path: String::from(relative_path),
                                    mime_type: String::from(mime_type),
                                    payload: buf.to_vec()
                                }
                            );
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

    pub fn get_file(&self, filepath: &str) -> Option<&File> {

        let filepath = if filepath.ends_with("/") {
            format!("{}index.html", filepath)
        } else {
            String::from(filepath)
        };

        self.files.get(&filepath)
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
