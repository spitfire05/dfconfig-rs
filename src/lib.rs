//! dfconfig is a lib for parsing and manipulating Dwarf Fortress' `init.txt` and `d_init.txt` config files (and possibly many others using the same format).
//! This lib's functioanlity has been specifically tailored to behave simmiliar as DF internal parser, which imples:
//!
//! * [`Config::get`] returns the last occurence value, if config specifes the key more than once.
//! * Whitespaces are not allowed at the start of lines, any line not starting with `[` character is treated as a comment.
//!
//! Other notable functionality is that the parser preserves all of the parsed string content, including blank lines and comments.
//!
//! # Examples
//!
//! ```no_run
//! # use std::io;
//! use std::fs::{read_to_string, write};
//! use dfconfig::Config;
//! 
//! // Parse existing config
//! let path = r"/path/to/df/data/init/init.txt";
//! let mut conf = Config::read_str(read_to_string(path).unwrap());
//! 
//! // Read some value
//! let sound = conf.get("SOUND");
//! 
//! // Modify and save the config
//! conf.set("VOLUME", "128");
//! write(path, conf.print())?;
//! # Ok::<(), io::Error>(())
//! ```

/// The main struct of this crate. Represents DF config file, while also providing functions to parse and manipulate the data.
/// See crate doc for example usage.
#[doc(inline)]
#[derive(Clone, Debug)]
pub struct Config {
    lines: Vec<Line>,
}

#[derive(Clone, Debug)]
enum Line {
    Blank,
    Comment(String),
    Entry(String, String),
}

impl Config {
    /// Creates an empty config.
    pub fn new() -> Self {
        Self { lines: vec![] }
    }

    /// Parse the config from a string.
    pub fn read_str<T: AsRef<str>>(input: T) -> Self {
        let mut lines = Vec::<Line>::new();
        for l in input.as_ref().lines() {
            let lt = l.trim_end();

            if lt.is_empty() {
                lines.push(Line::Blank);
                continue;
            }

            if lt.starts_with('[') && lt.contains(':') && lt.ends_with(']') {
                if let Some((separator_position, _)) = lt.char_indices().find(|&(_, b)| b == ':') {
                    let (key, value) = lt.split_at(separator_position);
                    lines.push(Line::Entry(
                        key[1..].to_string(),
                        value[1..value.len() - 1].to_string(),
                    ));
                    continue;
                }
            }

            lines.push(Line::Comment(l.to_owned()));
        }

        Self { lines }
    }

    /// Tries to retrieve the value for `key`.
    /// If the key is defined more than once, returns the value of the last occurence.
    pub fn get(&self, key: &str) -> Option<String> {
        self.lines.iter().rev().find_map(|x| match x {
            Line::Entry(e_key, value) => {
                if e_key == key {
                    Some(value.clone())
                } else {
                    None
                }
            }
            _ => None
        })
    }

    /// Sets all the occurences of `key` to [`key`:`value`]
    pub fn set(&mut self, key: &str, value: &str) {
        for e in self.lines.iter_mut() {
            match e {
                Line::Entry(k, _) => {
                    if k == key {
                        *e = Line::Entry(key.to_string(), value.to_string());
                    }
                }
                _ => {}
            }
        }

        self.lines
            .push(Line::Entry(key.to_string(), value.to_string()));
    }

    /// Returns number of configuration entries present in this `Config`
    pub fn len(&self) -> usize {
        self.lines.iter().filter(|&x| match x {
            Line::Entry(_, _) => true,
            _ => false
        }).count()
    }

    /// Returns the string representing the configuration in its current state (aka what you'd write to the file usually).
    pub fn print(&self) -> String {
        let mut buff = Vec::<String>::with_capacity(self.lines.len());
        for l in self.lines.iter() {
            match l {
                Line::Blank => buff.push("".to_string()),
                Line::Comment(x) => buff.push(x.to_string()),
                Line::Entry(k, v) => buff.push(format!("[{}:{}]", k, v)),
            }
        }

        buff.join("\r\n")
    }
}

#[cfg(test)]
mod tests {
    use std::fs::read_to_string;

    use super::*;

    #[test]
    fn test_basic_parse() {
        let c = Config::read_str("[A:B]");
        assert_eq!(c.get("A").unwrap(), "B");
    }

    #[test]
    fn test_multi_value() {
        let c = Config::read_str("[A:B:C]");
        assert_eq!(c.get("A").unwrap(), "B:C");
    }

    #[test]
    fn test_basic_set() {
        let mut c = Config::new();
        c.set("A", "B");
        assert_eq!(c.get("A").unwrap(), "B");
    }

    #[test]
    fn test_read_modify() {
        let mut c = Config::read_str("[A:B]");
        assert_eq!(c.get("A").unwrap(), "B");
        c.set("A", "C");
        assert_eq!(c.get("A").unwrap(), "C");
        c.set("D", "F");
        assert_eq!(c.get("A").unwrap(), "C");
        assert_eq!(c.get("D").unwrap(), "F");
    }

    #[test]
    fn test_read_file_smoke() {
        let s = read_to_string("test.init").unwrap();
        let c = Config::read_str(s);
        c.print();
    }

    #[test]
    fn test_len() {
        let mut c = Config::read_str("[A:B]\r\nfoo bar\r\n[C:D]");
        assert_eq!(c.len(), 2);
        c.set("E", "F");
        assert_eq!(c.len(), 3);
    }
}