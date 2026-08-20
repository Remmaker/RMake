use std::ffi::OsStr;
use std::fs;
use std::process::ExitStatus;
use crate::config::*;
use crate::cache::*;

#[derive(Default, Debug)]
pub struct BuildConfig {
    compiler: String,
    flags: Option<Vec<String>>,
    pub src: Vec<String>,
    include: Option<Vec<String>>,
    lflags: Option<Vec<String>>,
    lpaths: Option<Vec<String>>,
    target: String
}

fn parse_glob_src(src: Vec<String>) -> Vec<String> {
    let mut ret: Vec<String> = Vec::new();

    // TODO: Maybe handle "folder" wildcard too
    for s in &src {
        if s.contains('*') {
            let extension = s.rsplit_once('.').map(|(_, ext)| ext).unwrap_or("cpp");
            let folder = s.rsplit_once('/').map(|(left, _)| left).unwrap_or(".");

            if let Ok(entries) = fs::read_dir(folder) {
                let matches = entries
                    .filter_map(Result::ok)
                    .map(|e| e.path())
                    .filter(|p| p.extension().and_then(|e| e.to_str()) == Some(extension))
                    .filter_map(|p| p.to_str().map(String::from));
                ret.extend(matches);
            }
        } else {
            ret.push(s.clone());
        }
    }

    ret
}

pub fn parse_build(conf: &Config) -> Result<BuildConfig, ConfigError> {

    let mut build_conf: BuildConfig = BuildConfig::default();
    let hash_build = conf.section.get("build");
    let build = hash_build.unwrap();
    
    if !build.contains_key("compiler") || !build.contains_key("src") {
        return Err(ConfigError::InvalidConfig { message: "Missing 'compiler' and/or 'src' key/value to execute build".into() });
    }

    for (k, v) in build.iter() {
        match k.as_str() {
            "compiler" => {
                if v.split_once(" ").is_some() {
                    return Err(ConfigError::InvalidConfig { message: "Only one compiler is supported at time".into() })
                }
                build_conf.compiler = v.to_string(); 
            },
            "flags" => {
                build_conf.flags.get_or_insert_with(Vec::new).extend(v.split_whitespace().map(|s| s.to_string()));
            },
            "src" => {
                build_conf.src.extend(v.split_whitespace().map(|s| s.to_string()));
            },
            "include" => {
                build_conf.include.get_or_insert_with(Vec::new).extend(v.split_whitespace().map(|s| s.to_string()));
            },
            "lflags" => {
                build_conf.lflags.get_or_insert_with(Vec::new).extend(v.split_whitespace().map(|s| s.to_string()));
            },
            "lpaths" => {
                build_conf.lpaths.get_or_insert_with(Vec::new).extend(v.split_whitespace().map(|s| s.to_string()));
            },
            "target" => {
                if v.split_once(" ").is_some() {
                    return Err(ConfigError::InvalidConfig { message: "Only one target is supported at time".into() })
                }
                build_conf.target = v.to_string();
            },
            _ => {
                eprintln!("Warning: Unknow keyword '{k}'");
            }  
        }
    }
    
    build_conf.src = parse_glob_src(build_conf.src);

    let mut tmp:Vec<String> = Vec::new();
    for s in build_conf.src {
        tmp.push(s.replace("\\", "/"));
    }
    build_conf.src = tmp;
    
    build_conf.flags.get_or_insert_with(Vec::new).push("-c".to_string());
    for s in build_conf.src.clone() {
        cache_build_obj_path(s)?;
    }
    
    Ok(build_conf)
}

pub fn build_obj_file(conf: &BuildConfig) -> Result<CmdOutput, ConfigError> {
    let obj_files = cache_get_all_obj()?;
    
    let mut cmd = std::process::Command::new(conf.compiler.clone());
    cmd.args(obj_files.iter());
    cmd.arg("-o");
    cmd.arg(conf.target.clone());

    if let Some(lpaths) = conf.lpaths.clone() && lpaths.len() > 0 {
        cmd.args(lpaths.iter()
                .map(|s| format!("-L{s}")));
    }

    cmd.args(conf.lflags.clone().get_or_insert(Vec::new()).iter());

    let args: Vec<&OsStr> = cmd.get_args().collect();
    let mut cmdstr: String = conf.compiler.clone();
    for arg in args {
        if let Some(a) = arg.to_str() {
            cmdstr += format!(" {}", a).as_str();      
        }
    }
    eprintln!("RMake: {}", cmdstr);

    let output = cmd.output()
            .map_err(|_| ConfigError::CommandFailed { cmd: cmdstr.clone(), message: "Unexpected".into() })?;

    Ok(CmdOutput { stdout: String::from_utf8_lossy(&output.stdout).into(), stderr: String::from_utf8_lossy(&output.stderr).into(), status: output.status })
}

pub fn execute_build(conf: &BuildConfig) -> Result<CmdOutput, ConfigError> {
    if conf.src.len() == 0 {
        return Ok(CmdOutput { stdout: "RMake cache: Nothing to do".into(), stderr: "".into(), status: ExitStatus::default() })
    }

    let compiler = conf.compiler.clone();
    let is_cl = compiler.ends_with("cl") || compiler.ends_with("cl.exe");

    for s in conf.src.clone() {
        let mut cmd = std::process::Command::new(conf.compiler.clone());
        cmd.args(conf.flags.clone().get_or_insert_with(Vec::new).iter()
            .map(|s| if !is_cl {
                if s.starts_with("--") {
                    s.to_string()
                } else {
                    format!("-{}", s.trim_start_matches('-'))
                }
            } else { s.to_string() }))
        
            .args(conf.include.clone().get_or_insert_with(Vec::new).iter()
                .map(|s| if !is_cl { format!("-I{s}") } else { format!("/I {s}")}))
            
            .arg(s.clone())
            .arg("-o")
            .arg(format!("{}{}.o", cache_get_obj_path(), s));

        let args: Vec<&OsStr> = cmd.get_args().collect();
        let mut cmdstr: String = compiler.clone();
        for arg in args {
            if let Some(a) = arg.to_str() {
                cmdstr += format!(" {}", a).as_str();      
            }
        }

        eprintln!("RMake: {}", cmdstr);
        let output = cmd.output()
            .map_err(|_| ConfigError::CommandFailed { cmd: cmdstr.clone(), message: "Unexpected".into() })?;
        
        if let Some(code) = output.status.code() {
            if code != 0 {
                return Err(ConfigError::CommandFailed { 
                    cmd: cmdstr, 
                    message: format!("stdout:{}\nstderr{}\nexit:{}", String::from_utf8_lossy(&output.stdout), String::from_utf8_lossy(&output.stderr), code) 
                });
            }
        }
        
    }

    build_obj_file(conf)
}

