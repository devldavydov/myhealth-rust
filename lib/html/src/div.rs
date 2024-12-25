use crate::Element;

pub struct Div {
    class: String,
    elements: Vec<Box<dyn Element>>,
}

impl Div {
    pub fn new(class: &str) -> Self {
        Self {
            class: class.into(),
            elements: Vec::default(),
        }
    }

    pub fn new_container() -> Self {
        Div::new("container")
    }

    pub fn add_element(mut self, element: Box<dyn Element>) -> Self {
        self.elements.push(element);
        self
    }
}

impl Element for Div {
    fn build(&self) -> String {
        let mut div = format!(r#"<div class="{}">"#, self.class);

        for elem in &self.elements {
            div.push_str(&elem.build());
        }

        div.push_str("</div>");

        div
    }
}
