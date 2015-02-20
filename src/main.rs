#![feature(rand,core,rustc_private,collections)]
extern crate crypto;
extern crate serialize;

use std::rand;
use std::mem::transmute;
use serialize::hex::ToHex;

use crypto::digest::Digest;
use crypto::sha2::Sha256;

fn build_string(in_string: &str, n: i64) -> String {
    let mut header = String::from_str(in_string);
    let n_bytes: [u8; 8] = unsafe { transmute(n) };
    let n_hex = n_bytes.to_hex();

    header.push(':');
    header.push_str(n_hex.as_slice());

    header
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
    let mut count: u32 = 0;
    let mut n: i64 = rand::random::<i64>();
    let num_zeros = 5;

    loop {
        count += 1;
        let input = build_string(in_string, n);
        let hash = find_hash(input.as_slice());
        if valid_hash(hash.as_slice(), num_zeros) {
            println!("Found hash after {} tries:", count);
            println!("Input: {}", input);
            println!("Hash: {}", hash);
            return;
        }
        n += 1;
    }
}

fn main() {
    let in_string: String = String::from_str("Hello there.");

    find_partial(in_string.as_slice());
}
