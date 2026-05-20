use std::path::PathBuf;

fn home_dir() -> PathBuf {
    dirs_sys::home_dir().unwrap_or_else(|| PathBuf::from("/tmp"))
}

mod dirs_sys {
    use std::path::PathBuf;
    pub fn home_dir() -> Option<PathBuf> {
        std::env::var("HOME")
            .or_else(|_| std::env::var("USERPROFILE"))
            .map(PathBuf::from)
            .ok()
    }
}

pub fn episteme_home() -> PathBuf {
    std::env::var("EPISTEME_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| home_dir().join(".episteme"))
}

pub fn data_dir() -> PathBuf {
    episteme_home().join("data")
}

pub fn db_dir() -> PathBuf {
    episteme_home().join("db")
}

pub fn db_path() -> PathBuf {
    db_dir().join("episteme.db")
}

pub fn relations_path() -> PathBuf {
    data_dir().join("relations.json")
}

pub fn code_smells_path() -> PathBuf {
    data_dir().join("code_smells.json")
}

pub fn file_to_entity_path() -> PathBuf {
    data_dir().join("file_to_entity.json")
}

pub fn raw_dir() -> PathBuf {
    episteme_home().join("raw")
}

pub fn log_dir() -> PathBuf {
    episteme_home().join("logs")
}

pub fn pid_file() -> PathBuf {
    episteme_home().join("mcp.pid")
}

pub fn cache_dir() -> PathBuf {
    episteme_home().join("cache")
}
