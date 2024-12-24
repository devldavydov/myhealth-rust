use crate::Element;

pub struct Canvas {
    id: String,
}

impl Canvas {
    pub fn new(id: &str) -> Self {
        Self { id: id.into() }
    }
}

impl Element for Canvas {
    fn build(&self) -> String {
        format!(r#"<canvas id="{}"></canvas>"#, self.id)
    }
}
