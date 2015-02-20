#![feature(rand,core,rustc_private,collections,io)]
extern crate crypto;
extern crate serialize;

use std::old_io;
use std::rand;
use std::mem::transmute;
use serialize::hex::ToHex;

use crypto::digest::Digest;
use crypto::sha2::Sha256;

fn build_n_string(n: i64) -> String {
    let n_bytes: [u8; 8] = unsafe { transmute(n) };

    n_bytes.to_hex()
}

fn hash_partial_header(input: &str) -> Sha256 {
    let mut hasher = Sha256::new();
    hasher.input_str(input);

    hasher
}

fn find_hash(input: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.input_str(input);
    hasher.result_str()
}

fn valid_hash(hash: &str, num_zeros: u32) -> bool {
    for x in 0..num_zeros {
        if hash.char_at(x as usize) != '0' {
            return false;
        }
    }
    true
}

fn find_partial(in_string: &str) {
    let mut count: u64 = 0;
    let mut n: i64 = rand::random::<i64>();
    let num_zeros = 5;
    let hasher = hash_partial_header(in_string);
    loop {
        count += 1;
        let mut hasher_clone = hasher;
        let n_string = build_n_string(n);
        hasher_clone.input_str(":");
        hasher_clone.input_str(n_string.as_slice());
        let hash = hasher_clone.result_str();
        if valid_hash(hash.as_slice(), num_zeros) {
            let mut full_input = String::from_str(in_string);
            full_input.push(':');
            full_input.push_str(n_string.as_slice());
            println!("Found hash after {} tries.", count);
            println!("Input: {}", full_input);
            println!("Hash: {}", hash);
            return;
        }
        n += 1;
    }
}

fn main() {
    print!("Please enter an input string: ");
    let in_string: String = old_io::stdin().read_line().ok().expect("Failed to read line.");

    find_partial(in_string.trim().as_slice());
}
