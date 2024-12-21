use minigrep::{Config, Grep};
use std::env;
use std::process;
use std::thread;
use std::time::Instant;

fn main() {
    let config: Config = Config::build(env::args()).unwrap_or_else(|err| {
        eprintln!("Ошибка при парсинге аргументов: {err}");
        process::exit(1);
    });

    let grep = Grep { config };

    let start = Instant::now();
    thread::scope(|scope| grep.run(scope));
    let duration = start.elapsed();

    println!("time: {duration:?}");
}
