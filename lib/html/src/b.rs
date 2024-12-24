use crate::{attrs::Attrs, Element};

pub struct B {
    val: String,
    attrs: Attrs,
}

impl B {
    pub fn new(val: &str, attrs: Attrs) -> Self {
        Self {
            val: val.into(),
            attrs,
        }
    }
}

impl Element for B {
    fn build(&self) -> String {
        format!("<b {}>{}</b>", self.attrs, self.val)
    }
}
