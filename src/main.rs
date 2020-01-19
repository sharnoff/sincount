extern crate clap;
extern crate serde;
extern crate serde_json;

use std::{
    collections::HashMap,
    io,
    fs,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    thread,
    time::Duration
};

fn write_file(path: &Path, counts: &HashMap<String, usize>) {
    let file = fs::File::create(path).unwrap();
    serde_json::to_writer(file, &counts).unwrap();
}

fn main() {
    let yaml_file = clap::load_yaml!("clap.yml");
    let matches = clap::App::from_yaml(yaml_file).get_matches();

    let file: PathBuf = matches
        .value_of("FILE")
        .unwrap()
        .into();

    let delay = matches
        .value_of("delay")
        .unwrap_or("10000") // ten thousand
        .parse::<u64>()
        .expect("Delay argument must be an integer");

    let trim = !matches.is_present("no-trim");

    // records counts, whether it's changed
    let counts: Arc<Mutex<(HashMap<String, usize>, bool)>> = Arc::new(Mutex::new((HashMap::new(), true)));

    let counts_cloned = counts.clone();
    thread::spawn(move || {
        let sleep_duration = Duration::from_millis(delay);
        loop {
            let mut counts_guard = counts_cloned.lock().unwrap();
            
            if counts_guard.1 {
                write_file(&file, &counts_guard.0);
                counts_guard.1 = false;
            }

            drop(counts_guard);
            thread::sleep(sleep_duration);
        }
    });

    let stdin = io::stdin();
    let mut buf = String::new();

    loop {
        stdin.read_line(&mut buf).unwrap();

        if trim {
            buf.truncate(buf.trim_end().len());
        }

        let mut counts_guard = counts.lock().unwrap();
        let map = &mut counts_guard.0;

        let current_count_mut = match map.get_mut(&buf) {
            Some(c) => c,
            None => {
                map.insert(buf.clone(), 0);
                map.get_mut(&buf).unwrap()
            },
        };

        *current_count_mut += 1;

        counts_guard.1 = true;
        drop(counts_guard);
        
        // TODO: Add options for reallocating the buffer if it gets out of hand
        buf.truncate(0);
    }
}
