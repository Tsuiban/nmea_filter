use nmea::NmeaBaseSentence;
use clap::Parser;
use std::path::PathBuf;
use std::fs::File;
use std::io::{stdin, BufRead, BufReader};

#[derive(Debug, Parser)]
struct Cli {
	filenames : Option<Vec<PathBuf>>,

	#[arg(long, short, value_delimiter=',')]
	include : Option<Vec<String>>,

	#[arg(long, short, value_delimiter=',')]
	exclude : Option<Vec<String>>,
}
	  
fn main() {
	let cli = Cli::parse();
	if cli.filenames.is_some() {
		for f in &cli.filenames.clone().unwrap() {
			match File::open(f) {
				Ok(f) => { process_file(&mut BufReader::new(f), &cli); }
				Err(e) => { eprintln!("{}", e); }
			}
		}
	} else {
		process_file(&mut BufReader::new(stdin()), &cli);
	}
}

fn process_file(reader : &mut dyn BufRead, cli : &Cli) {
	for line in reader.lines() {
		if let Ok(line) = line {
			let sentence = NmeaBaseSentence::from(&line);
			if !sentence.message_type().is_empty() {
				if is_in(&cli.include, &sentence.message_type()) {
					pass_through(&line);
				} else if cli.exclude.is_some() && !is_in(&cli.exclude, &sentence.message_type()) {
					pass_through(&line);
				}
			}
		}
	}
}

fn pass_through(line : &String) {
	println!("{}", line);
}

fn is_in(list : &Option<Vec<String>>, msg_type : &String) -> bool {
	match list {
		Some(l) => {
			for item in l {
				let compare = format!("{}", item);
				if compare == *msg_type {
					return true;
				}
			}
			return false;
		},
		None => return false,
	}
}
