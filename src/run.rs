use crate::config::*;

#[derive(Default, Debug)]
pub struct RunConfig {
    target: String,
    args: Option<String>,
    pub rebuild: bool,
    
}

pub fn parse_run(conf: &Config) -> Result<RunConfig, ConfigError> {
    let mut run_conf: RunConfig = RunConfig::default();
    let hash_run = conf.section.get("run");
    let target = hash_run.unwrap();

    if !target.contains_key("target") {
        return Err(ConfigError::InvalidConfig { message: "Missing target section".into()  })
    }


    for (k, v) in target.iter() {
        match k.as_str() {
            "target" => { 
                if v.split_once(" ").is_some() {
                    return Err(ConfigError::InvalidConfig { message: "Only one target is supported at time".into() })
                }
                run_conf.target = v.to_string() 
            },
            "args" => {
                run_conf.args = Some(v.to_string())
            },
            "rebuild" => {
                if v.split_once(" ").is_some() {
                    return Err(ConfigError::InvalidConfig { message: "Only one argument to rebuild is supported at time".into() })
                }
                run_conf.rebuild = match v.as_str() {
                    "true"  => true,
                    "false" => false,
                    "t"     => true,
                    "f"     => false,
                    _       => false
                }
            },
            _ => {
                eprintln!("warning: RMake Unknow keyword {k}");
            }
        }
    }

    Ok(run_conf)
} 

pub fn execute_run(run_conf: &RunConfig) -> Result<CmdOutput, ConfigError> {
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
    
    let mut clonarg = run_conf.args.clone();
    let args = clonarg.get_or_insert("".into()).split_whitespace().map(|s| s.to_string());
    
    let cmd = std::process::Command::new(binary)
        .args(args)
        .output().map_err(|_| ConfigError::CommandFailed { cmd: run_conf.target.clone(), message: "Failed to run target".into() })?;
    let cmdoutput: CmdOutput = CmdOutput { 
        stdout: String::from_utf8_lossy(&cmd.stdout).to_string(),
        stderr: String::from_utf8_lossy(&cmd.stderr).to_string(),
        status: cmd.status 
    };

    Ok(cmdoutput)
}
