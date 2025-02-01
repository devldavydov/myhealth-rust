use crate::{attrs::Attrs, Element};

pub struct B {
    val: String,
    attrs: Attrs,
}

impl B {
    pub fn new(val: &str) -> Self {
        Self {
            val: val.into(),
            attrs: Attrs::default(),
        }
    }

    pub fn set_attr(mut self, attrs: Attrs) -> Self {
        self.attrs = attrs;
        self
    }

    pub fn as_box(self) -> Box<dyn Element> {
        Box::new(self)
    }
}

impl Element for B {
    fn build(&self) -> String {
        format!("<b {}>{}</b>", self.attrs, self.val)
    }
}
