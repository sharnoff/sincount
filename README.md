# sincount - count from stdin

This is a fairly simple cli tool, intended for use with shell scripts. It reads
lines from stdin, counting occurances of every given string and periodically
writing those counts as a JSON object to the given file.

# Installing / Usage

This tool is not on crates.io, but can be easily built directly from the
repository:
```
cargo install --git https://sharnoff/sincount.git
```

Typical usage will be in your shell, something like:
```
my-command-with-predictable-output | sincount --delay 500 foo.json
```

# Arguments

- File: A required argument - this is where the counts will be logged.
- delay: '--delay \<TIME\>'. Specifies the number of milliseconds to wait
  between successive syncs of the file.
- no-trim: '--no-trim'. Does not trim whitespace from reading each line (note
  that this will result in trailing newline characters)
- start: '--start=\<FILE\>'. Starts the with an existing count json file. Will
  allow this file to be absent by default, which can be configured with
  'force-file'
- force-file: '--force-file'. Requires that the start file exist
