extern crate clap;
extern crate serde;
extern crate serde_json;

use std::{
    collections::HashMap,
    io::{self, Read},
    fs,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    thread,
    time::Duration
};

fn write_file(path: &Path, counts: &HashMap<String, usize>) {
    let file = fs::File::create(path)
        .unwrap_or_else(|e| panic!("Couldn't open file {:?} to write output:\n{}", path, e));
    serde_json::to_writer(file, &counts)
        .unwrap_or_else(|e| panic!("Failed to write json to file {:?}:\n{}", path, e))
}

// Reads the hashmap from the file. Returns None if the file doesn't
// exist
fn read_file(path: &Path) -> Option<HashMap<String, usize>> {
    match fs::File::open(path) {
        Err(e) => {
            eprintln!("Failed to open start file: {:?}", e);
            None
        },
        Ok(mut f) => {
            let mut s = String::new();
            f.read_to_string(&mut s)
                .unwrap_or_else(|e| {
                    panic!("Couldn't read start file {:?}:\n{}", path, e)
                });
            Some(serde_json::from_str(&s)
                 .unwrap_or_else(|e| {
                     panic!("Failed to deserialize json from start file {:?}:\n{}", path, e)
                 }))
        }
    }
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

    let start = matches
        .value_of("start")
        .map(PathBuf::from);

    let force_file = matches.is_present("force-file");

    let trim = !matches.is_present("no-trim");

    let previous_state = match start {
        None => HashMap::new(),
        Some(path) => {
            let m = read_file(&path);
            if force_file {
                m.unwrap_or_else(|| panic!("Couldn't find start file {:?}", path))
            } else {
                m.unwrap_or_else(|| HashMap::new())
            }
        }
    };

    // records counts, whether it's changed
    let counts: Arc<Mutex<(HashMap<String, usize>, bool)>> = Arc::new(Mutex::new((previous_state, true)));

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
        stdin.read_line(&mut buf).expect("Failed to read stdin");

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
