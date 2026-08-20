use std::{collections::VecDeque, fs, io::{self, ErrorKind::AlreadyExists, Write}, time::UNIX_EPOCH};

use crate::config::ConfigError;

pub const RM_CACHE: &str = ".rm_cache";
pub const INCREM_FILE: &str = "incremental.cache";
pub const OBJ_FOLDER: &str = "obj";

fn cache_increm_file() -> String {
    RM_CACHE.to_owned() + "/" + INCREM_FILE
}

pub fn cache_get_obj_path() -> String {
    RM_CACHE.to_owned() + "/" + OBJ_FOLDER + "/"
}

pub fn cache_build_obj_path(mut s: String) -> Result<(), ConfigError> {
    s = s.replace("\\", "/");
    
    let resopt = s.rsplit_once("/"); 
    if let Some(p) = resopt {
        let respath = p.0.to_string();
        let res: io::Result<()> = fs::create_dir(cache_get_obj_path() + &respath);
        
        if let Some(reserr) = res.err() {
            if reserr.kind() != AlreadyExists {
                return Err(ConfigError::CommandFailed { cmd:"Create cache".into(), message: "Failed to create rmake cache directory".into() });
            }
        }
    }
    
    Ok(())
}

pub fn cache_get_all_obj() -> Result<Vec<String>, ConfigError> {
    let mut ret: Vec<String> = Vec::new();

    let mut folder_list: VecDeque<String> = VecDeque::new();
    folder_list.push_back(cache_get_obj_path());

    while !folder_list.is_empty() {
        if let Some(current_buf) = folder_list.pop_front() {
            if let Ok(entries) = fs::read_dir(current_buf) {
                for entry in entries {
                    if let Ok(entry) = entry {
                        if let Ok(file_type) = entry.file_type() {
                            if file_type.is_dir() {
                                if let Some(respath) = entry.path().to_str() {
                                    folder_list.push_back(respath.to_string());
                                }
                            } else if file_type.is_file() {
                                if let Some(filepath) = entry.path().to_str() {
                                    ret.push(filepath.to_string().replace("\\", "/"));
                                }
                            }
                        } else {
                            eprintln!("RMake error: Couldn't get file type for {:?}", entry.path());
                        }
                    }
                }
            }
        }
    }
    
    Ok(ret)
}

#[derive(Default, Hash, Clone)]
struct CacheData {
    size: u64,
    mtime: u64,
}
#[derive(Default, Clone)]
pub struct CacheMetadata {
    map: std::collections::HashMap<String, CacheData>,
}

pub fn cache_create(_from_clean: bool) -> Result<i32, ConfigError> {
    let res: io::Result<()> = fs::create_dir(RM_CACHE);
    if let Some(reserr) = res.err() {
        if reserr.kind() != AlreadyExists {
            return Err(ConfigError::CommandFailed { cmd:"Create cache".into(), message: "Failed to create rmake cache directory".into() });
        }
    }

    let res: io::Result<()> = fs::create_dir(cache_get_obj_path());
    if let Some(reserr) = res.err() {
        if reserr.kind() != AlreadyExists {
            return Err(ConfigError::CommandFailed { cmd:"Create cache".into(), message: "Failed to create rmake cache directory".into() });
        }
    }

    if !fs::exists(cache_increm_file()).map_err(|_| ConfigError::CommandFailed { cmd: "File exist".into(), message: "Failed to check wheter the cache file exist or not".into() })? {
        let resfile = fs::File::create(cache_increm_file());
        if resfile.is_err() {
            return Err(ConfigError::CommandFailed { cmd: "Create cache file".into(), message: "Failed to create incremental cache file".into() });
        }
    }

    Ok(0)
}

pub fn cache_clean() -> Result<i32, ConfigError> {
    let res: io::Result<()> = fs::remove_dir_all(RM_CACHE);
    if res.is_err() {
        return Err(ConfigError::CommandFailed { cmd: "Clean cache".into(), message: "Failed to clean rmake cache directory".into() });
    }
    cache_create(true)
}

pub fn cache_get_current() -> Result<CacheMetadata, ConfigError> {
    let mut metavec: CacheMetadata = CacheMetadata::default();
    
    let data = std::fs::read_to_string(cache_increm_file())
        .map_err(|_| ConfigError::CommandFailed { cmd: "Open file".into(), message: format!("Failed to open cache incremental file {}", INCREM_FILE) })?;
    if data.len() == 0 {
        return Ok(metavec);
    }

    let datas: Vec<&str> = data.split("\n").collect();
    let mut count = 0;
    for dat in datas {
        if dat.len() == 0 {
            break;
        }

        count += 1;
        let line: Vec<&str> = dat.split(" ").collect();
        if line.len() != 3 {
            return Err(ConfigError::InvalidSyntax { line: count, message: "Cache file probably corrupted, either clean it or fix it".into() });
        }

        let name  = line[0].to_string();
        let mtime = line[1].parse::<u64>().map_err(|_| ConfigError::CommandFailed { cmd: "Cast".into(), message: format!("Failed to cast {} in u64 for second parameter at line {}", line[1], count) })?;
        let size = line[2].parse::<u64>().map_err(|_| ConfigError::CommandFailed { cmd: "Cast".into(), message: format!("Failed to cast {} in u64 for third parameter at line {}", line[2], count) })?;
        metavec.map.insert(name, CacheData { size: size, mtime: mtime });
    }

    Ok(metavec)
}

pub fn cache_compute_src(src: Vec<String>) -> Result<CacheMetadata, ConfigError> {
    let mut ret: CacheMetadata = CacheMetadata::default();
    
    for dat in src {
        let file = fs::File::open(dat.clone())
        .map_err(|_| ConfigError::CommandFailed { cmd: "Open file".into(), message: format!("Failed to open file {}", dat) })?;
        let meta = file.metadata().map_err(|_| ConfigError::InvalidConfig { message: "TODO THIS ERROR #ER1:".into() } )?;
        let tmp =  meta.modified().map_err(|_| ConfigError::InvalidConfig { message: "TODO THIS ERROR #ER2".into() })?;
        let seconds = tmp
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs(); 
    
        let mtime =  seconds;
        let name = dat;
        let size = meta.len();

        ret.map.insert(name, CacheData { size: size, mtime: mtime });
    }

    Ok(ret)
}

pub fn cache_compute_diff(src: Vec<String>, srccache: CacheMetadata, cached: CacheMetadata) -> Result<Vec<String>, ConfigError> {
    let mut ret: Vec<String> = Vec::new();
    if cached.map.len() == 0 { return Ok(src); }

    for src in src {
        if cached.map.contains_key(&src) {
            if let Some(cvalue) = cached.map.get(&src.clone()) && let Some(svalue) = srccache.map.get(&src.clone()) {
                if cvalue.mtime != svalue.mtime || cvalue.size != svalue.size {
                    ret.push(src);
                }
            } else {
                ret.push(src);
            }
        } else {
            ret.push(src);
        }
    }
    
    Ok(ret)
}

pub fn cache_update(srccache: CacheMetadata) -> Result<(), ConfigError> {
    let _ = fs::remove_file(cache_increm_file());
    let mut resfile = fs::File::create(cache_increm_file()).map_err(|_| ConfigError::CommandFailed { cmd: "Create cache file".into(), message: "Failed to create incremental cache file".into() })?;
    
    for (k,v) in srccache.map {
        let line: String = format!("{} {} {}", k, v.mtime, v.size);
        resfile.write(line.as_bytes()).map_err(|_| ConfigError::CommandFailed { cmd: "Write file".into(), message: "Failed to write cache file".into() })?;
        resfile.write("\n".as_bytes()).map_err(|_| ConfigError::CommandFailed { cmd: "Write file".into(), message: "Failed to write cache file".into() })?;
    }

    Ok(())
}