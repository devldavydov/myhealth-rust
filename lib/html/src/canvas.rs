use crate::Element;

pub struct Canvas {
    id: String,
}

impl Canvas {
    pub fn create(id: &str) -> Box<dyn Element> {
        Box::new(Self { id: id.into() })
    }
}

impl Element for Canvas {
    fn build(&self) -> String {
        format!(r#"<canvas id="{}"></canvas>"#, self.id)
    }
}
