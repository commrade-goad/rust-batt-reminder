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
    starting_bleep: bool,
    target_session: String,
    enable_plug_in_check: bool,
    plug_in_check_interval: u64,
}

//

fn read_configuration_file() -> (String, i32, i32, u64, u64, u64, bool, String, bool, u64) {
    let home_env: String = "HOME".to_string();
    let mut path_to_conf: String = match env::var(&home_env) {
        Ok(val) => val,
        Err(e) => {
            err_notif(format!(
                "goad-rust-batt-reminder could not find env var of {} = {}",
                &home_env, e
            ));
            panic!("could not find {}: {}", &home_env, e);
        }
    };
    path_to_conf.push_str("/.config/batt_reminder.toml");
    match std::path::Path::new(&path_to_conf).is_file() {
        false => {
            let mut create_config =
                fs::File::create(&path_to_conf).expect("Error encountered while creating file!");
            create_config.write_all(b"[config]\naudio_path = \"none\"\nbattery_critical = 30\nbattery_low = 45\nnormal_sleep_time = 300\n fast_sleep_time = 5\ncritical_sleep_time = 120\nstarting_bleep = true\ntarget_session = \"sway\"\nenable_plug_in_check = true\nplug_in_check_interval = 2").expect("Error while writing to file");
            println!("Created the config file.\nusing the default settings.");
            return (
                // using default settings
                "none".to_string(),
                30,
                45,
                300,
                5,
                120,
                false,
                "sway".to_string(),
                true,
                2,
            );
        }
        true => {
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
            return (
                data.config.audio_path,
                data.config.battery_critical,
                data.config.battery_low,
                data.config.normal_sleep_time,
                data.config.fast_sleep_time,
                data.config.critical_sleep_time,
                data.config.starting_bleep,
                data.config.target_session,
                data.config.enable_plug_in_check,
                data.config.plug_in_check_interval,
            );
        }
    }
}

fn get_batt_percentage() -> i32 {
    let batt_capacity_percentage: String =
        fs::read_to_string("/sys/class/power_supply/BAT1/capacity")
            .expect("Failed read the battery capacity!");
    let batt_capacity_percentage_int: i32 = batt_capacity_percentage.trim().parse::<i32>().unwrap();
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
                "The program is already running.\nclose the program and do 'rm {}' if the lock file failed to be deleted automatically",
                lock_file_location
            );
            return 1;
        }
    }
}

fn the_program(configuration: &(String, i32, i32, u64, u64, u64, bool, String, bool, u64)) {
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

fn get_session_env(session: &String) -> i32 {
    let intended_session_name = &session.to_owned();
    let some_value = "XDG_CURRENT_DESKTOP".to_string();
    let get_current_session = match env::var(&some_value) {
        Ok(val) => val,
        Err(e) => {
            err_notif(format!(
                "goad-rust-batt-reminder could not found env var of {} = {}",
                &some_value, e
            ));
            println!("could not found {} = {}", &some_value, e);
            return 0;
        }
    };
    match &get_current_session.eq(intended_session_name) {
        true => {
            return 0;
        }
        _ => {
            return 1;
        }
    }
}

fn play_notif_sound(_path_to_file: &String) -> Result<i32, i32> {
    match &_path_to_file[..] {
        "none" => {
            return Err(1);
        }
        _ => {}
    }
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

fn check_charging(path_to_file: &String, interval: u64) {
    println!(
        "check_charging: this thread will check if the battery is Discharging every {} sec(s)...",
        &interval
    );
    loop {
        let battery_status = get_batt_status();
        match &battery_status[..] {
            // check from Discharging to charging
            "Discharging" => {
                thread::sleep(Duration::from_secs(interval));
                match &get_batt_status()[..] {
                    "Discharging" => {}
                    _ => {
                        match play_notif_sound(&path_to_file) {
                            Ok(..) => {
                                println!("Audio played");
                            }
                            _ => {
                                println!("Audio Cant be played");
                            }
                        };
                    }
                }
            }
            _ => {
                // check from charging or full to Discharge
                thread::sleep(Duration::from_secs(interval));
                match &get_batt_status()[..] {
                    "Discharging" => {
                        match play_notif_sound(&path_to_file) {
                            Ok(..) => {
                                println!("Audio played");
                            }
                            _ => {
                                println!("Audio Cant be played");
                            }
                        };
                    }
                    _ => {
                        thread::sleep(Duration::from_secs(10));
                    }
                }
            }
        }
    }
}

fn err_notif(string: String) {
    process::Command::new("/usr/bin/dunstify")
        .arg("-u")
        .arg("2")
        .arg(&format!("{string}"))
        .spawn()
        .expect("Failed!");
}

fn main() -> Result<(), Error> {
    let user_configuration = read_configuration_file();
    println!("Reading the configuration file...");
    thread::spawn(move || {
        let user_configuration = read_configuration_file();
        // print user config for debug
        println!(" == Configuration == ");
        println!("\taudio_path : {}", user_configuration.0);
        println!("\tbattery_critical : {}", user_configuration.1);
        println!("\tbattery_low : {}", user_configuration.2);
        println!("\tnormal_sleep_time : {}", user_configuration.3);
        println!("\tfast_sleep_time : {}", user_configuration.4);
        println!("\tcritical_sleep_time : {}", user_configuration.5);
        println!("\tstarting_bleep : {}", user_configuration.6);
        println!("\ttarget_session : {}", user_configuration.7);
        println!("\tenable_plug_in_check : {}", user_configuration.8);
        println!("\tplug_in_check_interval : {}", user_configuration.9);
        println!(" == ~/.config/batt_reminder.toml == ");

        let check_session = get_session_env(&user_configuration.7);
        match user_configuration.6 {
            true => {
                match play_notif_sound(&user_configuration.0) {
                    Ok(..) => {
                        println!("Audio played");
                    }
                    _ => {
                        println!("Audio Cant be played");
                    }
                };
            }
            false => {}
        };
        if check_session == 1 {
            process::exit(1);
        }
        let check_lock = program_lock();
        if check_lock == 1 {
            process::exit(1);
        }
        let program_loop = 1;
        while program_loop == 1 {
            the_program(&user_configuration);
        }
    });

    match &user_configuration.8 {
        true => {
            thread::spawn(move || {
                check_charging(&user_configuration.0, user_configuration.9);
            });
        }
        false => {}
    }

    let term = Arc::new(AtomicBool::new(false));
    for sig in signal_hook::consts::TERM_SIGNALS {
        flag::register(*sig, Arc::clone(&term))?;
    }
    while !term.load(Ordering::Relaxed) {
        thread::sleep(Duration::from_secs(2));
    }
    fs::remove_file("/tmp/batt_file_lock.lock")
        .expect("Failed to delete the lock file.\n Please delete it manually.");
    Ok(())
}
