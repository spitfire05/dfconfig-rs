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

#[derive(Clone, Debug)]
enum Line {
    Blank,
    Comment(String),
    Entry(Entry),
}

#[derive(Clone, Debug)]
struct Entry {
    key: String,
    value: String,
}

impl Entry {
    pub fn new(key: String, value: String) -> Self {
        Self { key, value }
    }

    pub fn get_value(&self) -> &str {
        &self.value
    }

    pub fn get_key(&self) -> &str {
        &self.key
    }

    pub fn set_value(&mut self, value: String) {
        self.value = value;
    }
}

/// The main struct of this crate. Represents DF config file, while also providing functions to parse and manipulate the data.
/// See crate doc for example usage.
#[doc(inline)]
#[derive(Clone, Debug)]
pub struct Config {
    lines: Vec<Line>,
}

impl Config {
    /// Creates an empty config.
    pub fn new() -> Self {
        Self { lines: vec![] }
    }

    /// Parse the config from a string.
    pub fn read_str<T: Into<String>>(input: T) -> Self {
        let mut lines = Vec::<Line>::new();
        for l in input.into().lines() {
            let lt = l.trim_end();

            if lt.is_empty() {
                lines.push(Line::Blank);
                continue;
            }

            if lt.starts_with('[') && lt.contains(':') && lt.ends_with(']') {
                if let Some((separator_position, _)) = lt.char_indices().find(|&(_, b)| b == ':') {
                    let (key, value) = lt.split_at(separator_position);
                    lines.push(Line::Entry(Entry::new(
                        key[1..].to_string(),
                        value[1..value.len() - 1].to_string(),
                    )));
                    continue;
                }
            }

            lines.push(Line::Comment(l.to_owned()));
        }

        Self { lines }
    }

    /// Tries to retrieve the value for `key`.
    /// If the key is defined more than once, returns the value of the last occurence.
    pub fn get<T: AsRef<str>>(&self, key: T) -> Option<&str> {
        self.lines.iter().rev().find_map(|x| match x {
            Line::Entry(entry) => {
                if entry.get_key() == key.as_ref() {
                    Some(entry.get_value().clone())
                } else {
                    None
                }
            }
            _ => None,
        })
    }

    /// Sets all the occurences of `key` to [`key`:`value`]
    pub fn set<T: AsRef<str>, U: Into<String>>(&mut self, key: T, value: U) {
        let key = key.as_ref();
        let value = value.into();
        let mut n = 0;
        for e in self.lines.iter_mut() {
            if let Line::Entry(entry) = e {
                if entry.get_key() == key {
                    entry.set_value(value.clone());
                    n += 1;
                }
            }
        }

        if n == 0 {
            self.lines
                .push(Line::Entry(Entry::new(key.to_string(), value)));
        }
    }

    /// Returns number of configuration entries present in this `Config`.
    pub fn len(&self) -> usize {
        self.lines
            .iter()
            .filter(|&x| matches!(x, Line::Entry(_)))
            .count()
    }

    /// Returns true if there are no entries defined in this `Config`.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns the string representing the configuration in its current state (aka what you'd write to the file usually).
    pub fn print(&self) -> String {
        let mut buff = Vec::<String>::with_capacity(self.lines.len());
        for l in self.lines.iter() {
            match l {
                Line::Blank => buff.push("".to_string()),
                Line::Comment(x) => buff.push(x.to_string()),
                Line::Entry(entry) => {
                    buff.push(format!("[{}:{}]", entry.get_key(), entry.get_value()))
                }
            }
        }

        buff.join("\r\n")
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
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
        c.set("E", "G");
        assert_eq!(c.len(), 3);
    }
}
