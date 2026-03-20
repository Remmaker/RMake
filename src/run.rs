use crate::config::*;

#[derive(Default, Debug)]
pub struct RunConfig {
    target: String,
    rebuild: bool,
}

pub fn parse_run(conf: &Config) -> Result<RunConfig, ConfigError> {
    let mut run_conf: RunConfig = RunConfig::default();
    let hash_run = conf.section.get("run");
    let target = hash_run.unwrap();

    if !target.contains_key("target") {
        return Err(ConfigError::InvalidConfig { message: "Missing target section".into()  })
    }


    for (k, v) in target.iter() {
        match k.as_str()  {
            "target" => { 
                if v.split_once(" ").is_some() {
                    return Err(ConfigError::InvalidConfig { message: "Only one target is supported at time".into() })
                }
                run_conf.target = v.to_string() 
            },
            _ => {
                eprintln!("warning: RMake Unknow keyword {k}");
            }
        }
    }

    Ok(run_conf)
} 

pub fn execute_run(run_conf: &RunConfig) -> Result<CmdOutput, ConfigError> {
    let mut ret: CmdOutput;

    let binary = if run_conf.target.starts_with(['/', '\\'])
                    || run_conf.target.contains(":\\")
                    || run_conf.target.starts_with("./")
                    || run_conf.target.starts_with("../") 
    {
        run_conf.target.clone()
    } else {
        if cfg!(windows) {
            format!(".\\{}", run_conf.target)
        } else {
            format!("./{}", run_conf.target)
        }
    };

    let cmd = std::process::Command::new(binary)
        .output().map_err(|_| ConfigError::CommandFailed { cmd: run_conf.target.clone(), message: "Failed to run target".into() })?;
    let cmdoutput: CmdOutput = CmdOutput { 
        stdout: String::from_utf8_lossy(&cmd.stdout).to_string(),
        stderr: String::from_utf8_lossy(&cmd.stderr).to_string(),
        status: cmd.status 
    };

    Ok(cmdoutput)
}
