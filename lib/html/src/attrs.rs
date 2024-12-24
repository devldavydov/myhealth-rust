use core::fmt;
use std::collections::HashMap;

pub struct Attrs(HashMap<String, String>);

impl Attrs {
    pub fn new(attrs: HashMap<String, String>) -> Self {
        Self(attrs)
    }

    pub fn insert(&mut self, k: &str, v: &str) {
        self.0.insert(k.into(), v.into());
    }
}

impl fmt::Display for Attrs {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut s = Vec::with_capacity(self.0.len());
        for (k, v) in &self.0 {
            s.push(format!("{}={}", k, v));
        }

        f.write_str(&s.join(" "))
    }
}
