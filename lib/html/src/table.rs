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

    pub fn add_row(mut self, row: Tr) -> Self {
        self.rows.push(row);
        self
    }

    pub fn add_footer_element(mut self, elem: Box<dyn Element>) -> Self {
        self.footer.push(elem);
        self
    }
}

impl Element for Table {
    fn build(&self) -> String {
        // Header
        let mut table = format!(
            r#"
        <table class="table table-bordered table-hover">
		    <thead class="table-light">
			    <tr>
        "#
        );

        for h in &self.header {
            table.push_str(&format!("<th>{}</th>", h));
        }

        table.push_str(&format!(
            r#"
                </tr>
		    </thead>
		    <tbody>
        "#
        ));

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
    pub fn new(attrs: Attrs) -> Self {
        Self {
            items: Vec::default(),
            attrs,
        }
    }

    pub fn add_td(mut self, td: Td) -> Self {
        self.items.push(td);
        self
    }
}

impl Element for Tr {
    fn build(&self) -> String {
        let mut tr = format!("< tr {}>", self.attrs);

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
    pub fn new(val: Box<dyn Element>, attrs: Attrs) -> Self {
        Self { val, attrs }
    }
}

impl Element for Td {
    fn build(&self) -> String {
        format!(r#"<td {}>{}</td>"#, self.attrs, &self.val.build())
    }
}
