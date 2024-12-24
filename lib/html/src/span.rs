use crate::Element;

pub struct Span {
    elements: Vec<Box<dyn Element>>,
}

impl Span {
    pub fn new(elements: Vec<Box<dyn Element>>) -> Self {
        Self { elements }
    }
}

impl Element for Span {
    fn build(&self) -> String {
        let mut span = format!("<span>");

        for elem in &self.elements {
            span.push_str(&elem.build());
        }

        span.push_str("</span>");

        span
    }
}
