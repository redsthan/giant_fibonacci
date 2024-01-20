use indicatif::{ProgressBar, ProgressStyle};
use num_bigint::BigUint;
use num_traits::{One, Zero};
use serde::{Serialize, Deserialize};
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::mem::replace;

#[derive(Serialize, Deserialize)]
struct FibonacciState {
    i: u64,
    a: BigUint,
    b: BigUint,
}

fn fibonacci_with_state(n: u64, progress_bar: &ProgressBar) -> BigUint {
    let mut state = match read_state_from_file() {
        Some(state) => state,
        None => FibonacciState {
            i: 1,
            a: BigUint::one(),
            b: BigUint::one(),
        },
    };
    progress_bar.inc(state.i);

    for i in state.i+1..n {
        let mut result = &state.a + &state.b;
        state.i = i;
        state.a = replace(&mut state.b, replace(&mut result, BigUint::zero()));
        if i%100000 == 0 {
            save_state_to_file(&state);
        }
        if i%1000 == 0 {
            progress_bar.inc(1000);
        }
    }

    save_state_to_file(&state); // Save final state
    progress_bar.finish_and_clear();
    state.b
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

fn main() {
    let n = 300000000000;

    let progress_bar = ProgressBar::new(n);
    progress_bar.set_style(
        ProgressStyle::default_bar()
            .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos}/{len} ({eta})").expect("REASON")
            .progress_chars("##-"),
    );

    let result = fibonacci_with_state(n, &progress_bar);
    println!("Fibonacci({}) = {}", n, result);
}
