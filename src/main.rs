mod file_cache;
mod connections;
use crossbeam_channel::{bounded};
use crossbeam_utils::thread as scoped_thread;
use std::net::TcpStream;

fn main() {
    
    // read all files into memory
    let file_cache = file_cache::FileCache::from_root_dir("/home/me/git/website");
    println!("done reading {} files to memory", file_cache.files.len());
    
    scoped_thread::scope(|s| {
        let (channel_sender, channel_receiver) = bounded::<TcpStream>(1024);    

        let n_workers = 10;

        for _ in 0..n_workers {
            let channel_receiver_thread = channel_receiver.clone();
            let file_cache_ref = &file_cache;

            s.spawn(move |_| {
                connections::process_from_channel(channel_receiver_thread, file_cache_ref);
            });
        }

        connections::accept_to_channel(channel_sender);
    }).unwrap();
}
