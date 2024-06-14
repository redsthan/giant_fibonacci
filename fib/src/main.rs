extern crate rayon;
extern crate num_bigint;
extern crate num_traits;
extern crate tokio;
extern crate warp;
extern crate num_cpus;

use rayon::prelude::*;
use num_bigint::BigUint;
use num_traits::{One, Zero};
use std::sync::{Arc, Mutex};
use warp::Filter;
use serde::Serialize;
use std::fs::File;
use rayon::ThreadPoolBuilder;

type Matrix = [[BigUint; 2]; 2];

#[derive(Serialize)]
struct Progress {
    index: u64,
    result: String,
    percentage: f64,
}

fn matrix_mult(a: &Matrix, b: &Matrix) -> Matrix {
    [
        [
            &a[0][0] * &b[0][0] + &a[0][1] * &b[1][0],
            &a[0][0] * &b[0][1] + &a[0][1] * &b[1][1],
        ],
        [
            &a[1][0] * &b[0][0] + &a[1][1] * &b[1][0],
            &a[1][0] * &b[0][1] + &a[1][1] * &b[1][1],
        ],
    ]
}

fn matrix_pow(matrix: &Matrix, mut exp: u64) -> Matrix {
    let identity: Matrix = [
        [BigUint::one(), BigUint::zero()],
        [BigUint::zero(), BigUint::one()]
    ];
    let mut result = identity;
    let mut base = matrix.clone();

    while exp > 0 {
        if exp % 2 == 1 {
            result = matrix_mult(&result, &base);
        }
        base = matrix_mult(&base, &base);
        exp /= 2;
    }

    result
}

fn fibonacci(n: u64) -> BigUint {
    if n == 0 {
        return BigUint::zero();
    }
    if n == 1 {
        return BigUint::one();
    }

    let fib_matrix: Matrix = [
        [BigUint::one(), BigUint::one()],
        [BigUint::one(), BigUint::zero()]
    ];
    let result_matrix = matrix_pow(&fib_matrix, n - 1);

    result_matrix[0][0].clone()
}

fn parallel_fibonacci(n: u64, threads: usize, progress: Arc<Mutex<Progress>>) -> BigUint {
    let n_per_thread = n / threads as u64;
    let remainders = n % threads as u64;
    let fib_matrix: Matrix = [
        [BigUint::one(), BigUint::one()],
        [BigUint::one(), BigUint::zero()]
    ];
    let results: Arc<Mutex<Vec<Matrix>>> = Arc::new(Mutex::new(vec![
        [
            [BigUint::one(), BigUint::zero()],
            [BigUint::zero(), BigUint::one()]
        ];
        threads
    ]));

    (0..threads).into_par_iter().for_each(|i| {
        let mut local_result = [
            [BigUint::one(), BigUint::zero()],
            [BigUint::zero(), BigUint::one()]
        ];
        let mut local_exp = n_per_thread;
        if i == 0 {
            local_exp += remainders;
        }
        let mut base = fib_matrix.clone();
        let total_exp = local_exp;
        while local_exp > 0 {
            if local_exp % 2 == 1 {
                local_result = matrix_mult(&local_result, &base);
            }
            base = matrix_mult(&base, &base);
            local_exp /= 2;

            // Update progress
            let mut prog = progress.lock().unwrap();
            prog.index = n;
            prog.result = local_result[0][0].to_string();
            prog.percentage = (total_exp - local_exp) as f64 / total_exp as f64 * 100.0;
        }
        results.lock().unwrap()[i] = local_result;
    });

    let final_result = results.lock().unwrap().iter().fold(
        [
            [BigUint::one(), BigUint::zero()],
            [BigUint::zero(), BigUint::one()]
        ],
        |acc, mat| matrix_mult(&acc, mat)
    );
    final_result[0][0].clone()
}

#[tokio::main]
async fn main() {
    // Set up rayon to use all available threads
    let num_cpus = num_cpus::get();
    ThreadPoolBuilder::new().num_threads(num_cpus).build_global().unwrap();

    let progress = Arc::new(Mutex::new(Progress {
        index: 0,
        result: String::new(),
        percentage: 0.0,
    }));
    
    let progress_filter = warp::any().map({
        let progress = progress.clone();
        move || {
            let progress = progress.lock().unwrap();
            warp::reply::json(&*progress)
        }
    });

    tokio::spawn(async move {
        let n = 299_999_999_999;
        let threads = num_cpus;
        let result = parallel_fibonacci(n, threads, progress.clone());
        
        // Write result to a file
        let progress = progress.lock().unwrap();
        let file = File::create("fibonacci_result.json").expect("Unable to create file");
        serde_json::to_writer(file, &*progress).expect("Unable to write data to file");

        println!("Fibonacci of {} is {}", n, result);
    });

    let routes = warp::path("progress")
        .and(progress_filter);

    warp::serve(routes)
        .run(([0, 0, 0, 0], 3030))
        .await;
}
