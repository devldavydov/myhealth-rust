use crate::Element;

pub struct I {
    val: String,
}

impl I {
    pub fn create(val: &str) -> Box<dyn Element> {
        Box::new(Self { val: val.into() })
    }
}

impl Element for I {
    fn build(&self) -> String {
        format!("<i>{}</i>", self.val)
    }
}
