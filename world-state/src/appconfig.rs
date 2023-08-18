use std::env;

use config::{Config};
use lazy_static::lazy_static;

lazy_static! {
  pub static ref CONFIG: Config = {
    let mut config_file: String = "./Config".to_string();

    if let Ok(exe_path) = env::current_exe() {
      if let Some(exe_dir) = exe_path.parent() {
        if let Some(exe_dir_str) = exe_dir.to_str() {
          config_file = format!("{}/Config", exe_dir_str);
        }
      }
    }
      let settings = Config::builder()
                     .add_source(config::File::with_name(&config_file))
                     .build()
                     .unwrap();
      settings
  };
}