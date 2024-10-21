use clap::Parser;
use std::path::PathBuf;
use std::time::Duration;
use chrono::Local;
use std::{thread, time};
use rodio::{Decoder, OutputStream, Sink};
use std::fs::File;
use std::io::{BufReader, Write};
use termion::{clear, cursor, terminal_size};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to the sound file
    #[arg(short, long)]
    sound: PathBuf,

    /// Duration of the timer (e.g., 10m, 1h30m)
    duration: String,
}

fn main() {
    let args = Args::parse();
    
    let duration = parse_duration(&args.duration).expect("Invalid duration format");
    let start_time = Local::now();
    let end_time = start_time + chrono::Duration::from_std(duration).unwrap();

    println!("Timer set for {} at {}", args.duration, end_time.format("%H:%M:%S"));
    thread::sleep(time::Duration::from_secs(2));

    let mut stdout = std::io::stdout();

    loop {
        let now = Local::now();
        if now >= end_time {
            break;
        }

        let remaining = end_time - now;
        let (width, height) = terminal_size().unwrap();
        let time_str = format!("{:02}:{:02}:{:02}", 
                               remaining.num_hours(), 
                               remaining.num_minutes() % 60, 
                               remaining.num_seconds() % 60);
        
        write!(stdout, "{}{}", clear::All, cursor::Goto(1, 1)).unwrap();
        
        for y in 1..=height {
            for x in 1..=width {
                if y == height / 2 && x > (width - time_str.len() as u16) / 2 && x <= (width + time_str.len() as u16) / 2 {
                    let char_index = (x - (width - time_str.len() as u16) / 2 - 1) as usize;
                    write!(stdout, "{}", time_str.chars().nth(char_index).unwrap()).unwrap();
                } else {
                    write!(stdout, " ").unwrap();
                }
            }
            if y < height {
                write!(stdout, "\r\n").unwrap();
            }
        }
        
        stdout.flush().unwrap();
        thread::sleep(time::Duration::from_secs(1));
    }

    write!(stdout, "{}{}\nTime's up! Playing sound...\n", clear::All, cursor::Goto(1, 1)).unwrap();
    play_sound(&args.sound);
}

fn parse_duration(s: &str) -> Result<Duration, String> {
    let mut total_seconds = 0;
    let mut current_number = String::new();

    for c in s.chars() {
        if c.is_digit(10) {
            current_number.push(c);
        } else {
            let number = current_number.parse::<u64>().map_err(|_| "Invalid number")?;
            current_number.clear();

            match c {
                'h' => total_seconds += number * 3600,
                'm' => total_seconds += number * 60,
                's' => total_seconds += number,
                _ => return Err(format!("Invalid unit: {}", c)),
            }
        }
    }

    if !current_number.is_empty() {
        return Err("Number without unit at the end".to_string());
    }

    Ok(Duration::from_secs(total_seconds))
}

fn play_sound(path: &PathBuf) {
    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let sink = Sink::try_new(&stream_handle).unwrap();

    let file = BufReader::new(File::open(path).unwrap());
    let source = Decoder::new(file).unwrap();
    sink.append(source);

    sink.sleep_until_end();
}
