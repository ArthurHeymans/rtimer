use chrono::{Local, NaiveTime, Timelike};
use clap::Parser;
use notify_rust::Notification;
use rodio::{Decoder, OutputStream, Sink};
use std::io::Write;
use std::path::PathBuf;
use std::time::Duration;
use std::{str::FromStr, thread, time};
use termion::{clear, color, cursor, terminal_size};

fn create_ascii_clock(hours: u32, minutes: u32) -> Vec<String> {
    let mut clock = vec![
        "      12      ".to_string(),
        "   11     1   ".to_string(),
        " 10    |    2 ".to_string(),
        "9    --+--    3".to_string(),
        " 8     |    4 ".to_string(),
        "   7      5   ".to_string(),
        "      6       ".to_string(),
    ];

    // Calculate hand positions
    let hour_angle = (hours % 12) as f32 / 12.0 * 2.0 * std::f32::consts::PI;
    let minute_angle = minutes as f32 / 60.0 * 2.0 * std::f32::consts::PI;

    let hour_x = (2.0 * hour_angle.sin()).round() as i32;
    let hour_y = (-2.0 * hour_angle.cos()).round() as i32;
    let minute_x = (3.0 * minute_angle.sin()).round() as i32;
    let minute_y = (-3.0 * minute_angle.cos()).round() as i32;

    // Place hour hand
    if let Some(line) = clock.get_mut((3 + hour_y) as usize) {
        let mut chars: Vec<char> = line.chars().collect();
        if let Some(ch) = chars.get_mut((7 + hour_x) as usize) {
            *ch = 'H';
        }
        *line = chars.into_iter().collect();
    }

    // Place minute hand
    if let Some(line) = clock.get_mut((3 + minute_y) as usize) {
        let mut chars: Vec<char> = line.chars().collect();
        if let Some(ch) = chars.get_mut((7 + minute_x) as usize) {
            *ch = 'M';
        }
        *line = chars.into_iter().collect();
    }

    clock
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to the sound file
    #[arg(short, long, default_value = "rooster.mp3")]
    sound: PathBuf,

    /// Duration of the timer (e.g., 10m, 1h30m) or fixed time (e.g., 09:10:03)
    #[arg(value_parser = parse_time_or_duration)]
    time: TimeOrDuration,

    /// Custom notification message
    #[arg(short, long)]
    message: Option<String>,
}

#[derive(Debug, Clone)]
enum TimeOrDuration {
    Duration(Duration),
    FixedTime(NaiveTime),
}

fn parse_time_or_duration(s: &str) -> Result<TimeOrDuration, String> {
    if let Ok(duration) = parse_duration(s) {
        Ok(TimeOrDuration::Duration(duration))
    } else if let Ok(time) = NaiveTime::from_str(s) {
        Ok(TimeOrDuration::FixedTime(time))
    } else {
        Err(format!("Invalid time or duration format: {}", s))
    }
}

fn main() {
    let args = Args::parse();

    let (end_time, duration_str, start_time) = match args.time {
        TimeOrDuration::Duration(duration) => {
            let start_time = Local::now();
            let end_time = start_time + chrono::Duration::from_std(duration).unwrap();
            (end_time, format!("{:?}", duration), start_time)
        }
        TimeOrDuration::FixedTime(time) => {
            let now = Local::now();
            let mut end_time = now
                .date_naive()
                .and_time(time)
                .and_local_timezone(Local)
                .unwrap();
            if end_time <= now {
                end_time = end_time + chrono::Duration::days(1);
            }
            (end_time, time.format("%H:%M:%S").to_string(), now)
        }
    };

    println!("Timer set for {}", duration_str);
    println!("Start time: {}", start_time.format("%H:%M:%S"));
    println!("End time: {}", end_time.format("%H:%M:%S"));
    thread::sleep(time::Duration::from_secs(2));

    let mut stdout = std::io::stdout();

    loop {
        let now = Local::now();
        if now >= end_time {
            break;
        }

        let remaining = end_time - now;
        let elapsed = now - start_time;
        let (width, height) = terminal_size().unwrap();
        let remaining_elapsed_str = format!(
            "Remaining: {:02}:{:02}:{:02} | Elapsed: {:02}:{:02}:{:02}",
            remaining.num_hours(),
            remaining.num_minutes() % 60,
            remaining.num_seconds() % 60,
            elapsed.num_hours(),
            elapsed.num_minutes() % 60,
            elapsed.num_seconds() % 60
        );
        let start_end_str = format!(
            "Start time: {} | End time: {}",
            start_time.format("%H:%M:%S"),
            end_time.format("%H:%M:%S")
        );

        let ascii_clock = create_ascii_clock(now.hour(), now.minute());

        write!(stdout, "{}{}", clear::All, cursor::Goto(1, 1)).unwrap();

        let clock_start_y = (height - ascii_clock.len() as u16 - 3) / 2;
        let clock_start_x = (width - ascii_clock[0].len() as u16) / 2;

        for (i, line) in ascii_clock.iter().enumerate() {
            write!(
                stdout,
                "{}",
                cursor::Goto(clock_start_x, clock_start_y + i as u16)
            )
            .unwrap();
            write!(stdout, "{}", color::Fg(color::Blue)).unwrap();
            write!(stdout, "{}", line).unwrap();
            write!(stdout, "{}", color::Fg(color::Reset)).unwrap();
        }

        write!(
            stdout,
            "{}",
            cursor::Goto(
                (width - remaining_elapsed_str.len() as u16) / 2,
                clock_start_y + ascii_clock.len() as u16 + 1
            )
        )
        .unwrap();
        write!(stdout, "{}", color::Fg(color::Green)).unwrap();
        write!(stdout, "{}", remaining_elapsed_str).unwrap();
        write!(stdout, "{}", color::Fg(color::Reset)).unwrap();

        write!(
            stdout,
            "{}",
            cursor::Goto(
                (width - start_end_str.len() as u16) / 2,
                clock_start_y + ascii_clock.len() as u16 + 2
            )
        )
        .unwrap();
        write!(stdout, "{}", color::Fg(color::Green)).unwrap();
        write!(stdout, "{}", start_end_str).unwrap();
        write!(stdout, "{}", color::Fg(color::Reset)).unwrap();

        stdout.flush().unwrap();
        thread::sleep(time::Duration::from_secs(1));
    }

    write!(stdout, "{}{}\nTime's up!\n", clear::All, cursor::Goto(1, 1)).unwrap();

    // Send notification
    let notification_message = args
        .message
        .unwrap_or_else(|| "🐓 rtimer: time's up!".to_string());
    Notification::new()
        .summary("rtimer")
        .body(&notification_message)
        .show()
        .expect("Failed to send notification");

    // Play sound
    write!(stdout, "Playing sound...\n").unwrap();
    play_sound();
}

fn parse_duration(s: &str) -> Result<Duration, String> {
    let mut total_seconds = 0;
    let mut current_number = String::new();

    for c in s.chars() {
        if c.is_digit(10) {
            current_number.push(c);
        } else {
            let number = current_number
                .parse::<u64>()
                .map_err(|_| "Invalid number")?;
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

include!(concat!(env!("OUT_DIR"), "/rooster_sound.rs"));

fn play_sound() {
    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let sink = Sink::try_new(&stream_handle).unwrap();

    let cursor = std::io::Cursor::new(ROOSTER_SOUND);
    let source = Decoder::new(cursor).unwrap();
    sink.append(source);

    sink.sleep_until_end();
}
