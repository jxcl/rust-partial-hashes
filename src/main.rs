#![feature(core,collections)]
extern crate rand;
extern crate crypto;
extern crate "rustc-serialize" as rustc_serialize;

use std::mem::transmute;
use std::thread;
use std::sync::mpsc::{channel, Sender};
use std::sync::{Arc,Mutex};

use rustc_serialize::hex::ToHex;

use crypto::digest::Digest;
use crypto::sha2::Sha256;

type NumTries = u64;
type ResultStr = String;
type HashThreadResult = Result<(ResultStr, NumTries), NumTries>;

// Encapsulates all the fields that a new thread requires
// to start calculating hashes.
struct HashThreadProperties {
    tx: Sender<HashThreadResult>,
    in_string: Arc<String>,
    hash_found_mutex: Arc<Mutex<bool>>,
    thread_n: i64,
    hasher: Sha256,
    num_procs: i64,
    num_zeros: u32,
}

// Concatenate a string with the hex representation of an i64
// with a `:` in between.
fn build_string(in_string: &str, n: i64) -> String {
    let mut header = String::from_str(in_string);
    let n_bytes: [u8; 8] = unsafe { transmute(n) };
    let n_hex = n_bytes.to_hex();

    header.push(':');
    header.push_str(n_hex.as_slice());

    header
}

fn find_hash(input: &str, hasher: &mut Sha256) -> String {
    (*hasher).input_str(input);
    (*hasher).result_str()
}

fn valid_hash(hash: &str, num_zeros: u32) -> bool {
    for x in 0..num_zeros {
        if hash.char_at(x as usize) != '0' {
            return false;
        }
    }
    true
}

fn continue_running(hash_found_mutex: &Arc<Mutex<bool>>) -> bool {
    let hash_found = hash_found_mutex.lock().unwrap();
    if *hash_found {
        return false;
    }

    true
}

fn set_found(hash_found_mutex: &Arc<Mutex<bool>>) {
    let mut hash_found = hash_found_mutex.lock().unwrap();
    *hash_found = true;
}

// Try to find a partial hash collision
// On success send an Ok signal to parent process
// with the hash and the number of times we tried.
// If another thread beats us to it, send an Err with
// the number of times we tried.
fn do_hashes(mut props: HashThreadProperties) {
    let mut tries = 0;
    let mut thread_n = props.thread_n;
    while continue_running(&props.hash_found_mutex) {
        tries += 1;
        let input = build_string(props.in_string.as_slice(), thread_n);
        let hash = find_hash(input.as_slice(), &mut props.hasher);
        if valid_hash(hash.as_slice(), props.num_zeros) {
            props.tx.send(Ok((hash, tries))).unwrap();
            set_found(&props.hash_found_mutex);
            return;
        }
        props.hasher.reset();
        thread_n += props.num_procs;
    }
    props.tx.send(Err(tries)).unwrap();
}

fn find_partial(in_str: &str) {
    let mut count: u64 = 0;
    let n: i64 = rand::random::<i64>();

    let num_zeros = 6;
    let num_procs = 6;
    let (tx, rx) = channel::<HashThreadResult>();
    let hash_found: Arc<Mutex<bool>> = Arc::new(Mutex::new(false));
    let in_string: Arc<String> = Arc::new(String::from_str(in_str));

    for i in 0..num_procs {
        let thread_props = HashThreadProperties {
            tx: tx.clone(),
            in_string: in_string.clone(),
            hash_found_mutex: hash_found.clone(),
            thread_n: n + i,
            hasher: Sha256::new(),
            num_zeros: num_zeros,
            num_procs: num_procs,
        };

        thread::spawn(move || {
            do_hashes(thread_props);
        });
    }

    let mut output_string: String = String::new();

    for _ in 0..num_procs {
        let result: HashThreadResult = rx.recv().unwrap();
        match result {
            Ok((hash, n)) => {
                output_string = hash;
                count += n;
            },
            Err(n) => count += n,
        };
    }

    println!("Found hash after {} tries.", count);
    println!("Input: {}", in_str);
    println!("Output: {}", output_string);

}

fn main() {
    let in_string: String = String::from_str("World domination.");

    find_partial(in_string.as_slice());
}
