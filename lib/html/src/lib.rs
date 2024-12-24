pub mod accordion;
pub mod attrs;
pub mod b;
pub mod canvas;
pub mod div;
pub mod h;
pub mod i;
pub mod s;
pub mod script;
pub mod span;
pub mod table;

pub const CSS_BOOTSTRAP_URL: &str = "https://devldavydov.github.io/css/bootstrap/bootstrap.min.css";
pub const CSS_JS_BOOTSTRAP_URL: &str =
    "https://devldavydov.github.io/js/bootstrap/bootstrap.bundle.min.js";
pub const JS_CHART_URL: &str = "https://devldavydov.github.io/js/chartjs/chart.umd.min.js";

pub trait Element: Send + Sync {
    fn build(&self) -> String;
}

pub struct Builder {
    title: String,
    elements: Vec<Box<dyn Element>>,
}

impl Builder {
    pub fn new(title: &str) -> Self {
        Self {
            title: title.into(),
            elements: Vec::default(),
        }
    }

    pub fn add<E>(&mut self, elements: E)
    where
        E: Iterator<Item = Box<dyn Element>>,
    {
        self.elements.extend(elements);
    }

    pub fn build(&self) -> String {
        let mut doc = format!(
            r#"
        <!doctype html>
        <html lang="ru">
	
        <head>
            <meta charset="utf-8">
            <title>{}</title>
            <link href="{}" rel="stylesheet">
        </head>
        <body>
        "#,
            self.title, CSS_BOOTSTRAP_URL
        );

        for elem in &self.elements {
            doc.push_str(&elem.build());
        }

        doc.push_str(
            r#"
        </body>
        </html>
        "#,
        );

        doc
    }
}
