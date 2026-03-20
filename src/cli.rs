use crate::config::*;
use crate::build::*;
use crate::run::*;

fn help() {
    println!(
        r#"
– Rmake a readable build system

Usage
  rmake                 → build (search for the first ".rm" file in the cwd and execute build section)
  rmake file | file.rm  → build using provided config file
  rmake run             → execute run section (without rebuild)
  rmake help            → show this
"#);
}

pub fn run() -> Result<i32, ConfigError>{
    let rmake_arg: Vec<String> = std::env::args().collect();
    let mut path_build_file: String = String::new();
    let mut should_build = false;
    
    match rmake_arg.get(1).map(|s| s.as_str()) {
        Some("run") => {
            if let Some(second) = rmake_arg.get(2) {
                file_is_conf(&second)?;
                path_build_file = second.to_string();
            } else {
                path_build_file = find_first_rm()?;
            }

        },
        Some("watch") => { todo!() },
        Some("help") => { help(); return Ok(0) },
        Some(file) if file_is_conf(file)? => { path_build_file = file.to_string(); should_build = true },
        None => { path_build_file = find_first_rm()?; should_build = true },
        _ => { eprintln!("UNREACHABLE") }
    }

    let content = std::fs::read_to_string(path_build_file.clone())
                .map_err(|_| ConfigError::FileNotFound {message: format!("Config File not found: {path_build_file}")} )?;
    let lines: Vec<&str> = content.lines().collect();
    let config: Config = parse(lines)?;

    let res: CmdOutput;
    check_for_config_requirement(&config)?;

    if should_build {
        let build_conf = parse_build(&config)?;
        res = execute_build(&build_conf)?;
    } else {
        let run_conf = parse_run(&config)?;
        res = execute_run(&run_conf)?;
    }

    let msgerr: String = res.stderr.clone();
    let msgok: String  = res.stdout.clone();

    if let Some(v) = res.status.code() {
        if v == 0 {
            if !msgok.is_empty() {
                eprintln!("{}", msgok);
            }
            eprintln!("RMake returned {} exit status", v);
            Ok(v)
        } else {
            if !msgerr.is_empty() {
                eprintln!("{}", msgerr);
            }
            Err(ConfigError::NonZeroExit { code: v })
        }
    } else {
        if !msgerr.is_empty() {
            eprintln!("{}", res.stderr);
        }
        Err(ConfigError::NonZeroExit { code: -1 } )
    }
}
