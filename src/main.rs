use chrono::NaiveTime;
use clap::Parser;
use nmea::NmeaBaseSentence;
use std::fs::File;
use std::io::{stdin, BufRead, BufReader};
use std::path::PathBuf;

/// This program filters an NMEA-0183 stream.  It filters out unwanted NMEA-0183 sentences
/// by sentence name or time or else merely removes invalid sentences.
///
/// For both --include and --exclude, the argument is a LIST of sentences.   For example
/// --include GLL,GGA,ZDA or --exclude XDR,HDG
///
/// --start and --end both take a time in the form of hh:mm:ss.sss.
#[derive(Debug, Parser)]
struct Cli {
    filenames: Option<Vec<PathBuf>>,

    /// Pass through all sentences that occur on or before this time
    #[arg(long = "end", short = 'E', default_value = "23:59:59.999")]
    end_time: NaiveTime,

    /// Pass through these sentences
    #[arg(long, short = 'm', value_delimiter = ',')]
    include_messages: Option<Vec<String>>,

    /// Pass through all sentences that are NOT these sentences.
    #[arg(long, short = 'M', value_delimiter = ',')]
    exclude_messages: Option<Vec<String>>,

    /// Pass through all sentences that occur on or after this time
    #[arg(long = "start", short = 'S', default_value = "00:00:00")]
    start_time: NaiveTime,

    /// Pass through sentences only from these talkers
    #[arg(long, short = 't', value_delimiter = ',')]
    include_talkers: Option<Vec<String>>,

    /// Excluse sentences from these talkers
    #[arg(short = 'T', long, value_delimiter = ',')]
    exclude_talkers: Option<Vec<String>>,

    /// List the unique messages that are in the file
    #[arg(short = 'u', long)]
    list_unique_messages: bool,

    /// List the unique talkers that are in the file
    #[arg(short = 'U', long)]
    list_unique_talkers: bool,

    /// Pass through all valid sentences
    #[arg(long, short = 'V')]
    valid: bool,
}

struct Uniques {
    talkers: Vec<String>,
    messages: Vec<String>,
}

impl Uniques {
    pub fn new() -> Uniques {
        Uniques {
            talkers: Vec::new(),
            messages: Vec::new(),
        }
    }
}

fn main() {
    let cli = Cli::parse();
    let mut uniques = Uniques::new();
    /*
        if cli.include_messages.is_some() {
            let i =
                cli
                .include_messages
                .as_ref()
                .unwrap();
            eprintln!("Include only {}", display_vec(&i));
        }

        if cli.exclude_messages.is_some() {
            let e =
                cli
                .exclude_messages
                .as_ref()
                .unwrap();
            eprintln!("Exclude {}", display_vec(&e));
        }

        if cli.include_talkers.is_some() {
            let i =
                cli
                .include_talkers
                .as_ref()
                .unwrap();
            eprintln!("Include {}", display_vec(&i));
        }

        if cli.exclude_talkers.is_some() {
            let e =
                cli
                .exclude_talkers
                .as_ref()
                .unwrap();
            eprintln!("Exclude {}", display_vec(&e));
        }
    */
    if cli.filenames.is_some() {
        for f in &cli.filenames.clone().unwrap() {
            match File::open(f) {
                Ok(f) => {
                    process_file(&mut BufReader::new(f), &cli, &mut uniques);
                }
                Err(e) => {
                    eprintln!("{}", e);
                }
            }
        }
    } else {
        process_file(&mut BufReader::new(stdin()), &cli, &mut uniques);
    }
}

fn display_vec<T>(v: &Vec<T>) -> String
where
    T: std::fmt::Display,
{
    let mut ret_string = String::new();
    for item in v {
        if !ret_string.is_empty() {
            ret_string.push(',');
        };
        ret_string.push_str(&(*item).to_string());
    }
    ret_string
}

fn process_file(reader: &mut dyn BufRead, cli: &Cli, uniques: &mut Uniques) {
    let mut current_time = NaiveTime::default();
    for line in reader.lines() {
        if let Ok(line) = line {
            let sentence = NmeaBaseSentence::from(&line);
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
                    _ => {}
                }
                if let Some(new_time) = new_time {
                    current_time = new_time;
                }

                if current_time >= cli.start_time && current_time <= cli.end_time {
                    let message_type = sentence.message_type();
                    if (cli.include_messages.is_some()
                        && cli
                            .include_messages
                            .as_ref()
                            .unwrap()
                            .contains(&message_type))
                        || (cli.exclude_messages.is_some()
                            && !cli
                                .exclude_messages
                                .as_ref()
                                .unwrap()
                                .contains(&message_type))
                        || (cli.include_messages.is_none() && cli.exclude_messages.is_none())
                    {
                        pass_through(&cli, &sentence, uniques);
                    }
                }
            }
        }
    }
    if cli.list_unique_talkers {
        (*uniques).talkers.sort();
        println!("{}", display_vec(&uniques.talkers))
    }

    if cli.list_unique_messages {
        uniques.messages.sort();
        println!("{}", display_vec(&uniques.messages))
    }
}

fn pass_through(cli: &Cli, sentence: &NmeaBaseSentence, uniques: &mut Uniques) {
    let sender = &sentence.sender();
    if (cli.exclude_talkers.is_some() && !cli.exclude_talkers.as_ref().unwrap().contains(sender))
        || (cli.include_talkers.is_some() && cli.include_talkers.as_ref().unwrap().contains(sender))
        || (!cli.exclude_talkers.is_some() && !cli.include_talkers.is_some())
    {
        let talker = sentence.sender();
        let message_type = sentence.message_type();
        if cli.list_unique_talkers && !uniques.talkers.contains(&talker) && talker.ne("") {
            (*uniques).talkers.push(talker.clone());
        }
        if cli.list_unique_messages
            && !uniques.messages.contains(&message_type)
            && message_type.ne("")
        {
            uniques.messages.push(message_type.clone());
        }
        if !cli.list_unique_messages && !cli.list_unique_talkers {
            println!("{}", &sentence.original().unwrap());
        }
    }
}
