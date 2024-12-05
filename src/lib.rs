use std::fs;
use std::error::Error;

pub struct Config {
    pub query: String,
    pub file_path: String,
    pub ignore_case: bool,
    pub recursive: bool,
    pub paths_to_ignore: Vec<String>,
}

impl Config {
    pub fn build(args: &[String]) -> Result<Self, &'static str> {
        let mut common_args: Vec<&String> = Vec::new();

        let mut ignore_case: bool = false;
        let mut recursive: bool = false;
        let mut paths_to_ignore: Vec<String> = Vec::new();

        for arg in args {
            if arg.contains("-") {
                match arg.as_str() {
                    "-i" | "--ignore" => ignore_case = true,
                    "-R" | "-r" => recursive = true,
                    "-iR" | "-Ri" | "-ri" | "-ir" => {
                        ignore_case = true;
                        recursive = true;
                    },
                    value if value.contains("--exclude-dir=") => {
                        let index = value.find("=").expect("already checked") + 1;
                        paths_to_ignore.push(value[index..].to_string());
                    },
                    _ => (),
                }
            } else {
                common_args.push(arg);
            }
        }

        if common_args.len() != 3 {
            return Err("Вы должны передать запрос и путь к файлу!");
        }

        let query: String = common_args[1].clone();
        let file_path: String = common_args[2].clone();

        Ok(Self { query, file_path, ignore_case, recursive, paths_to_ignore })
    }
}

pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
    if config.recursive {
        print_queried_lines_in_dir_files(&config.file_path, &config.query, config.ignore_case, &config.paths_to_ignore);
    } else {
        print_queried_lines_in_file(&config.file_path, &config.query, config.ignore_case);
    };

    Ok(())
}

fn print_queried_lines_in_dir_files(file_path: &str, query: &str, ignore_case: bool, paths_to_ignore: &Vec<String>) {
    if paths_to_ignore.contains(&file_path.to_string()) {
        return;
    }

    let entries = match fs::read_dir(file_path) {
        Ok(result) => result,
        Err(_) => {
            eprintln!("Не удалось прочитать директорию {file_path}!");
            return;
        }
    };

    for entry in entries {
        let entry: fs::DirEntry = match entry {
            Ok(result) => result,
            Err(_) => {
                eprintln!("Не удалось прочитать файл {file_path}!");
                continue;
            }
        };

        let file = entry.path();
        let path = match file.to_str() {
            Some(value) => value,
            None => {
                eprintln!("Не смог прочитать файл {:?}", &file);
                continue;
            },
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
        Err(_) => {
            eprintln!("Не удалось прочитать файл {file_path}");
            return ();
        }
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
