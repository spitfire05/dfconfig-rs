//! dfconfig is a lib for parsing and manipulating Dwarf Fortress' `init.txt` and `d_init.txt` config files (and possibly many others using the same format).
//! This lib's functionality has been specifically tailored to behave similar as DF internal parser, which implies:
//!
//! * [`Config::get`] returns the last occurrence value, if config specifies the key more than once.
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
//! let mut conf = Config::read_str(read_to_string(path)?);
//!
//! // Read some value
//! let sound = conf.get("SOUND");
//!
//! // Modify and save the config
//! conf.set("VOLUME", "128");
//! write(path, conf.print())?;
//! # Ok::<(), io::Error>(())
//! ```

#[macro_use]
extern crate lazy_static;

use std::collections::HashMap;

use regex::Regex;

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
    pub fn read_str<T: AsRef<str>>(input: T) -> Self {
        lazy_static! {
            static ref RE: Regex = Regex::new(r"^\[([\w\d]+):([\w\d:]+)\]$").unwrap();
        }
        let mut lines = Vec::<Line>::new();
        for l in input.as_ref().lines() {
            let lt = l.trim_end();

            if lt.is_empty() {
                lines.push(Line::Blank);
                continue;
            }

            let captures = RE.captures(lt);
            match captures {
                Some(c) => lines.push(Line::Entry(Entry::new(
                    c.get(1).unwrap().as_str().to_owned(),
                    c.get(2).unwrap().as_str().to_owned(),
                ))),
                None => lines.push(Line::Comment(l.to_owned())),
            };
        }

        Self { lines }
    }

    /// Tries to retrieve the value for `key`.
    /// If the key is defined more than once, returns the value of the last occurrence.
    pub fn get<T: AsRef<str>>(&self, key: T) -> Option<&str> {
        self.lines.iter().rev().find_map(|x| match x {
            Line::Entry(entry) => {
                if entry.get_key() == key.as_ref() {
                    Some(entry.get_value())
                } else {
                    None
                }
            }
            _ => None,
        })
    }

    /// Sets all the occurrences of `key` to `value`
    ///
    /// # Panics
    ///
    /// Panics if `key` or `value` is either empty or non-alphanumeric.
    pub fn set<T: AsRef<str>, U: Into<String>>(&mut self, key: T, value: U) {
        let key = key.as_ref();
        let value = value.into();
        if key.is_empty()
            || !key.chars().all(|x| x.is_alphanumeric())
            || value.is_empty()
            || !value.chars().all(|x| x.is_alphanumeric())
        {
            panic!("Both key and value have to be non-empty alphanumeric strings!")
        }
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

    /// Removes all occurrences of `key` from this `Config`. Returns the number of keys removed.
    pub fn remove<T: AsRef<str>>(&mut self, key: T) -> usize {
        let mut n: usize = 0;
        loop {
            let to_remove = self.lines.iter().enumerate().find_map(|(i, x)| {
                if let Line::Entry(entry) = x {
                    if entry.get_key() == key.as_ref() {
                        return Some(i);
                    }
                }
                return None;
            });
            match to_remove {
                None => break,
                Some(i) => {
                    self.lines.remove(i);
                    n += 1;
                }
            };
        }
        n
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

    /// Returns an iterator over the `key` strings.
    pub fn keys_iter(&self) -> impl Iterator<Item = &str> + '_ {
        self.lines.iter().filter_map(|x| {
            if let Line::Entry(entry) = x {
                Some(entry.get_key())
            } else {
                None
            }
        })
    }

    /// Returns an iterator over (`key`, `value`) tuples.
    pub fn keys_values_iter(&self) -> impl Iterator<Item = (&str, &str)> + '_ {
        self.lines.iter().filter_map(|x| {
            if let Line::Entry(entry) = x {
                Some((entry.get_key(), entry.get_value()))
            } else {
                None
            }
        })
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

impl From<Config> for HashMap<String, String> {
    fn from(conf: Config) -> Self {
        let mut output = HashMap::new();
        conf.keys_values_iter().for_each(|(key, value)| {
            output.insert(key.to_owned(), value.to_owned());
        });
        output
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use rand::distributions::Alphanumeric;
    use rand::{thread_rng, Rng};
    use std::fs::read_to_string;
    use std::iter;

    use super::*;

    fn random_alphanumeric() -> String {
        let mut rng = thread_rng();
        iter::repeat(())
            .map(|()| rng.sample(Alphanumeric))
            .map(char::from)
            .take(thread_rng().gen_range(1..50))
            .collect()
    }

    #[test]
    fn test_basic_parse() {
        let key = random_alphanumeric();
        let key2 = random_alphanumeric();
        let value: String = random_alphanumeric();
        let c = Config::read_str(format!("[{}:{}]", key, value));
        assert_eq!(c.get(key).unwrap(), value);
        assert_eq!(c.get(key2), None);
    }

    #[test]
    fn test_multi_value() {
        let key = random_alphanumeric();
        let value: String = format!("{}:{}", random_alphanumeric(), random_alphanumeric());
        let c = Config::read_str(format!("[{}:{}]", key, value));
        assert_eq!(c.get(key).unwrap(), value);
    }

    #[test]
    fn test_basic_set() {
        let key = random_alphanumeric();
        let value: String = random_alphanumeric();
        let mut c = Config::new();
        c.set(&key, &value);
        assert_eq!(c.get(key).unwrap(), value);
    }

    #[test]
    fn test_read_modify() {
        let key_a = random_alphanumeric();
        let value_b: String = random_alphanumeric();
        let value_c: String = random_alphanumeric();
        let key_d = random_alphanumeric();
        let value_e: String = random_alphanumeric();
        let mut c = Config::read_str(format!("[{}:{}]", key_a, value_b));
        assert_eq!(c.get(&key_a).unwrap(), value_b);
        c.set(&key_a, &value_c);
        assert_eq!(c.get(&key_a).unwrap(), value_c);
        c.set(&key_d, &value_e);
        assert_eq!(c.get(&key_a).unwrap(), value_c);
        assert_eq!(c.get(&key_d).unwrap(), value_e);
    }

    #[test]
    fn test_read_file_smoke() {
        let s = read_to_string("test-data/test.init").unwrap();
        let c = Config::read_str(&s);
        s.lines()
            .zip(c.print().lines())
            .for_each(|(a, b)| assert_eq!(a, b));
    }

    #[test]
    fn test_len() {
        let a: String = random_alphanumeric();
        let b: String = random_alphanumeric();
        let c: String = random_alphanumeric();
        let d: String = random_alphanumeric();
        let e: String = random_alphanumeric();
        let f: String = random_alphanumeric();
        let g: String = random_alphanumeric();
        let mut conf = Config::read_str(format!("[{}:{}]\r\nfoo bar\r\n[{}:{}]", a, b, c, d));
        assert_eq!(conf.len(), 2);
        conf.set(&e, &f);
        assert_eq!(conf.len(), 3);
        conf.set(&e, &g);
        assert_eq!(conf.len(), 3);
    }

    #[test]
    #[should_panic]
    fn panic_on_empty_set() {
        let mut c = Config::new();
        c.set("", "");
    }

    #[test]
    #[should_panic]
    fn panic_on_non_alphanumeric_set() {
        let mut c = Config::new();
        c.set("\r", "\n");
    }

    #[test]
    fn test_keys_iter() {
        let a: String = random_alphanumeric();
        let b: String = random_alphanumeric();
        let mut conf = Config::new();
        conf.set(&a, "foo");
        conf.set(&b, "bar");
        let mut iter = conf.keys_iter();
        assert_eq!(Some(a.as_ref()), iter.next());
        assert_eq!(Some(b.as_ref()), iter.next());
        assert_eq!(None, iter.next());
    }

    #[test]
    fn test_remove() {
        let a: String = random_alphanumeric();
        let b: String = random_alphanumeric();
        let c: String = random_alphanumeric();
        let d: String = random_alphanumeric();
        let mut conf = Config::read_str(format!(
            "[{}:foo]\r\n[{}:bar]\r\n[{}:bar2]\r\n[{}:foobar]\r\n[{}:foobar2]",
            a, b, b, c, d
        ));
        assert_eq!(conf.len(), 5);
        assert_eq!(conf.remove(&b), 2);
        assert_eq!(conf.len(), 3);
        assert_eq!(conf.get(&b), None);
        assert_eq!(conf.remove(&a), 1);
        assert_eq!(conf.len(), 2);
        assert_eq!(conf.get(&a), None);
        assert_eq!(conf.remove(random_alphanumeric()), 0);
    }

    #[test]
    fn test_keys_values_iter() {
        let a: String = random_alphanumeric();
        let b: String = random_alphanumeric();
        let mut conf = Config::new();
        conf.set(&a, "foo");
        conf.set(&b, "bar");
        let mut iter = conf.keys_values_iter();
        assert_eq!(Some((a.as_ref(), "foo")), iter.next());
        assert_eq!(Some((b.as_ref(), "bar")), iter.next());
        assert_eq!(None, iter.next());
    }

    #[test]
    fn test_hashmap() {
        let a: String = random_alphanumeric();
        let b: String = random_alphanumeric();
        let mut conf = Config::new();
        conf.set(&a, "foo");
        conf.set(&b, "bar");
        let hash_map_owned: HashMap<String, String> = conf.into();
        assert_eq!(hash_map_owned.len(), 2);
        assert_eq!(hash_map_owned.get(&a).unwrap(), "foo");
        assert_eq!(hash_map_owned.get(&b).unwrap(), "bar");
    }
}
