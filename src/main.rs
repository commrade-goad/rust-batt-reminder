use rodio::{source::Source, Decoder, OutputStream};
use serde_derive::Deserialize;
use signal_hook::flag;
use std::env;
use std::fs;
use std::io::prelude::*;
use std::io::BufReader;
use std::io::Error;
use std::process;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use toml;

// CONFIG STRUCT

#[derive(Deserialize)]
struct Data {
    config: Config,
}

#[derive(Deserialize)]
struct Config {
    audio_path: String,
    battery_critical: i32,
    battery_low: i32,
    normal_sleep_time: u64,
    fast_sleep_time: u64,
    critical_sleep_time: u64,
}

//

fn read_configuration_file() -> (String, i32, i32, u64, u64, u64) {
    let home_env: String = "HOME".to_string();
    let mut path_to_conf: String = match env::var(&home_env) {
        Ok(val) => val,
        Err(e) => panic!("could not find {}: {}", &home_env, e),
    };
    path_to_conf.push_str("/.config/batt_reminder.toml");
    match std::path::Path::new(&path_to_conf).is_file() {
        false => {
            let mut create_config =
                fs::File::create(&path_to_conf).expect("Error encountered while creating file!");
            create_config.write_all(b"[config]\naudio_path = \"none\"\nbattery_critical = 30\nbattery_low = 45\nnormal_sleep_time = 300\n fast_sleep_time = 5\ncritical_sleep_time = 120").expect("Error while writing to file");
            println!("Created the config file.");
            process::exit(0);
        }
        true => {
            println!("Reading the config file..");
            let contents: String = match fs::read_to_string(path_to_conf) {
                Ok(c) => c,
                Err(_) => {
                    println!("Error reading the configuration file!");
                    process::exit(1);
                }
            };
            let data: Data = match toml::from_str(&contents) {
                Ok(d) => d,
                Err(_) => {
                    println!("Unable to load the config file!");
                    process::exit(1);
                }
            };
            println!("audio : {}", data.config.audio_path);
            println!("battery_critical : {}", data.config.battery_critical);
            println!("battery_low : {}", data.config.battery_low);
            println!("normal_sleep_time : {}", data.config.normal_sleep_time);
            println!("fast_sleep_time : {}", data.config.fast_sleep_time);
            println!("critical_sleep_time : {}", data.config.critical_sleep_time);
            return (
                data.config.audio_path,
                data.config.battery_critical,
                data.config.battery_low,
                data.config.normal_sleep_time,
                data.config.fast_sleep_time,
                data.config.critical_sleep_time,
            );
        }
    }
}

fn get_batt_percentage() -> i32 {
    let batt_capacity_percentage: String =
        fs::read_to_string("/sys/class/power_supply/BAT1/capacity")
            .expect("Failed read the battery capacity!");
    let batt_capacity_percentage_int: i32 = batt_capacity_percentage.parse::<i32>().unwrap();
    return batt_capacity_percentage_int;
}

fn get_batt_status() -> String {
    let bat_status = fs::read_to_string("/sys/class/power_supply/BAT1/status")
        .expect("Failed read the battery status!");
    return bat_status.trim().to_string();
}

fn program_lock() -> i32 {
    let lock_file_location: String = "/tmp/batt_file_lock.lock".to_string();
    match std::path::Path::new(&lock_file_location).is_file() {
        false => {
            let mut file_lock = fs::File::create(lock_file_location)
                .expect("Error encountered while creating file!");
            file_lock
                .write_all(b"Running\n")
                .expect("Error while writing to file");
            return 0;
        }
        true => {
            println!(
                "The program is already running.\nclose the program and do 'rm {}'",
                lock_file_location
            );
            return 1;
        }
    }
}

fn the_program(configuration: &(String, i32, i32, u64, u64, u64)) {
    // basic settings
    let batt_alert_percentage: i32 = configuration.1;
    let batt_low_percentage: i32 = configuration.2;
    let sleep_time_normal: u64 = configuration.3;
    let sleep_time_alert: u64 = configuration.4;
    let sleep_time_fast: u64 = configuration.5;

    let batt_status: String = get_batt_status();
    let batt_capacity: i32 = get_batt_percentage();
    match &batt_status[..] {
        "Charging" => {
            println!("Battery is Charging");
            println!("Batt level {}", batt_capacity);
            thread::sleep(Duration::from_secs(sleep_time_normal));
        }
        "Full" => {
            println!("Battery is Full");
            thread::sleep(Duration::from_secs(sleep_time_normal));
        }
        "Discharging" => {
            println!("Battery is Discharging");
            if batt_capacity < batt_alert_percentage {
                println!("Batt level {}", batt_capacity);
                process::Command::new("/usr/bin/dunstify")
                    .arg("-u")
                    .arg("2")
                    .arg(&format!(
                        "{batt_capacity} Battery remaining, please plug in the charger."
                    ))
                    .spawn()
                    .expect("Failed!");
                match play_notif_sound(&configuration.0.parse().unwrap()) {
                    Ok(..) => {
                        println!("Audio played");
                    }
                    _ => {
                        println!("Audio Cant be played");
                    }
                };
                thread::sleep(Duration::from_secs(sleep_time_fast));
            } else if batt_capacity < batt_low_percentage {
                println!("Batt level {}", batt_capacity);
                thread::sleep(Duration::from_secs(sleep_time_alert));
            } else {
                println!("Batt level {}", batt_capacity);
                thread::sleep(Duration::from_secs(sleep_time_normal));
            }
        }
        _ => {
            println!("Unknown.")
        }
    }
    thread::sleep(Duration::from_secs(5));
}

fn get_session_env() -> i32 {
    let some_value = "XDG_CURRENT_DESKTOP".to_string();
    let get_current_session = match env::var(&some_value) {
        Ok(val) => val,
        Err(e) => panic!("could not find {}: {}", &some_value, e),
    };
    match &get_current_session[..] {
        "sway" => {
            return 0;
        }
        _ => {
            return 1;
        }
    }
}

fn play_notif_sound(_path_to_file: &String) -> Result<i32, i32> {
    match std::path::Path::new(&_path_to_file).is_file() {
        false => {
            println!("Error : Cant read the specified file directory!");
            return Err(1);
        }
        true => {
            let (_stream, stream_handle) = OutputStream::try_default().unwrap();
            let file = BufReader::new(fs::File::open(_path_to_file).unwrap());
            let source = Decoder::new(file).unwrap();
            stream_handle
                .play_raw(source.convert_samples())
                .expect("ERROR : Failed to play the audio!");
            thread::sleep(std::time::Duration::from_secs(2));
            return Ok(0);
        }
    }
}

fn main() -> Result<(), Error> {
    thread::spawn(move || {
        let check_session = get_session_env();
        let user_configuration = read_configuration_file();
        match play_notif_sound(&user_configuration.0) {
            Ok(..) => {
                println!("Audio played");
            }
            _ => {
                println!("Audio Cant be played");
            }
        };
        if check_session == 1 {
            process::exit(1);
        }
        let check_lock = program_lock();
        if check_lock == 1 {
            std::process::exit(1);
        }
        let program_loop = 1;
        while program_loop == 1 {
            the_program(&user_configuration);
        }
    });

    let term = Arc::new(AtomicBool::new(false));
    for sig in signal_hook::consts::TERM_SIGNALS {
        //flag::register(signal_hook::consts::SIGTERM, Arc::clone(&term));
        flag::register(*sig, Arc::clone(&term))?;
    }
    while !term.load(Ordering::Relaxed) {
        std::thread::sleep(Duration::from_secs(2));
    }
    std::fs::remove_file("/tmp/batt_file_lock.lock")
        .expect("Failed to delete the lock file.\n Please delete it manually.");
    Ok(())
}
