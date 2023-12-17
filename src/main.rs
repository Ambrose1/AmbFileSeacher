use std::thread;
use std::fs;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::Path;
use std::path::PathBuf;
use once_cell::sync::Lazy;
use std::sync::Mutex;
use regex::Regex;
use std::env;
use colored::*;
use std::collections::HashMap;

static FILE_INFO: Lazy<Mutex<HashMap<PathBuf, Vec<String>>>> = Lazy::new(|| {
    Mutex::new(HashMap::new())
});

fn insert(key:PathBuf, value:String) {
    let mut file_info = FILE_INFO.lock().unwrap();
    file_info.entry(key)
             .and_modify(|values| values.push(value.clone()))
             .or_insert_with(|| vec![value.clone()]);
}

fn search_from_file(path:&Path, keywords: &str) -> io::Result<()>  {
    // 处理文件名
    if let Some(file_name) = path.file_name() {
        if file_name.to_string_lossy().contains(keywords) {
            insert(path.to_path_buf(), "file_name contained keywords".yellow().to_string());
        }
    }

    // 处理文件内容
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    let re = Regex::new(&format!(r"\b{}\b",keywords)).unwrap();

    let mut line_number = 0;
    for line in reader.lines() {
        line_number += 1;
        let line = line?;
        if re.is_match(&line) {
            let formatted_line = re.replace_all(&line, |caps: &regex::Captures| {
                format!("{}", &caps[0].red().bold())
            });
            let result = format!("{} :  {}",line_number, formatted_line);
            insert(path.to_path_buf(), result);
        }
    }
    Ok(())
}
// 修改 search_from_dir 函数，引入多线程
fn search_from_dir(dir: &Path, keyword: &str) -> io::Result<()> {
    let mut handles = vec![];
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            let keyword = keyword.to_string(); // 克隆关键词，以便在闭包中使用
            let handle = thread::spawn(move || {
                let _result = search_from_file(&path, &keyword);
            });
            handles.push(handle);
        } else if path.is_dir() {
            let keyword = keyword.to_string(); // 克隆关键词，以便在闭包中使用
            let handle = thread::spawn(move || {
                let _result = search_from_dir(&path, &keyword);
            });
            handles.push(handle);
        }
    }
    for handle in handles {
        if let Ok(_value) = handle.join() {
            // 处理返回的 Ok 值
        } else {
            // 处理返回的 Err 值
            println!("创建进程失败，检查访问权限");
        }
    }
    Ok(())
}

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: {} <directory> <keyword>", args[0]);
        std::process::exit(1);
    }
    let dir = &args[1];
    let keywords = &args[2];
    let dir_path = Path::new(dir);
    println!("开始检索目录:{} - 关键词:{}",dir_path.display(),keywords);
    search_from_dir(dir_path, keywords)?;

    let file_info = FILE_INFO.lock().unwrap();
    for (path, values) in file_info.iter() {
        println!(" {:} ", path.display());
        for item in values.iter() {
            println!("{:}",item.blue());
        }
    }
    println!("执行结束...");
    Ok(())
}
