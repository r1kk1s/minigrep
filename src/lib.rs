use std::collections::VecDeque;
use std::fs;
use std::path::PathBuf;
use std::thread;

pub struct Config {
    pub query: String,
    pub file_paths: Vec<PathBuf>,
    pub ignore_case: bool,
    pub paths_to_ignore: Vec<PathBuf>,
}

impl Config {
    pub fn build(mut args: impl Iterator<Item = String>) -> Result<Self, &'static str> {
        args.next();

        let mut common_args: VecDeque<String> = VecDeque::new();
        let mut ignore_case: bool = false;
        let mut paths_to_ignore = Vec::new();

        for arg in args {
            if let "-i" | "--ignore" = arg.as_str() {
                ignore_case = true;
            } else if let Some(path) = arg
                .strip_prefix("-not=")
                .or_else(|| arg.strip_prefix("--exclude-dir="))
            {
                if let Ok(path) = fs::canonicalize(path) {
                    paths_to_ignore.push(path);
                }
            } else {
                common_args.push_back(arg.to_string());
            }
        }

        if common_args.len() < 2 {
            return Err("Вы должны передать запрос и путь к файлу!");
        }

        let query: String = common_args.pop_front().expect("already checked");

        let mut file_paths = Vec::new();

        for path in common_args {
            if let Ok(path) = fs::canonicalize(path) {
                file_paths.push(path)
            }
        }

        Ok(Self {
            query,
            file_paths,
            ignore_case,
            paths_to_ignore,
        })
    }
}

pub struct Grep {
    pub config: Config,
}

impl Grep {
    pub fn run<'scope, 'env>(&'scope self, scope: &'scope thread::Scope<'scope, 'env>) {
        for file in &self.config.file_paths {
            if self.config.paths_to_ignore.contains(file) {
                continue;
            };

            if file.is_dir() {
                self.print_queried_lines_in_dir_files(file, scope);
            } else if file.is_file() {
                self.print_queried_lines_in_file(file);
            };
        }
    }

    fn print_queried_lines_in_dir_files<'scope, 'env>(
        &'scope self,
        dir_name: &PathBuf,
        scope: &'scope thread::Scope<'scope, 'env>,
    ) {
        if self.config.paths_to_ignore.contains(&dir_name) {
            return;
        }

        let entries = match fs::read_dir(dir_name) {
            Ok(result) => result,
            Err(_) => return,
        };

        for entry in entries {
            let file = match entry {
                Ok(result) => result.path(),
                Err(_) => continue,
            };

            if file.is_dir() {
                scope.spawn(move || self.print_queried_lines_in_dir_files(&file, scope));
            } else if file.is_file() {
                scope.spawn(move || self.print_queried_lines_in_file(&file));
            }
        }
    }

    fn print_queried_lines_in_file(&self, file_path: &PathBuf) {
        let content: String = match fs::read_to_string(file_path) {
            Ok(value) => value,
            Err(_) => return,
        };

        let results: Vec<&str>;

        if self.config.ignore_case {
            results = search_case_insensetive(&self.config.query, &content);
        } else {
            results = search(&self.config.query, &content);
        };

        if !results.is_empty() {
            println!("{file_path:?}");
            for result in results {
                println!("{result}");
            }
        }
    }
}

fn search<'a>(query: &str, content: &'a str) -> Vec<&'a str> {
    content
        .lines()
        .filter(|line| line.contains(query))
        .collect()
}

fn search_case_insensetive<'a>(query: &str, content: &'a str) -> Vec<&'a str> {
    content
        .lines()
        .filter(|line| line.to_lowercase().contains(&query.to_lowercase()))
        .collect()
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

        assert_eq!(vec!["safe, fast, productive."], search(query, content));
    }

    #[test]
    fn case_insensetive() {
        let query = "rUst";
        let content = "\
Rust:
safe, fast, productive.
Pick three.
Trust me.";

        assert_eq!(
            vec!["Rust:", "Trust me."],
            search_case_insensetive(query, content)
        );
    }
}
