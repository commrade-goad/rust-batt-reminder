use std::fs;
use std::io::Error;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use std::io::prelude::*;
use std::thread;
use signal_hook::flag;
use std::env;
use std::process;

fn get_batt_percentage() -> i32 {
    let batt_capacity_percentage: String = fs::read_to_string("/sys/class/power_supply/BAT1/capacity")
        .expect("Failed read the battery capacity!");
    let batt_capacity_percentage_int:i32=batt_capacity_percentage.trim().parse::<i32>().unwrap();
    return batt_capacity_percentage_int;
}

fn get_batt_status() -> String{ 
    let bat_status= fs::read_to_string("/sys/class/power_supply/BAT1/status")
        .expect("Failed read the battery status!");
    return bat_status.to_string();
}

fn program_lock() -> i32 {
        let lock_file_location: String = "/tmp/batt_file_lock.lock".to_string();
        let file_lock_check = std::path::Path::new(&lock_file_location).is_file();
        if file_lock_check == false { 
            let mut file_lock = std::fs::File::create(lock_file_location).expect("Error encountered while creating file!"); 
            file_lock.write_all(b"Running\n").expect("Error while writing to file");
            return 0;
        }
        else {
            println!("The program is already running.\nclose the program and do 'rm {}'",lock_file_location);
            return 1;
        }
}

fn the_program(){
    // basic settings
    let batt_alert_percentage:i32 = 30;
    let batt_low_percentage:i32 = 45;
    let sleep_time_normal:u64 = 300;
    let sleep_time_fast:u64 = 120;
    let sleep_time_alert:u64 = 5;

    let batt_status: String = get_batt_status();
    let batt_capacity: i32 = get_batt_percentage();
    match &batt_status[..]{
        "Charging\n" => { println!("Battery is Charging");
                        std::thread::sleep(Duration::from_secs(sleep_time_normal));
                        }
        "Full\n" => {println!("Battery is Full");
                    std::thread::sleep(Duration::from_secs(sleep_time_normal));
                    }
        "Discharging\n" => {println!("Battery is Discharging");
                            if batt_capacity < batt_alert_percentage {
                                println!("Batt {}", batt_capacity);
                                std::thread::sleep(Duration::from_secs(sleep_time_alert));
                            }
                            else if batt_capacity < batt_low_percentage{
                                println!("Batt {}", batt_capacity);
                                std::process::Command::new("dunstify").arg("-u").arg("2").arg(&format!("{batt_capacity} Battery remaining, please plug in the charger.")).spawn().expect("Failed!");
                                std::thread::sleep(Duration::from_secs(sleep_time_fast));
                            }
                            else {
                                println!("More than {}", batt_capacity);
                                std::thread::sleep(Duration::from_secs(sleep_time_normal));
                                }
                            }
        _ => {println!("Unknown.")
                 }
            }
        std::thread::sleep(Duration::from_secs(5));

}

fn get_session_env() -> i32 {
    let some_value = "XDG_CURRENT_DESKTOP".to_string();
    let get_current_session = match env::var(&some_value){
        Ok(val) => val,
        Err(e) => panic!("could not find {}: {}", &some_value, e),
    };
    match &get_current_session[..]{
        "sway" => {return 0;}
        _ => {return 1;}
    }
}

fn main() -> Result<(), Error>{
    thread::spawn(move ||{ 
        let check_session = get_session_env();
        if check_session == 1{
            process::exit(1);
        }
        let check_lock = program_lock(); 
            if check_lock == 1 {
                std::process::exit(1);
            }
        let mut program_loop = 1;
        while program_loop == 1 {
            the_program();
            program_loop = 0;
            };        
    });

    let term = Arc::new(AtomicBool::new(false));
    for sig in signal_hook::consts::TERM_SIGNALS{
        //flag::register(signal_hook::consts::SIGTERM, Arc::clone(&term));
         flag::register(*sig, Arc::clone(&term))?;
    }
    while !term.load(Ordering::Relaxed) {
        std::thread::sleep(Duration::from_secs(2));    
        }
    std::fs::remove_file("/tmp/batt_file_lock.lock").expect("Failed to delete the lock file.\n Please delete it manually.");
    Ok(())
}
