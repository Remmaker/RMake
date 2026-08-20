use crate::config::*;
use crate::build::*;
use crate::run::*;
use crate::cache::*;

fn help() {
    println!(
        r#"
– Rmake a readable build system

Usage
  rmake                 → build (search for the first ".rm" file in the cwd and execute build section)
  rmake file | file.rm  → build using provided config file
  rmake run             → execute run section
  rmake help            → show this
"#);
}

fn end(res: &CmdOutput) -> Result<i32, ConfigError> { 
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

pub fn run() -> Result<i32, ConfigError>{
    let rmake_arg: Vec<String> = std::env::args().collect();
    let mut path_build_file: String = String::new();
    let mut should_build = false;
    let mut should_run = false;

    cache_create(false)?;
    match rmake_arg.get(1).map(|s| s.as_str()) {
        Some("run") => {
            if let Some(second) = rmake_arg.get(2) {
                file_is_conf(&second)?;
                path_build_file = second.to_string();
            } else {
                path_build_file = find_first_rm()?;
            }
            should_run = true;
        },
        Some("help") => { help(); return Ok(0) },
        Some("clean") => { return cache_clean() },
        Some(file) if file_is_conf(file)? => { path_build_file = file.to_string(); should_build = true },
        Some("") | None => { path_build_file = find_first_rm()?; should_build = true },
        _ => { eprintln!("UNREACHABLE") }
    }

    let content = std::fs::read_to_string(path_build_file.clone())
                .map_err(|_| ConfigError::FileNotFound {message: format!("Config File not found: {path_build_file}")} )?;
    let lines: Vec<&str> = content.lines().collect();
    let config: Config = parse(lines)?;

    let res_run: CmdOutput;
    let res_build: CmdOutput;
    check_for_config_requirement(&config)?;

    let mut build_conf = parse_build(&config)?;
    let ccache = cache_get_current()?;
    let scache = cache_compute_src(build_conf.src.clone())?;

    if should_build {
        let src_to_build = cache_compute_diff(build_conf.src.clone(), scache.clone(), ccache)?;
        
        build_conf.src = src_to_build;
        res_build = execute_build(&build_conf)?;

        if let Some(v) = res_build.status.code() {
            if v == 0 {
                cache_update(scache)?;
            }
        }
        return end(&res_build)
    } 

    if should_run {
        let run_conf = parse_run(&config)?;
        if run_conf.rebuild {
            let src_to_build = cache_compute_diff(build_conf.src.clone(), scache.clone(), ccache)?;
            
            build_conf.src = src_to_build;
            res_build = execute_build(&build_conf)?;

            if let Some(r) = res_build.status.code() && r != 0 {
                return end(&res_build)
            }
        }
        res_run = execute_run(&run_conf)?;
        return end(&res_run)
    }

    Err(ConfigError::FileNotFound {message: "Unexpected".into()})
}
