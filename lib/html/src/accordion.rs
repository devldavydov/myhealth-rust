use crate::Element;

pub struct Accordion {
    id: String,
    items: Vec<AccordionItem>,
}

impl Accordion {
    pub fn new(id: &str) -> Self {
        Self {
            id: id.into(),
            items: Vec::default(),
        }
    }

    pub fn add_item(&mut self, mut item: AccordionItem) {
        item.set_accordion_id(&self.id);
        self.items.push(item);
    }

    pub fn as_box(self) -> Box<dyn Element> {
        Box::new(self)
    }
}

impl Element for Accordion {
    fn build(&self) -> String {
        let mut acrd = format!(
            r#"
        <div class="accordion" id="{}">
        "#,
            self.id
        );

        for item in &self.items {
            acrd.push_str(&item.build());
        }

        acrd.push_str("</div>");

        acrd
    }
}

pub struct AccordionItem {
    accordion_id: String,
    id: String,
    header: String,
    body: Box<dyn Element>,
}

impl AccordionItem {
    pub fn new(id: &str, header: &str, body: Box<dyn Element>) -> Self {
        Self {
            accordion_id: String::default(),
            id: id.into(),
            header: header.into(),
            body,
        }
    }

    fn set_accordion_id(&mut self, id: &str) {
        self.accordion_id = id.into();
    }
}

impl Element for AccordionItem {
    fn build(&self) -> String {
        let mut item = format!(
            r##"
        <div class="accordion-item">
		<h2 class="accordion-header">
			<button class="accordion-button" type="button" data-bs-toggle="collapse" data-bs-target="#{}"
					aria-expanded="false" aria-controls="{}">
				<b>{}</b>
			</button>
		</h2>
		<div id="{}" class="accordion-collapse collapse" data-bs-parent="#{}">
			<div class="accordion-body">
        "##,
            self.id, self.id, self.header, self.id, self.accordion_id
        );

        item.push_str(&self.body.build());

        item.push_str(
            r#"
        		</div>
		    </div>
	    </div>
        "#,
        );

        item
    }
}
