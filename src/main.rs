mod file_cache;
mod connections;
mod mime_types;
use crossbeam_channel::{bounded};
use crossbeam_utils::thread as scoped_thread;
use std::net::TcpStream;
use std::time::Instant;
use std::env;

use std::convert::Infallible;
use std::net::SocketAddr;
use hyper::{Body, Request, Response, Server};
use hyper::service::{make_service_fn, service_fn};



fn main() {

    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        println!("usage: ./{} <webroot_path> <number_of_workers>", args[0]);
        return;
    }

    let webroot_path = &args[1];
    let n_workers = args[2].parse::<usize>().unwrap();
    
    println!("Static Webserver\nwebroot_path: \"{}\"\nn_workers: {}\nlistening on: 0.0.0.0:8080", webroot_path, n_workers);

    // generate read-only file cache for serving
    let time_start = Instant::now();
    let file_cache = file_cache::FileCache::from_root_dir(webroot_path);
    print!("initializing ... ");
    println!("done reading {} files to memory ({}ms)", file_cache.files.len(), time_start.elapsed().as_millis());
    
    scoped_thread::scope(|s| {

        // create single-producer-multiple-consumer channel for incoming requests
        let (channel_sender, channel_receiver) = bounded::<TcpStream>(1024);    

        // spawn all workers
        for _ in 0..n_workers {
            let channel_receiver_thread = channel_receiver.clone();
            let file_cache_ref = &file_cache;

            // all workers will keep processing incoming requests
            s.spawn(move |_| {
                connections::process_from_channel(channel_receiver_thread, file_cache_ref);
            });
        }

        // now use "main thread" to accept incoming connections
        connections::accept_to_channel(channel_sender);
    }).unwrap();
}
