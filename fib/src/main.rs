use num_bigint::BigUint;
use serde::{Serialize, Deserialize};
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::thread;

#[derive(Serialize, Deserialize)]
struct FibonacciState {
    i: u64,
    a: BigUint,
    b: BigUint,
}

fn read_state_from_file() -> Option<FibonacciState> {
    let mut file = match File::open("fibonacci_state.json") {
        Ok(file) => file,
        Err(_) => return None,
    };

    let mut contents = String::new();
    file.read_to_string(&mut contents).expect("Failed to read file");

    match serde_json::from_str(&contents) {
        Ok(state) => Some(state),
        Err(_) => None,
    }
}

fn save_state_to_file(state: &FibonacciState) {
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open("fibonacci_state.json")
        .expect("Failed to open file");

    let json = serde_json::to_string_pretty(state).expect("Failed to serialize state");
    file.write_all(json.as_bytes()).expect("Failed to write to file");
}

fn fibonacci(n: u64) -> BigUint {
    if n == 0 {
        return BigUint::from(0u8);
    }
    let (result, _) = fib(n);
    result
}

fn fib(n: u64) -> (BigUint, BigUint) {
    let mut state = match read_state_from_file() {
        Some(state) => state,
        None => FibonacciState {
            i: 0,
            a: BigUint::from(0u8),
            b: BigUint::from(1u8),
        },
    };

    if n == state.i {
        (state.a, state.b)
    } else if n == 0 {
        (BigUint::from(0u8), BigUint::from(1u8))
    } else {
        
        let (a, b) = fib(n / 2);
        let a_clone = a.clone();
        let b_clone = b.clone();

        let handle_c = thread::spawn(move || {
            let c = &a_clone * &(&b_clone * BigUint::from(2u8) - &a_clone);
            c
        });

        let handle_d = thread::spawn(move || {
            let d = &a * &a + &b * &b;
            d
        });

        let c = handle_c.join().unwrap();
        let d = handle_d.join().unwrap();

        println!("fib({:012}) has been calculated...", n);
        state.i = n.clone();
        state.a = c.clone();
        state.b = d.clone();
        save_state_to_file(&state);
        if n % 2 == 0 {
            (c, d)
        } else {
            (d.clone(), &c + &d)
        }
    }
}

fn main() {
    let n: u64 = 300000000000;
    let result = fibonacci(n);
    println!("Le {}-ième nombre de Fibonacci est : {}", n, result);
}
