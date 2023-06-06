use nmea::NmeaBaseSentence;
use clap::Parser;
use std::path::PathBuf;
use std::fs::File;
use std::io::{stdin, BufRead, BufReader};
use chrono::NaiveTime;

#[derive(Debug, Parser)]
struct Cli {
	filenames : Option<Vec<PathBuf>>,

	#[arg(long, short, value_delimiter=',')]
	include : Option<Vec<String>>,

	#[arg(long, short, value_delimiter=',')]
	exclude : Option<Vec<String>>,

	#[arg(long="start", short='S', default_value="00:00:00")]
	start_time : NaiveTime,

	#[arg(long="end", short='E', default_value="23:59:59.999")]
	end_time : NaiveTime,
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
	let mut current_time = NaiveTime::default();
	for line in reader.lines() {
		if let Ok(line) = line {
			let sentence = NmeaBaseSentence::from(&line);
			if !sentence.message_type().is_empty() {
				let mut new_time = None;
				match sentence.message_type().as_str() {
					"BWC" => new_time = sentence.get_time(0),
					"BWR" => new_time = sentence.get_time(0),
					"GBS" => new_time = sentence.get_time(0),
					"GGA" => new_time = sentence.get_time(0),
					"GLL" => new_time = sentence.get_time(4),
					"GNS" => new_time = sentence.get_time(0),
					"GRS" => new_time = sentence.get_time(0),
					"GST" => new_time = sentence.get_time(0),
					"GXA" => new_time = sentence.get_time(0),
					"RLM" => new_time = sentence.get_time(1),
					"RMC" => new_time = sentence.get_time(0),
					"TLL" => new_time = sentence.get_time(6),
					"TRF" => new_time = sentence.get_time(0),
					"TTM" => new_time = sentence.get_time(13),
					"ZDA" => new_time = sentence.get_time(0),
					"ZFO" => new_time = sentence.get_time(0),
					"ZTG" => new_time = sentence.get_time(0),
					_ => {},
				}
				if let Some(new_time) = new_time {
					current_time = new_time;
				}
				println!("{:?} -- {:?} -- {:?}", &cli.start_time, &current_time, &cli.end_time);
				
				if current_time >= cli.start_time && current_time <= cli.end_time {
					if is_in(&cli.include, &sentence.message_type()) {
						pass_through(&line);
					} else if cli.exclude.is_some() && !is_in(&cli.exclude, &sentence.message_type()) {
						pass_through(&line);
					}
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
