use std::fs;
use std::error::Error;
use std::collections::VecDeque;

pub struct Config {
    pub query: String,
    pub file_paths: VecDeque<String>,
    pub ignore_case: bool,
    pub paths_to_ignore: Vec<String>,
}

impl Config {
    pub fn build(args: &[String]) -> Result<Self, &'static str> {
        let mut common_args: VecDeque<String> = VecDeque::new();

        let mut ignore_case: bool = false;
        let mut paths_to_ignore: Vec<String> = Vec::new();

        for arg in args {
            match arg.as_str() {
                "-i" | "--ignore" => ignore_case = true,
                value if value.contains("--exclude-dir=") || value.contains("-not=") => {
                    let index = value.find("=").expect("already checked") + 1;
                    paths_to_ignore.push(value[index..].to_string());
                },
                _ => common_args.push_back(arg.to_string()),
            }
        }

        if common_args.len() < 3 {
            return Err("Вы должны передать запрос и путь к файлу!");
        }

        let _ = common_args.pop_front();
        let query: String = common_args.pop_front().expect("already checked");

        Ok(Self { query, file_paths: common_args, ignore_case, paths_to_ignore })
    }
}

pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
    for file in config.file_paths {
        let path = match fs::canonicalize(&file) {
            Ok(result) => result,
            Err(_) => return Ok(()),
        };

        if config.paths_to_ignore.contains(&file) {
            return Ok(())
        };

        if path.is_dir() {
            print_queried_lines_in_dir_files(&file, &config.query, config.ignore_case, &config.paths_to_ignore);
        } else if path.is_file() {
            print_queried_lines_in_file(&file, &config.query, config.ignore_case);
        }
    }
    Ok(())
}

fn print_queried_lines_in_dir_files(file_path: &str, query: &str, ignore_case: bool, paths_to_ignore: &Vec<String>) {
    if paths_to_ignore.contains(&file_path.to_string()) {
        return;
    }

    let entries = match fs::read_dir(file_path) {
        Ok(result) => result,
        Err(_) => return,
    };

    for entry in entries {
        let entry: fs::DirEntry = match entry {
            Ok(result) => result,
            Err(_) => continue,
        };

        let file = entry.path();
        let path = match file.to_str() {
            Some(value) => value,
            None => continue,
        };
        if file.is_dir() {
            print_queried_lines_in_dir_files(path, query, ignore_case, paths_to_ignore)
        } else {
            print_queried_lines_in_file(path, query, ignore_case);
        }
    }
}

fn print_queried_lines_in_file(file_path: &str, query: &str, ignore_case: bool) {
    let content: String = match fs::read_to_string(file_path) {
        Ok(value) => value,
        Err(_) => return,
    };

    let results: Vec<String>;

    if ignore_case {
        results = search_case_insensetive(query, &content);
    } else {
        results = search(query, &content);
    };

    if !results.is_empty() {
        println!("{file_path}");
        for result in results {
            println!("{result}");
        }
    }
}

pub fn search(query: &str, content: &str) -> Vec<String> {
    let mut result = Vec::new();

    for (row, line) in content.lines().enumerate() {
        match line.find(&query) {
            Some(column) => result.push(format!("{row}:{column} {line}")),
            None => continue,
        };
    }

    result
}

pub fn search_case_insensetive(query: &str, content: &str) -> Vec<String> {
    let query = query.to_lowercase();
    let mut result = Vec::new();

    for (row, line) in content.lines().enumerate() {
        match line.to_lowercase().find(&query) {
            Some(column) => result.push(format!("{row}:{column} {line}")),
            None => continue,
        };
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn case_sensetive() {
        let query = "duct";
        let content = "\
Rust:
safe, fast, productive.
Pick three.
Duct tape.";

        assert_eq!(vec!["1:15 safe, fast, productive."], search(query, content));
    }

    #[test]
    fn case_insensetive() {
        let query = "rUst";
        let content = "\
Rust:
safe, fast, productive.
Pick three.
Trust me.";

        assert_eq!(vec!["0:0 Rust:", "3:1 Trust me."], search_case_insensetive(query, content));
    }
}
