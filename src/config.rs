use std::{collections::HashMap, process::ExitStatus};

pub enum ConfigError {
    InvalidSyntax { line: usize, message: String },
    FileNotFound { message: String },
    ReadError { message: String },
    InvalidConfig { message: String },
    CommandFailed { cmd: String, message: String },
    NonZeroExit { code: i32 },
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ConfigError::InvalidSyntax { line, message } => {
                write!(f, "InvalidSyntax: {message} L.{line}")
            },
            ConfigError::FileNotFound { message } => {
                write!(f, "FileNotFound: {message}")
            },
            ConfigError::ReadError { message } => {
                write!(f, "Read Error {message}")
            }
            ConfigError::InvalidConfig { message } => {
                write!(f, "Invalid Config: {message}")
            },
            ConfigError::CommandFailed { cmd, message } => {
                write!(f, "Failed to execute command {cmd}: {message}")
            },
            ConfigError::NonZeroExit { code } => {
                write!(f, "RMake returned {} exit status", code)
            },
       }
    }
}

#[derive(Default, Debug)]
pub struct Config {
    pub global: HashMap<String, String>,
    pub section: HashMap<String, HashMap<String, String>>,
    current_section: Option<String>
}

#[derive(Default, Debug)]
pub struct CmdOutput {
    pub stdout: String,
    pub stderr: String,
    pub status: ExitStatus
}

fn is_comment_line_or_empty(line: &str) -> bool {

    if line.starts_with("#") || line.starts_with(";") || line.starts_with("//") || line.is_empty() {
        return true;
    }

    false
}

pub fn parse_value(conf: &Config, input: &str) -> Result<String, ConfigError> {
    let mut ret = input.to_string();

    while let Some(target) = ret.find("${") {
        let start = target + 2;

        let rest = &ret[start..];
        let Some(rel_end) = rest.find('}') else {
            return Err(ConfigError::InvalidSyntax {
                line: 0,
                message: "Missing '}' in variable syntax".into(),
            });
        };

        let end = start + rel_end;
        let current_var = &ret[start..end];

        let Some(value) = conf.global.get(current_var) else {
            return Err(ConfigError::InvalidConfig {
                message: format!("Unknown variable: {current_var}"),
            });
        };

        ret.replace_range(target..=end, value);
    }

    Ok(ret)
}

pub fn parse(lines: Vec<&str>) -> Result<Config, ConfigError> {
    let mut config: Config = Config::default();
    
    for (i, raw_str) in lines.iter().enumerate() {
        let line = raw_str.trim();
        let line_number = i+1;
        
        
        if is_comment_line_or_empty(&line) {
            continue;
        }

        if line.ends_with("{") {
            if !config.current_section.is_none() { // Handle a '{' standalone in a scope
                return Err(ConfigError::InvalidSyntax { line: line_number, message: "Unexpected token '{'".into() })
            }
            if line.len() <= 1 {
                if i > 0 && !is_comment_line_or_empty(&lines[i-1].to_lowercase()) {
                    config.current_section = Some(lines[i-1].trim().to_string().to_lowercase());
                    continue;
                } else {
                    return Err(ConfigError::InvalidSyntax { line: line_number, message: "Unexpected token '{'".into() })
                }
                
            }

            config.current_section = Some(line[..line.len()-1].trim().to_string().to_lowercase());
            continue;
        }

        if let Some((key, value)) = line.split_once("=") {
            if let Some(section) = &config.current_section {
                let parsed_value = parse_value(&config, value)?;
                config.section.entry(section.clone()).or_default().insert(key.to_string(), parsed_value);
                continue;
            }

            config.global.insert(key.trim().to_string().to_lowercase(), value.trim().to_string());
            continue;

        } else if !line.ends_with("}") {
            if lines[i+1].trim().to_string() != "{" {
                return Err(ConfigError::InvalidSyntax { line: line_number, message: "Invalid standalone token, missing a '{' or a key=value pair".into()})
            }
            continue;
        }

        if line.ends_with("}") {
            if config.current_section == None {
                return Err(ConfigError::InvalidSyntax { line: line_number, message: "Standalone closing brace '}', missing section".into()})
            }
            config.current_section = None;
            continue;
        }


    }
    Ok(config)
}

pub fn find_first_rm() -> Result<String, ConfigError> {
    let mut ret: String = String::new();
    let dir = std::fs::read_dir(".").map_err(|_| ConfigError::ReadError { message: "Failed to read current directory '.'".into() })?;
    
    for (_, content) in dir.enumerate() {
        if let Ok(entry) = content {
            if entry.path().extension().is_some_and(|ext| ext.to_str() == Some("rm")) {
                ret = entry.file_name().to_string_lossy().to_string();
                break
            }
        }
    }

    if ret.is_empty() {
        return Err(ConfigError::FileNotFound { message: "Config file not found".into() })
    }

    Ok(ret)
}

pub fn file_is_conf(file: &str) -> Result<bool, ConfigError> {
    let dir = std::fs::read_dir(".").map_err(|_| ConfigError::ReadError { message: "Failed to read current directory '.'".into() })?;
    let mut ret: bool = false;
    let mut file_search = file.to_string();

    if !file_search.ends_with(".rm") {
        file_search += ".rm";
    } 
    
    for (_, content) in dir.enumerate() {
        if let Ok(entry) = content {
            if entry.path().is_file() && entry.file_name().to_string_lossy().eq(file_search.as_str())  {
                ret = true;
                break
            }
        }
    }

    if !ret {
        return Err(ConfigError::FileNotFound { message: format!("Config file not found: {file}").into() })
    }

    Ok(true)
}

pub fn check_for_config_requirement(conf: &Config) -> Result<(), ConfigError> {
    if !conf.section.contains_key("build") {
        return Err(ConfigError::InvalidConfig{message: "Missing build section, that's required (please, it's a build tool...)".into()});
    }

    Ok(())
} 
