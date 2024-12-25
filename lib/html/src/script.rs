use crate::Element;

pub struct Script {
    url: String,
}

impl Script {
    pub fn create(url: &str) -> Box<dyn Element> {
        Box::new(Self { url: url.into() })
    }
}

impl Element for Script {
    fn build(&self) -> String {
        format!(r#"<script src="{}"></script>"#, self.url)
    }
}
