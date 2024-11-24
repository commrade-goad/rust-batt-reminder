mod config;
use config::*;
use rodio::{source::Source, Decoder, OutputStream};
use signal_hook::flag;
use std::env;
use std::fs;
use std::io::prelude::*;
use std::io::BufReader;
use std::io::Error;
use std::path;
use std::process;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use toml;

const NEAR_DED: u64 = 10;

fn read_configuration_file() -> Config {
    let home_env: String = "HOME".to_string();
    let mut path_to_conf: String = match env::var(&home_env) {
        Ok(val) => val,
        Err(e) => {
            spawn_notif(
                format!(
                    "goad-rust-batt-reminder could not find env var of {}",
                    &home_env
                ),
                0,
            );
            panic!("could not find {}: {}", &home_env, e);
        }
    };
    path_to_conf.push_str("/.config/batt_reminder.toml");
    match path::Path::new(&path_to_conf).is_file() {
        false => {
            let create_config: DataForWrite = DataForWrite { config: Config::default_config() };
            match toml::to_string(&create_config) {
                Ok(val) => {
                    let mut some = fs::File::create(&path_to_conf)
                        .expect("Error encountered while creating file!");
                    some.write_all(val.as_bytes())
                        .expect("Error encountered while writing to the file!");
                }
                Err(e) => {
                    panic!("{}", e);
                }
            }
            return Config::default_config();
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
                    println!("Failed to parse the config file! Using the default config..");
                    return Config::default_config();
                }
            };
            let mut conf: Config = Config::default_config();
            conf = conf.convert_data(data);
            return conf;
        }
    }
}

fn get_batt_percentage(path_to_file: &String) -> u64 {
    let batt_capacity_percentage: String =
        fs::read_to_string(path_to_file).expect("Failed read the battery capacity!");
    let batt_capacity_percentage_int: u64 = batt_capacity_percentage.trim().parse::<u64>().unwrap();
    return batt_capacity_percentage_int;
}

fn get_batt_status(path_to_file: &String) -> String {
    let bat_status = fs::read_to_string(path_to_file).expect("Failed read the battery status!");
    return bat_status.trim().to_string();
}

fn write_prog_pid(lock_file_location: String, pid: u32) -> () {
    let mut file_lock =
        fs::File::create(lock_file_location).expect("Error encountered while creating file!");
    file_lock
        .write_all(format!("{}", pid).as_bytes())
        .expect("Error while writing to file");
}

fn program_lock() -> i32 {
    let pid: u32 = process::id();
    let lock_file_location: String = "/tmp/batt_file_lock.lock".to_string();
    match path::Path::new(&lock_file_location).is_file() {
        false => {
            write_prog_pid(lock_file_location.clone(), pid);
            return 0;
        }
        true => {
            let c_pid: Option<String> =
                match fs::read_to_string("/tmp/batt_file_lock.lock".to_string()) {
                    Ok(val) => Some(val),
                    Err(_) => None,
                };
            match c_pid {
                Some(val) => {
                    if std::path::Path::new(&format!("/proc/{}", val)).exists() {
                        println!(
                            "The program is already running.\nclose the program and do 'rm {}' if the lock file failed to be deleted automatically",
                            lock_file_location
                        );
                        return 1;
                    }
                    write_prog_pid(lock_file_location.clone(), pid);
                }
                None => write_prog_pid(lock_file_location.clone(), pid),
            };
        }
    }
    return 0;
}

fn the_program(configuration: &Config, allow_execute: &mut bool) {
    // basic settings
    let batt_alert_percentage: u64 = configuration.battery_critical;
    let batt_low_percentage: u64 = configuration.battery_low;
    let sleep_time_normal: u64 = configuration.normal_sleep_time;
    let sleep_time_alert: u64 = configuration.fast_sleep_time;
    let sleep_time_fast: u64 = configuration.critical_sleep_time;
    let c_near_ded: String = configuration.near_ded_command.clone();
    let c_exec_low: String = configuration.bat_low_command_to_exec.clone();
    let c_exec_crit: String = configuration.bat_crit_command_to_exec.clone();

    let batt_status: String = get_batt_status(&configuration.path_to_status);
    let batt_capacity: u64 = get_batt_percentage(&configuration.path_to_capacity);
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
            println!("Batt level {}", batt_capacity);
            println!("allow execute : {}", allow_execute);
            if batt_capacity < batt_alert_percentage {
                if *allow_execute == true && !c_exec_crit.is_empty() {
                    let vectorized: Vec<&str> = c_exec_crit.split_whitespace().collect();
                    let proc = vectorized[0];
                    let proc_args = &vectorized[1..];
                    spawn_process(proc, proc_args.to_vec());
                    *allow_execute = false;
                    println!("set allow execute to : {}", allow_execute);
                }
                spawn_notif(
                    format!("{batt_capacity}% Battery remaining, please plug in the charger."),
                    batt_capacity,
                );
                match play_notif_sound(&configuration.audio_path.parse().unwrap()) {
                    Ok(..) => {
                        println!("Audio played");
                    }
                    _ => {
                        println!("Audio Cant be played");
                    }
                };
                thread::sleep(Duration::from_secs(sleep_time_fast));
            } else if batt_capacity < batt_low_percentage {
                if *allow_execute == true && !c_exec_low.is_empty() {
                    let vectorized: Vec<&str> = c_exec_low.split_whitespace().collect();
                    let proc = vectorized[0];
                    let proc_args = &vectorized[1..];
                    spawn_process(proc, proc_args.to_vec());
                    *allow_execute = false;
                    println!("set allow execute to : {}", allow_execute);
                }
                thread::sleep(Duration::from_secs(sleep_time_alert));
            } else if batt_capacity < NEAR_DED {
                spawn_notif(
                    format!(
                        "Battery is less than {}% The system will run {} in 15 seconds from now...",
                        batt_capacity, c_near_ded
                    ),
                    0,
                );
                let vectorized: Vec<&str> = c_near_ded.split_whitespace().collect();
                let proc = vectorized[0];
                let proc_args = &vectorized[1..];
                spawn_process(proc, proc_args.to_vec());
            } else {
                println!("Batt level {}", batt_capacity);
                if *allow_execute == false {
                    *allow_execute = true;
                }
                println!("set allow execute to : {}", allow_execute);
                thread::sleep(Duration::from_secs(sleep_time_normal));
            }
        }
        _ => {
            println!("Unknown.")
        }
    }
    thread::sleep(Duration::from_secs(5));
}

fn get_session_env(session: &Vec<String>) -> i32 {
    let intended_session_name = &session.to_owned();
    let some_value = "XDG_CURRENT_DESKTOP".to_string();
    let get_current_session = match env::var(&some_value) {
        Ok(val) => val,
        Err(e) => {
            spawn_notif(
                format!(
                    "goad-rust-batt-reminder could not found env var of {}",
                    &some_value
                ),
                0,
            );
            panic!("could not found {} = {}", &some_value, e);
        }
    };
    for i in 0..intended_session_name.len() {
        if get_current_session.eq(&intended_session_name[i]) == true
            || intended_session_name[i].eq("any") == true
        {
            return 0;
        }
    }
    return 1;
}

fn play_notif_sound(_path_to_file: &String) -> Result<i32, i32> {
    match &_path_to_file[..] {
        "none" => {
            return Err(1);
        }
        _ => {}
    }
    match path::Path::new(&_path_to_file).is_file() {
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

fn check_charging(
    path_to_file: &String,
    interval: u64,
    path_to_status: &String,
    plug_in_check_command_to_exec: &String,
    plug_out_check_command_to_exec: &String,
) {
    println!(
        "check_charging: this thread will check if the battery is Discharging every {} sec(s)...",
        &interval
    );
    loop {
        let battery_status = get_batt_status(path_to_status);
        match &battery_status[..] {
            // check from Discharging to charging
            "Discharging" => {
                thread::sleep(Duration::from_secs(interval));
                match &get_batt_status(path_to_status)[..] {
                    "Discharging" => {}
                    _ => {
                        let vectorized: Vec<&str> =
                            plug_in_check_command_to_exec.split_whitespace().collect();
                        let proc = vectorized[0];
                        let proc_args = &vectorized[1..];
                        spawn_process(proc, proc_args.to_vec());
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
                match &get_batt_status(path_to_status)[..] {
                    "Discharging" => {
                        let vectorized: Vec<&str> =
                            plug_out_check_command_to_exec.split_whitespace().collect();
                        let proc = vectorized[0];
                        let proc_args = &vectorized[1..];
                        spawn_process(proc, proc_args.to_vec());
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
                        thread::sleep(Duration::from_secs(5));
                    }
                }
            }
        }
    }
}

fn spawn_process(proc: &str, args: Vec<&str>) {
    process::Command::new(proc)
        .args(args)
        .spawn()
        .expect("Failed!");
}

fn spawn_notif(string: String, progress_bar_value: u64) {
    match &progress_bar_value {
        0 => {
            process::Command::new("/usr/bin/notify-send")
                .arg("--app-name=batt-reminder")
                .arg("--replace-id=2592")
                .arg("--urgency=critical")
                .arg("--expire-time= 10000")
                .arg(&format!("{string}"))
                .spawn()
                .expect("Failed!");
        }
        _ => {
            process::Command::new("/usr/bin/notify-send")
                .arg("--app-name=batt-reminder")
                .arg("--replace-id=2592")
                .arg(&format!("--hint=int:value:{}", progress_bar_value))
                .arg("--urgency=critical")
                .arg("--expire-time= 10000")
                .arg(&format!("{string}"))
                .spawn()
                .expect("Failed!");
        }
    }
}

fn main() -> Result<(), Error> {
    let user_configuration = read_configuration_file();

    thread::spawn(move || {
        let user_configuration = read_configuration_file();
        // print user config for debug
        user_configuration.print_debug();

        let check_session = get_session_env(&user_configuration.target_session);
        match user_configuration.starting_bleep {
            true => {
                match play_notif_sound(&user_configuration.audio_path) {
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
        if program_lock() == 1 {
            process::exit(1);
        }
        let mut allow_execute = true;
        loop {
            the_program(&user_configuration, &mut allow_execute);
        }
    });

    match &user_configuration.enable_plug_in_check {
        true => {
            thread::spawn(move || {
                check_charging(
                    &user_configuration.audio_path,
                    user_configuration.plug_in_check_interval,
                    &user_configuration.path_to_status,
                    &user_configuration.plug_in_check_command_to_exec,
                    &user_configuration.plug_out_check_command_to_exec,
                );
            });
        }
        false => {}
    }

    let term = Arc::new(AtomicBool::new(false));
    for sig in signal_hook::consts::TERM_SIGNALS {
        flag::register(*sig, Arc::clone(&term))?;
    }
    while !term.load(Ordering::Relaxed) {
        thread::sleep(Duration::from_millis(
            user_configuration.signal_check_interval,
        ));
    }
    fs::remove_file("/tmp/batt_file_lock.lock")
        .expect("Failed to delete the lock file.\n Please delete it manually.");
    Ok(())
}
