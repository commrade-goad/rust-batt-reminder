use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Deserialize, Serialize)]
pub struct Data {
    pub config: HashMap<String, ConfigType>,
}

#[derive(Deserialize, Serialize)]
pub struct DataForWrite {
    pub config: Config,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(untagged)]
pub enum ConfigType {
    String(String),
    Integer(u64),
    Boolean(bool),
    StringArray(Vec<String>),
}

#[derive(Deserialize, Serialize)]
pub struct Config {
    pub audio_path: String,
    pub battery_critical: u64,
    pub battery_low: u64,
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
    pub near_ded_command: String,
    pub bat_low_command_to_exec: String,
    pub bat_crit_command_to_exec: String,
    pub plug_in_check_command_to_exec: String,
    pub plug_out_check_command_to_exec: String,
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
            target_session: vec!["any".to_string()],
            enable_plug_in_check: true,
            plug_in_check_interval: 2,
            signal_check_interval: 1000,
            path_to_status: "/sys/class/power_supply/BAT1/status".to_string(),
            path_to_capacity: "/sys/class/power_supply/BAT1/capacity".to_string(),
            near_ded_command: "systemctl poweroff".to_string(),
            bat_low_command_to_exec: "".to_string(),
            bat_crit_command_to_exec: "".to_string(),
            plug_in_check_command_to_exec: "".to_string(),
            plug_out_check_command_to_exec: "".to_string(),
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
        println!("\tnear_ded_command : {}", self.near_ded_command);
        println!(
            "\tbat_low_command_to_exec : {}",
            self.bat_low_command_to_exec
        );
        println!(
            "\tbat_crit_command_to_exec : {}",
            self.bat_crit_command_to_exec
        );
        println!(
            "\tplug_in_check_command_to_exec : {}",
            self.plug_in_check_command_to_exec
        );
        println!(
            "\tplug_out_check_command_to_exec : {}",
            self.plug_out_check_command_to_exec
        );
        println!(" == ~/.config/batt_reminder.toml == ");
    }

    pub fn convert_data(&self, data: Data) -> Config {
        let mut config = Config::default_config();
        for (key, value) in data.config {
            match key.as_str() {
                "audio_path" => {
                    if let ConfigType::String(v) = value {
                        config.audio_path = v;
                    }
                }
                "battery_critical" => {
                    if let ConfigType::Integer(v) = value {
                        config.battery_critical = v;
                    }
                }
                "battery_low" => {
                    if let ConfigType::Integer(v) = value {
                        config.battery_low = v;
                    }
                }
                "normal_sleep_time" => {
                    if let ConfigType::Integer(v) = value {
                        config.normal_sleep_time = v as u64;
                    }
                }
                "fast_sleep_time" => {
                    if let ConfigType::Integer(v) = value {
                        config.fast_sleep_time = v as u64;
                    }
                }
                "critical_sleep_time" => {
                    if let ConfigType::Integer(v) = value {
                        config.critical_sleep_time = v as u64;
                    }
                }
                "starting_bleep" => {
                    if let ConfigType::Boolean(v) = value {
                        config.starting_bleep = v;
                    }
                }
                "target_session" => {
                    if let ConfigType::StringArray(v) = value {
                        config.target_session = v;
                    }
                }
                "enable_plug_in_check" => {
                    if let ConfigType::Boolean(v) = value {
                        config.enable_plug_in_check = v;
                    }
                }
                "plug_in_check_interval" => {
                    if let ConfigType::Integer(v) = value {
                        config.plug_in_check_interval = v as u64;
                    }
                }
                "signal_check_interval" => {
                    if let ConfigType::Integer(v) = value {
                        config.signal_check_interval = v as u64;
                    }
                }
                "path_to_capacity" => {
                    if let ConfigType::String(v) = value {
                        config.path_to_capacity = v;
                    }
                }
                "path_to_status" => {
                    if let ConfigType::String(v) = value {
                        config.path_to_status = v;
                    }
                }
                "near_ded_command" => {
                    if let ConfigType::String(v) = value {
                        config.near_ded_command = v;
                    }
                }
                "bat_low_command_to_exec" => {
                    if let ConfigType::String(v) = value {
                        config.bat_low_command_to_exec = v;
                    }
                }
                "bat_crit_command_to_exec" => {
                    if let ConfigType::String(v) = value {
                        config.bat_crit_command_to_exec = v;
                    }
                }
                "plug_in_check_command_to_exec" => {
                    if let ConfigType::String(v) = value {
                        config.plug_in_check_command_to_exec = v;
                    }
                }
                "plug_out_check_command_to_exec" => {
                    if let ConfigType::String(v) = value {
                        config.plug_out_check_command_to_exec = v;
                    }
                }
                _ => {}
            }
        }

        config
    }
}
