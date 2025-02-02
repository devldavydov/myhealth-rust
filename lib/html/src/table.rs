use crate::{attrs::Attrs, Element};

pub struct Table {
    header: Vec<String>,
    rows: Vec<Tr>,
    footer: Vec<Box<dyn Element>>,
}

impl Table {
    pub fn new(header: Vec<String>) -> Self {
        Self {
            header,
            rows: Vec::default(),
            footer: Vec::default(),
        }
    }

    pub fn add_row(&mut self, row: Tr) {
        self.rows.push(row);
    }

    pub fn add_footer_element(&mut self, elem: Box<dyn Element>) {
        self.footer.push(elem);
    }

    pub fn as_box(self) -> Box<dyn Element> {
        Box::new(self)
    }
}

impl Element for Table {
    fn build(&self) -> String {
        // Header
        let mut table = r#"
        <table class="table table-bordered table-hover">
		    <thead class="table-light">
			    <tr>
        "#
        .to_string();

        for h in &self.header {
            table.push_str(&format!("<th>{}</th>", h));
        }

        table.push_str(
            r#"
                </tr>
		    </thead>
		    <tbody>
        "#,
        );

        // Rows
        for r in &self.rows {
            table.push_str(&r.build());
        }
        table.push_str(
            r#"
            </tbody>
        "#,
        );

        // Footer
        table.push_str(
            r#"
            <tfoot>
        "#,
        );

        for f in &self.footer {
            table.push_str(&f.build());
        }

        table.push_str(
            r#"
            </tfoot>
        "#,
        );

        // End
        table.push_str(
            r#"
        </table>
        "#,
        );

        table
    }
}

//
//
//

pub struct Tr {
    items: Vec<Td>,
    attrs: Attrs,
}

impl Tr {
    pub fn new() -> Self {
        Self {
            items: Vec::default(),
            attrs: Attrs::default(),
        }
    }

    pub fn set_attrs(mut self, attrs: Attrs) -> Self {
        self.attrs = attrs;
        self
    }

    pub fn add_td(mut self, td: Td) -> Self {
        self.items.push(td);
        self
    }

    pub fn as_box(self) -> Box<dyn Element> {
        Box::new(self)
    }
}

impl Default for Tr {
    fn default() -> Self {
        Self::new()
    }
}

impl Element for Tr {
    fn build(&self) -> String {
        let mut tr = format!("<tr {}>", self.attrs);

        for item in &self.items {
            tr.push_str(&item.build());
        }

        tr.push_str("</tr>");

        tr
    }
}

//
//
//

pub struct Td {
    val: Box<dyn Element>,
    attrs: Attrs,
}

impl Td {
    pub fn new(val: Box<dyn Element>) -> Self {
        Self {
            val,
            attrs: Attrs::default(),
        }
    }

    pub fn set_attrs(mut self, attrs: Attrs) -> Self {
        self.attrs = attrs;
        self
    }
}

impl Element for Td {
    fn build(&self) -> String {
        format!(r#"<td {}>{}</td>"#, self.attrs, &self.val.build())
    }
}
