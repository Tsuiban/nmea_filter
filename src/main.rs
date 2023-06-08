use nmea::NmeaBaseSentence;
use clap::Parser;
use std::path::PathBuf;
use std::fs::File;
use std::io::{stdin, BufRead, BufReader};
use chrono::NaiveTime;

/// This program filters an NMEA-0183 stream.  It filters out unwanted NMEA-0183 sentences
/// by sentence name or time or else merely removes invalid sentences.
///
/// For both --include and --exclude, the argument is a LIST of sentences.   For example
/// --include GLL,GGA,ZDA or --exclude XDR,HDG
///
/// --start and --end both take a time in the form of hh:mm:ss.sss.
#[derive(Debug, Parser)]
struct Cli {
	filenames : Option<Vec<PathBuf>>,

	/// Pass through all sentences that occur on or before this time
	#[arg(long="end", short='E', default_value="23:59:59.999")]
	end_time : NaiveTime,

	/// Pass through these sentences
	#[arg(long, short='m', value_delimiter=',')]
	include_messages: Option<Vec<String>>,

	/// Pass through all sentences that are NOT these sentences.
	#[arg(long, short='M', value_delimiter=',')]
	exclude_messages: Option<Vec<String>>,

	/// Pass through all sentences that occur on or after this time
	#[arg(long="start", short='S', default_value="00:00:00")]
	start_time : NaiveTime,

	/// Pass through sentences only from these talkers
	#[arg(long, short='t', value_delimiter=',')]
	include_talkers : Option<Vec<String>>,

	/// Excluse sentences from these talkers
	#[arg(short='T', long, value_delimiter=',')]
	exclude_talkers : Option<Vec<String>>,

	/// List the unique messages that are in the file
	#[arg(short='u', long)]
	list_unique_messages : bool,

	/// List the unique talkers that are in the file
	#[arg(short='U', long)]
	list_unique_talkers : bool,

	/// Pass through all valid sentences
	#[arg(long,short='V')]
	valid : bool,
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
	let mut unique_messages : Vec<String> = Vec::new();
	let mut unique_talkers : Vec<String> = Vec::new();
	let mut current_time = NaiveTime::default();
	for line in reader.lines() {
		if let Ok(line) = line {
			let sentence = NmeaBaseSentence::from(&line);
			let talker = sentence.sender();
			let message_type = sentence.message_type();
			if cli.list_unique_talkers && !unique_talkers.contains(&talker) {
				unique_talkers.push(talker.clone());
			}
			if cli.list_unique_messages && !unique_messages.contains(&message_type) {
				unique_messages.push(message_type.clone());
			}
			if sentence.is_valid() {
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
				//println!("{:?} -- {:?} -- {:?}", &cli.start_time, &current_time, &cli.end_time);
				
				if current_time >= cli.start_time && current_time <= cli.end_time {
					if is_in(&cli.include_messages, &sentence.message_type()) {
							pass_through(&cli, &sentence.sender(), &line);
					} else if !cli.exclude_messages.is_some() || !is_in(&cli.exclude_messages, &sentence.message_type()) {
						pass_through(&cli, &sentence.sender(), &line);
					} else if cli.valid {
						pass_through(&cli, &sentence.sender(), &line);
					}
				}
			}
		}
	}
	if cli.list_unique_talkers {
		unique_talkers.sort();
		println!("Unique Talkers: {:?}", unique_talkers)
	}

	if cli.list_unique_messages {
		unique_messages.sort();
		println!("Unique messages: {:?}", unique_messages)
	}
}

fn pass_through(cli : &Cli, sender : &String, line : &String) {
	if (
		cli.exclude_talkers.is_some() &&
			!cli
				.exclude_talkers
				.as_ref()
				.unwrap()
				.contains(sender)
	) || (
		cli.include_talkers.is_some() &&
			cli
				.include_talkers
				.as_ref()
				.unwrap()
				.contains(sender)
	) {
		println!("{}", line);
	}
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
