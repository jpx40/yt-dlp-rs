use clap::Parser;
pub use dirs::home_dir;
use fs_extra::file;
use itertools::Itertools;
use rayon::iter::IntoParallelRefMutIterator;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::path;
use std::process::Command;
use std::{
    borrow::Cow,
    ffi::OsString,
    i16,
    ops::Index,
    path::{Component, Path, PathBuf, MAIN_SEPARATOR_STR},
};
use walkdir::{DirEntry, WalkDir};

#[derive(Parser, Debug)]
pub struct Cli {
    file: String,
}

#[derive(Default, Debug, Clone, Deserialize, Serialize)]
pub struct Videos {
    pub name: Option<String>,
    pub path: String,
    pub videos: Vec<String>,
}

impl Videos {
    pub fn new(path: String) -> Self {
        Self {
            name: None,
            path,
            videos: vec![],
        }
    }

    pub fn from_file(file: String) -> Result<Self, anyhow::Error> {
        let mut file = file;

        // let _chars: Vec<char> = file.chars().collect();
        //
        // if !file.contains(MAIN_SEPARATOR_STR) || !file.contains('~') || !file.contains("./") {
        //     file = "./".to_string() + file.as_str();
        // }
        let buf = std::fs::read_to_string(file)?;

        let mut res: Videos = toml::from_str(&buf)?;

        res.path = Path::new(
            &expand_tilde(PathBuf::from(&res.path))
                .to_string_lossy()
                .to_string(),
        )
        .to_string_lossy()
        .to_string();

        let path = Path::new(&res.path);

        if !path.exists() {
            std::fs::create_dir_all(&res.path)?;
        }

        Ok(res)
    }

    pub fn to_videos(&mut self) -> Vec<Video> {
        let mut videos: Vec<Video> = Vec::new();
        for v in &self.videos {
            let mut video = Video::new(v.to_string());
            video.path = self.path.clone();
            videos.push(video);
        }
        videos
    }
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Hash)]
pub struct Video {
    pub name: String,
    pub url: String,
    pub path: String,
    pub format: String,
    pub quality: String,
    pub resolution: String,
    pub codec: String,
    pub ext: String,
}

impl Video {
    pub fn new(url: String) -> Self {
        Self {
            name: String::new(),
            url,
            path: String::new(),
            format: "140".to_string(),
            quality: "high".to_string(),
            resolution: "1080".to_string(),
            codec: "h264".to_string(),
            ext: "m4a".to_string(),
        }
    }
}

const YTDL: &str = "yt-dlp";

pub fn run_fast(videos: Vec<Video>) {
    let videos: Vec<Video> = videos.into_iter().unique().collect_vec();
    let mut videos = videos;
    videos.par_iter_mut().for_each(|video| {
        extract(video.clone());
    });
}

fn some_correction(path: &str) -> Result<(), String> {
    let path = Path::new(path);

    if path.is_dir() {
        let walker = walkdir::WalkDir::new(path).into_iter();

        for entry in walker.filter_entry(|e| !is_hidden(e) || !is_dir(e)) {
            if let Ok(entry) = entry {
                if entry.path().is_file() {
                    let s = entry.path().to_string_lossy().to_string();

                    if s.contains("m4a") != s.contains("temp") {
                        correct_filename(
                            entry
                                .path()
                                .parent()
                                .unwrap()
                                .canonicalize()
                                .unwrap()
                                .to_str()
                                .unwrap(),
                            s,
                        );
                    } else if s.contains("temp") {
                        std::fs::remove_file(entry.path()).unwrap();
                    }
                }
            }
        }
    } else {
        return Err("Not a directory".to_string());
    }
    Ok(())
}
fn correct_filename(path: &str, filename: String) {
    let mut chars: Vec<char> = filename.chars().collect();

    if chars[0].to_string() == r#"'"# {
        chars.remove(0);
    }

    if chars[chars.len() - 1].to_string() == r#"'"# {
        chars.remove(chars.len() - 1);
    }

    let mut new_name = chars.iter().collect::<String>();

    let mut tmp: Vec<String>;
    if new_name.contains('[') {
        tmp = new_name.split('[').map(|s| s.to_string()).collect();

        let mut s: String = String::new();
        if filename.contains("m4a") {
            s = "m4a".to_string();
        } else {
            s = tmp[1].split('.').collect::<Vec<&str>>()[1].to_string();
        }
        new_name = tmp[0].clone() + "." + &s;
    }

    new_name = new_name.replace(' ', "_");

    let mut chars: Vec<char> = new_name.chars().collect();

    if chars[chars.len() - 5] == '_' {
        chars.remove(chars.len() - 5);
    }

    if chars[chars.len() - 6] == '_' {
        chars.remove(chars.len() - 6);
    }

    let mut new_name = chars.iter().collect::<String>();

    new_name = new_name.trim().to_string();
    new_name = new_name.replace(" ", "");
    if new_name != filename {
        let path = Path::new(path);

        let old_path = Path::new(path)
            .join(&filename)
            .to_string_lossy()
            .to_string();
        let new_path = path.join(new_name);
        if !new_path.exists() {
            let options = fs_extra::file::CopyOptions::new();
            file::move_file(old_path, new_path, &options).expect("Error moving file");
        } else {
            let old_path = Path::new(path).join(filename).to_string_lossy().to_string();

            std::fs::remove_file(old_path).expect("Error removing file");
        }
    }
}

fn extract(video: Video) {
    let mut cmd = Command::new(YTDL);

    cmd.arg(video.url);
    cmd.arg("--format").arg(video.format);
    cmd.arg("--paths").arg(video.path);
    let output = cmd.output().expect("failed to execute command");
    if !output.status.success() {
        println!("{}", String::from_utf8_lossy(&output.stdout));
    }
}
fn main() {
    let cli = Cli::parse();
    println!("Downloading videos");

    if !cli.file.is_empty() {
        let file = expand_tilde(PathBuf::from(&cli.file))
            .to_string_lossy()
            .to_string();
        let file = Path::new(&file);

        if file.canonicalize().unwrap().exists() {
            let file = file.canonicalize().unwrap().to_string_lossy().to_string();
            let mut videos = Videos::from_file(file).unwrap_or_else(|e| panic!("{}", e));
            run_fast(videos.to_videos());
            some_correction(videos.path.as_str()).unwrap_or_else(|e| panic!("{}", e))
        } else {
            println!("File not found");
        }
    }
}

pub fn expand_tilde<'a, P>(path: P) -> Cow<'a, Path>
where
    P: Into<Cow<'a, Path>>,
{
    let path = path.into();
    let mut components = path.components();
    if let Some(Component::Normal(c)) = components.next() {
        if c == "~" {
            if let Some(mut buf) = home_dir() {
                buf.push(components);
                return Cow::Owned(buf);
            }
        }
    }

    path
}

pub fn is_hidden(entry: &DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| s.starts_with('.'))
        .unwrap_or(false)
}

pub fn is_dir(entry: &DirEntry) -> bool {
    entry.path().is_dir()
}
