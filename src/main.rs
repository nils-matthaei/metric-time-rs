use core::time;
use std::{
    env,
    io::Write,
    process, thread,
    time::{SystemTime, UNIX_EPOCH},
};

fn main() {
    // Handle commandline arguments
    let args: Vec<String> = env::args().collect();
    let mut tz_offset: i32 = 0;
    let mut skip_next: bool = false;
    let mut oneshot: bool = false;
    for i in 1..args.len() {
        if skip_next {
            skip_next = false;
            continue;
        }
        match args[i].as_str() {
            "-t" => {
                if i + 1 < args.len() {
                    tz_offset = parse_timezone(&args[i + 1]);
                    skip_next = true;
                } else {
                    eprint!("-t must be followed by a number");
                    process::exit(1);
                }
            }
            "--oneshot" => {
                oneshot = true;
            }
            _ => {
                eprintln!("Unkown argument: {}", args[i]);
            }
        }
    }

    let mut decimal_now = DecimalTime::now(tz_offset);
    if oneshot {
        println!("{}", decimal_now.to_string());
    } else {
        decimal_now.print_loop();
    }
}

fn parse_timezone(tz_arg: &String) -> i32 {
    match tz_arg.parse::<i32>() {
        Ok(offset) => {
            return offset;
        }
        Err(_) => {
            eprintln!("Error: '{}' is not a valid integer.", tz_arg);
            process::exit(1);
        }
    }
}

const MSECS_DAY: f64 = 86_400_000.0;
const MSECS_DECIMAL_HOUR: f64 = 8_640_000.0;
const MSECS_HOUR: i32 = 3_600_000;

struct DecimalTime {
    dec_hour: u32,
    dec_min: u32,
    dec_sec: u32,
    //tz_offset: i32,
}

impl DecimalTime {
    fn now(tz_offset: i32) -> Self {
        let now = SystemTime::now();
        let duration_since_epoch = now
            .duration_since(UNIX_EPOCH)
            .expect("Jim, we're in the PAST!");

        let total_msecs = duration_since_epoch.as_millis();

        // Calculate current time in milliseconds
        let abs_offset: f64 = (tz_offset * MSECS_HOUR) as f64;
        let current_msecs: f64 = ((total_msecs as f64) + abs_offset) % MSECS_DAY;

        // Calculate decimal time
        // hour
        let decimal_time: f64 = (current_msecs as f64) / MSECS_DECIMAL_HOUR;
        let decimal_hour: f64 = decimal_time.floor();
        // minute
        let decimal_minute_frac: f64 = (decimal_time - decimal_hour) * 100.0;
        let decimal_minute: f64 = decimal_minute_frac.floor();
        //second
        let decimal_second_frac: f64 = (decimal_minute_frac - decimal_minute) * 100.0;
        let decimal_second: f64 = decimal_second_frac.floor();

        return DecimalTime {
            dec_hour: decimal_hour as u32,
            dec_min: decimal_minute as u32,
            dec_sec: decimal_second as u32,
            //tz_offset,
        };
    }

    fn to_string(&self) -> String {
        return format!(
            "{:0>2}:{:0>2}:{:0>2}",
            self.dec_hour, self.dec_min, self.dec_sec
        );
    }

    fn increment(&mut self) {
        self.dec_sec += 1;
        if self.dec_sec < 100 {
            return;
        }
        self.dec_sec %= 100;

        self.dec_min += 1;
        if self.dec_min < 100 {
            return;
        }
        self.dec_min %= 100;

        self.dec_hour += 1;
        if self.dec_hour < 10 {
            return;
        }
        self.dec_hour %= 10;
    }

    fn print_loop(&mut self) {
        let delay = time::Duration::from_millis(864); // 1 decimal second == 864 milliseconds
        loop {
            print!("\r{}", self.to_string());
            std::io::stdout().flush().unwrap();
            self.increment();
            thread::sleep(delay);
        }
    }
}
