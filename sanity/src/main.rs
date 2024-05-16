#![deny(clippy::panic)]

#[cfg(feature = "minimal_benchmark")]
use std::time::Instant;
use std::{fs, iter::zip, process::ExitCode};

use clap::Parser;
use sanitise::sanitise_string;

#[derive(Debug, Parser)]
struct Args {
    /// The file to process
    file_name: String,

    /// The root of the output file
    output_file_name: String,
}

fn main() -> ExitCode {
    println!("Starting...");
    #[cfg(feature = "minimal_benchmark")]
    let start = Instant::now();

    let args = Args::parse();

    println!("Getting CSV contents...");
    let file_contents = match fs::read_to_string(args.file_name) {
        Ok(contents) => contents,
        Err(_) => {
            eprintln!("Failed to read file");
            return ExitCode::FAILURE;
        }
    };

    #[cfg(feature = "benchmark")]
    println!("Got CSV contents: {}ms", start.elapsed().as_millis());
    #[cfg(feature = "benchmark")]
    let before_sanitise = Instant::now();

    println!("Processing CSV...");

    // Type hints are not necessary for this to compile,
    // but rust-analyzer can't detect the type,
    // so this allows it to insert inline type hints later
    #[allow(clippy::type_complexity)]
    let result: Vec<((Vec<i64>, Vec<i64>, Vec<bool>), (Vec<i64>, Vec<i64>))> =
        match sanitise_string!(include_str!("sanity.yaml"), &file_contents) {
            Ok(v) => v,
            Err((message, line)) => {
                eprintln!("Line {line}: {message}");
                return ExitCode::FAILURE;
            }
        };

    #[cfg(feature = "benchmark")]
    println!("Processed CSV: {}ms", before_sanitise.elapsed().as_millis());
    #[cfg(feature = "benchmark")]
    let before_file_writes = Instant::now();

    println!("Writing to output files...");
    for (i, ((time_millis, pulse_raw, movement), (time_mins, pulse_average))) in
        result.into_iter().enumerate()
    {
        #[cfg(feature = "benchmark")]
        let before_file_generate = Instant::now();

        let file_name_base = args.output_file_name.to_owned() + &format!("_{}", i + 1);
        let file_name_raw = file_name_base.to_owned() + "_raw.csv";
        let file_name_processed = file_name_base + "_processed.csv";

        // Sizes chosen to avoid reallocation
        let mut buf_raw = String::with_capacity(time_millis.len() * 20);
        let mut buf_processed = String::with_capacity(time_millis.len() * 10);

        buf_raw.push_str("time,pulse,movement\n");
        buf_processed.push_str("time,pulse\n");

        for ((time_millis, pulse), movement) in zip(zip(time_millis, pulse_raw), movement) {
            buf_raw.push_str(&format!("{time_millis},{pulse},{movement}\n"));
        }

        for (time_mins, pulse) in zip(time_mins, pulse_average) {
            buf_processed.push_str(&format!("{time_mins},{pulse}\n"));
        }

        #[cfg(feature = "benchmark")]
        println!(
            "Processed data for file {}: {}ms",
            i + 1,
            before_file_generate.elapsed().as_millis()
        );
        #[cfg(feature = "benchmark")]
        let before_file_write = Instant::now();

        let _ = fs::write(file_name_raw, buf_raw);
        let _ = fs::write(file_name_processed, buf_processed);

        #[cfg(feature = "benchmark")]
        println!(
            "Wrote to file {}: {}ms",
            i + 1,
            before_file_write.elapsed().as_millis()
        );
    }

    #[cfg(feature = "benchmark")]
    println!(
        "Wrote to output files: {}ms",
        before_file_writes.elapsed().as_millis()
    );

    println!("Done");
    #[cfg(feature = "minimal_benchmark")]
    println!("Total time taken: {}ms", start.elapsed().as_millis());

    ExitCode::SUCCESS
}
