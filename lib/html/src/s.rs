use crate::Element;

pub struct S {
    val: String,
}

impl S {
    pub fn new(val: &str) -> Self {
        Self { val: val.into() }
    }

    pub fn new_nbsp() -> Self {
        S::new("&nbsp;")
    }
}

impl Element for S {
    fn build(&self) -> String {
        self.val.clone()
    }
}
