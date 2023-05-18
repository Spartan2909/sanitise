use std::{fs, iter::zip, process::ExitCode, time::Instant};

use clap::Parser;
use sanitise::sanitise;

#[derive(Debug, Parser)]
struct Args {
    /// The file to process
    file_name: String,

    /// The root of the output file
    output_file_name: String,
}

fn main() -> ExitCode {
    println!("Starting...");
    let start = Instant::now();

    let args = Args::parse();

    println!("Getting CSV contents...");
    let file_contents = fs::read_to_string(args.file_name).unwrap();
    println!("Got CSV contents: {}ms", start.elapsed().as_millis());
    let before_sanitise = Instant::now();

    println!("Processing CSV...");
    let result = match sanitise!(include_str!("sanity.yaml"), file_contents) {
        Ok(v) => v,
        Err((message, line)) => {
            eprintln!("Line {line}: {message}");
            return ExitCode::FAILURE;
        }
    };

    println!("Processed CSV: {}ms", before_sanitise.elapsed().as_millis());
    let before_file_writes = Instant::now();

    println!("Writing to output files...");
    for (i, ((time_millis, pulse, movement), (time_mins,))) in result.into_iter().enumerate() {
        let before_file_write = Instant::now();

        let file_name_base = args.output_file_name.to_owned() + &format!("_{}", i + 1);
        let file_name_raw = file_name_base.to_owned() + "_raw.csv";
        let file_name_processed = file_name_base + "_processed.csv";

        let mut buf_raw = "time,pulse,movement\n".to_owned();
        let mut buf_processed = "time,pulse\n".to_owned();
        for (((time_millis, pulse), movement), time_mins) in
            zip(zip(zip(time_millis, pulse), movement), time_mins)
        {
            buf_raw.extend(format!("{time_millis},{pulse},{movement}\n").chars());
            buf_processed.extend(format!("{time_mins},{pulse}\n").chars());
        }

        let _ = fs::write(file_name_raw, buf_raw);
        let _ = fs::write(file_name_processed, buf_processed);

        println!("Wrote to file {}: {}ms", i + 1, before_file_write.elapsed().as_millis());
    }

    println!(
        "Wrote to output files: {}ms",
        before_file_writes.elapsed().as_millis()
    );

    println!("Done");
    println!("Total time taken: {}ms", start.elapsed().as_millis());

    ExitCode::SUCCESS
}
