#![feature(core,collections)]
extern crate rand;
extern crate crypto;
extern crate "rustc-serialize" as rustc_serialize;

use std::mem::transmute;
use std::thread;
use std::sync::mpsc::channel;
use std::sync::{Arc,Mutex};

use rustc_serialize::hex::ToHex;

use crypto::digest::Digest;
use crypto::sha2::Sha256;

type NumTries = u64;
type ResultStr = String;
type HashThreadResult = Result<(ResultStr, NumTries), NumTries>;

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

fn find_partial(in_str: &str) {
    let mut count: u64 = 0;
    let n: i64 = rand::random::<i64>();

    let num_zeros = 6;
    let num_procs = 6;
    let (tx, rx) = channel::<HashThreadResult>();
    let hash_found: Arc<Mutex<bool>> = Arc::new(Mutex::new(false));
    let in_string: Arc<String> = Arc::new(String::from_str(in_str));

    for i in 0..num_procs {
        let tx = tx.clone();
        let in_string = in_string.clone();
        let hash_found_mutex = hash_found.clone();
        let mut thread_n = n + i;
        let mut hasher = Sha256::new();
        let mut tries: u64 = 0;

        thread::spawn(move || {
            loop {
                if !continue_running(&hash_found_mutex) {
                    tx.send(Err(tries)).unwrap();
                    return;
                }
                tries += 1;
                let input = build_string((*in_string).as_slice(), thread_n);
                let hash = find_hash(input.as_slice(), &mut hasher);
                if valid_hash(hash.as_slice(), num_zeros) {
                    tx.send(Ok((hash, tries))).unwrap();
                    set_found(&hash_found_mutex);
                    return;
                }
                hasher.reset();
                thread_n += num_procs;
            }
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
