use crate::Element;

pub struct Span {
    elements: Vec<Box<dyn Element>>,
}

impl Span {
    pub fn create(elements: Vec<Box<dyn Element>>) -> Box<dyn Element> {
        Box::new(Self { elements })
    }
}

impl Element for Span {
    fn build(&self) -> String {
        let mut span = "<span>".to_string();

        for elem in &self.elements {
            span.push_str(&elem.build());
        }

        span.push_str("</span>");

        span
    }
}
