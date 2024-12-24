use crate::{attrs::Attrs, Element};

pub struct I {
    val: String,
    attrs: Attrs,
}

impl I {
    pub fn new(val: &str, attrs: Attrs) -> Self {
        Self {
            val: val.into(),
            attrs,
        }
    }
}

impl Element for I {
    fn build(&self) -> String {
        format!("<i {}>{}</i>", self.attrs, self.val)
    }
}
