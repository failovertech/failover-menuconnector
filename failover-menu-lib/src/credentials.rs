use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;
use home::home_dir;

#[derive(Debug)] 
pub struct Credentials {
    pub endpoint: String,
    pub login: String,
    pub key: String,
    pub email: String,
    pub expiration: String,
}

impl Credentials {
    pub fn from_file(path: &str) -> Result<Self, std::io::Error> {
        let path = home_dir()
            .map(|p| p.join(".failovermenu"))
            .expect("Home directory not found");

        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut credentials = Self {
            endpoint: String::new(),
            login: String::new(),
            key: String::new(),
            email: String::new(),
            expiration: String::new(),
        };

        for line in reader.lines() {
            let line = line?;
            let mut parts = line.split('=');
            let key = parts.next().unwrap().trim();
            let value = parts.next().unwrap().trim();

            match key {
                "endpoint" => credentials.endpoint = value.to_string(),
                "login" => credentials.login = value.to_string(),
                "key" => credentials.key = value.to_string(),
                "email" => credentials.email = value.to_string(),
                "expiration" => credentials.expiration = value.to_string(),
                _ => {}
            }
        }

        Ok(credentials)
    }
}
		