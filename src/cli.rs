use std::env;

fn usage() {
    println!("Usage: craban [-d <directory>]");
    println!("       craban [--help] [-h]");
    println!("Try `craban --help' for more information.");
}

pub fn parse_args() -> Result<Option<String>, ()> {
    let mut args = env::args();
    let directory = args.nth(1).expect("directory path flag is provided");

    match directory.as_str() {
        "-d" => {
            let directory_path = args.next().expect("path to directory is provided");
            Ok(Some(directory_path))
        }
        "-h" => {
            usage();
            Ok(None)
        }
        "--h" => {
            usage();
            Ok(None)
        }
        _ => {
            usage();
            eprintln!("ERROR: unknown command {directory}");
            Err(())
        }
    }
}
