use std::{fs, iter::zip, time::Instant};

use clap::Parser;
use sanitise::sanitise;

#[derive(Debug, Parser)]
struct Args {
    /// The file to process
    file_name: String,

    /// The root of the output file
    output_file_name: String,
}

fn main() {
    let start = Instant::now();

    let args = Args::parse();
    let file_contents = fs::read_to_string(args.file_name).unwrap();
    let result = match sanitise!(include_str!("sanity.yaml"), file_contents) {
        Ok(v) => v,
        Err((message, line)) => panic!("{message} at line {line}"),
    };
    for (i, ((time_millis, pulse, movement), (time_secs,))) in result.into_iter().enumerate() {
        let file_name_base = args.output_file_name.to_owned() + &format!("_{}", i + 1);
        let file_name_raw = file_name_base.to_owned() + "_raw.csv";
        let file_name_processed = file_name_base + "_processed.csv";

        let mut buf_raw = "time,pulse,movement\n".to_owned();
        let mut buf_processed = "time,pulse\n".to_owned();
        for (((time_millis, pulse), movement), time_secs) in
            zip(zip(zip(time_millis, pulse), movement), time_secs)
        {
            buf_raw.extend(format!("{time_millis},{pulse},{movement}\n").chars());
            buf_processed.extend(format!("{time_secs},{pulse}\n").chars());
        }

        let _ = fs::write(file_name_raw, buf_raw);
        let _ = fs::write(file_name_processed, buf_processed);
    }

    println!("Time taken: {}ms", start.elapsed().as_millis());
}
