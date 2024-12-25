use crate::Element;

pub struct S {
    val: String,
}

impl S {
    pub fn create(val: &str) -> Box<dyn Element> {
        Box::new(Self { val: val.into() })
    }

    pub fn create_nbsp() -> Box<dyn Element> {
        S::create("&nbsp;")
    }
}

impl Element for S {
    fn build(&self) -> String {
        self.val.clone()
    }
}
