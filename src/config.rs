use serde_derive::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct Data {
    pub config: Config,
}

#[derive(Deserialize, Serialize)]
pub struct Config {
    pub audio_path: String,
    pub battery_critical: i32,
    pub battery_low: i32,
    pub normal_sleep_time: u64,
    pub fast_sleep_time: u64,
    pub critical_sleep_time: u64,
    pub starting_bleep: bool,
    pub target_session: Vec<String>,
    pub enable_plug_in_check: bool,
    pub plug_in_check_interval: u64,
    pub signal_check_interval: u64,
    pub path_to_capacity: String,
    pub path_to_status: String,
}

impl Config {
    pub fn default_config() -> Config {
        return Config {
            audio_path: "none".to_string(),
            battery_critical: 30,
            battery_low: 45,
            normal_sleep_time: 300,
            fast_sleep_time: 5,
            critical_sleep_time: 120,
            starting_bleep: false,
            target_session: vec! ["any".to_string()],
            enable_plug_in_check: true,
            plug_in_check_interval: 2,
            signal_check_interval: 1000,
            path_to_status: "/sys/class/power_supply/BAT1/status".to_string(),
            path_to_capacity: "/sys/class/power_supply/BAT1/capacity".to_string(),
        };
    }
    
    pub fn print_debug(&self) -> () {
        println!(" == Configuration == ");
        println!("\taudio_path : {}", self.audio_path);
        println!("\tbattery_critical : {}", self.battery_critical);
        println!("\tbattery_low : {}", self.battery_low);
        println!("\tnormal_sleep_time : {}", self.normal_sleep_time);
        println!("\tfast_sleep_time : {}", self.fast_sleep_time);
        println!("\tcritical_sleep_time : {}", self.critical_sleep_time);
        println!("\tstarting_bleep : {}", self.starting_bleep);
        println!("\ttarget_session : {:?}", self.target_session);
        println!("\tenable_plug_in_check : {}", self.enable_plug_in_check);
        println!("\tplug_in_check_interval : {}", self.plug_in_check_interval);
        println!("\tsignal_check_interval : {}", self.signal_check_interval);
        println!("\tpath_to_status : {}", self.path_to_status);
        println!("\tpath_to_capacity : {}", self.path_to_capacity);
        println!(" == ~/.config/batt_reminder.toml == ");
    }
}
