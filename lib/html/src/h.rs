use crate::{attrs::Attrs, Element};

pub struct H {
    val: String,
    size: u8,
    attrs: Attrs,
}

impl H {
    pub fn new(val: &str, size: u8) -> Self {
        Self {
            val: val.into(),
            size,
            attrs: Attrs::default(),
        }
    }

    pub fn set_attr(mut self, attrs: Attrs) -> Self {
        self.attrs = attrs;
        self
    }
}

impl Element for H {
    fn build(&self) -> String {
        format!(
            "<h{} {}>{}</h{}>",
            self.size, self.attrs, self.val, self.size
        )
    }
}
