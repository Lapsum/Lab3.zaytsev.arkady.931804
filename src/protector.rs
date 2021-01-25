use rand::{thread_rng, Rng};
use std::cmp::min;

pub fn get_session_key() -> String {
    let mut res = vec![b'0';10];
    for i in 0..10 {
        res[i] += thread_rng().gen_range(0..10);
    }
    String::from_utf8(res).unwrap()
}

pub fn get_hash_key() -> String {
    let mut res = vec![b'1';5];
    for i in 0..5 {
        res[i] += thread_rng().gen_range(0..7);
    }
    String::from_utf8(res).unwrap()
}

pub struct SessionProtector{
    hash: String
}

impl SessionProtector {
    pub fn new(hash :String) -> Self {
        if hash.is_empty() || hash.chars().any(|ch| !ch.is_digit(10)) {
            panic!("Invalid hash");
        }
        SessionProtector{ hash }
    }
    pub fn next_session_key(&self, session_key: String) -> String {
        let mut res: u64 = 0;
        for ch in self.hash.chars() {
            res += Self::calc_hash(&session_key, ch.to_digit(10).unwrap());
        }
        let res = res.to_string();
        let res = "0000000000".to_string() + &res[..min(res.len(), 10)];
        res[res.len()-10..].to_string()
    }
    fn calc_hash(session_key: &String, b: u32) -> u64 {
        match b {
            1 => {
                let x: u64 = session_key[0..5].parse().unwrap();
                let x = x % 97;
                let x = "00".to_string() + x.to_string().as_str();
                x[x.len()-2..].parse().unwrap()
            },
            2 => {
                let mut res = String::with_capacity(session_key.capacity());
                for ch in session_key.chars().rev() {
                    res.push(ch);
                }
                res.parse().unwrap()
            },
            3 => {
                let mut res = String::with_capacity(session_key.capacity());
                res += &session_key[5..];
                res += &session_key[0..5];
                res.parse().unwrap()
            },
            4 => {
                let mut res = 0u64;
                for ch in session_key[1..9].chars() {
                    res += ch.to_digit(10).unwrap() as u64 + 41;
                }
                res
            },
            5 => {
                let mut res = 0u64;
                for b in session_key.bytes() {
                    let ch = char::from(b ^ 43);
                    if ch.is_digit(10) {
                        res += ch.to_digit(10).unwrap() as u64;
                    } else {
                        res += ch as u64
                    }
                }
                res
            }
            _ => session_key.parse::<u64>().unwrap() + b as u64
        }
    }
}