use crate::Element;

pub struct Script {
    url: String,
}

impl Script {
    pub fn new(url: &str) -> Self {
        Self { url: url.into() }
    }
}

impl Element for Script {
    fn build(&self) -> String {
        format!(r#"<script src="{}"></script>"#, self.url)
    }
}
