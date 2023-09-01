use std::sync::{Arc, Mutex};
use std::thread;
use std::thread::available_parallelism;

use clap::Parser;
use sha256::digest;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Number of zeros at the end of the hash
    #[arg(short = 'N')]
    zero_occurrences: u8,

    /// How many numbers with matching hashes should be found
    #[arg(short = 'F')]
    results_number: u8,
}

fn main() {
    let args = Args::parse();

    let cpu_cores_num = usize::from(available_parallelism().unwrap());
    // println!("CPU cores: {:?}", cpu_cores_num);

    let shared_next_number = Arc::new(Mutex::new(1u128));
    let shared_numbers_found = Arc::new(Mutex::new(0));
    let mut threads = vec![];
    for _ in 0..cpu_cores_num {
        // Create 1 thread for each CPU core
        let local_shared_next_number = shared_next_number.clone();
        let local_shared_numbers_found = shared_numbers_found.clone();
        let thrd = thread::spawn(move || {
            let mut number_to_process: u128;
            let mut shasum;
            let zeros_str = "0".repeat(usize::from(args.zero_occurrences));
            loop {
                {
                    let numbers_found_val = local_shared_numbers_found.lock().unwrap();
                    if *numbers_found_val == args.results_number {
                        break;
                    }
                }
                {
                    let mut next_number = local_shared_next_number.lock().unwrap();
                    number_to_process = *next_number;
                    *next_number += 1;
                }
                shasum = digest(format!("{}", number_to_process));
                if shasum.ends_with(&zeros_str) {
                    {
                        let mut numbers_found_val = local_shared_numbers_found.lock().unwrap();
                        if *numbers_found_val == args.results_number {
                            break; // No need to output this found number and its hash
                        } else {
                            *numbers_found_val += 1;
                            println!("{:?}, {:?}", number_to_process, shasum);
                        }
                    }
                }
            }
        });
        threads.push(thrd);
    }

    for thrd in threads {
        thrd.join().unwrap();
    }
}
