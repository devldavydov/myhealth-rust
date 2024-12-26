use anyhow::{anyhow, Context, Result};
use minijinja::{context, Environment};
use serde::Serialize;

pub const CHART_COLOR_RED: &str = "rgb(255, 99, 132)";
pub const CHART_COLOR_ORANGE: &str = "rgb(255, 159, 64)";
pub const CHART_COLOR_YELLOW: &str = "rgb(255, 205, 86)";
pub const CHART_COLOR_GREEN: &str = "rgb(75, 192, 192)";
pub const CHART_COLOR_BLUE: &str = "rgb(54, 162, 235)";
pub const CHART_COLOR_PURPLE: &str = "rgb(153, 102, 255)";
pub const CHART_COLOR_GREY: &str = "rgb(201, 203, 207)";

#[derive(Serialize)]
pub struct ChartData {
    pub elem_id: String,
    pub x_labels: Vec<String>,
    pub ctype: String,
    pub datasets: Vec<ChartDataset>,
}

#[derive(Serialize)]
pub struct ChartDataset {
    pub data: Vec<f64>,
    pub label: String,
    pub color: String,
}

pub fn get_chart_snippet(data: ChartData) -> Result<String> {
    let mut env = Environment::new();

    env.add_template(
        "chart",
        r#"
<script>
	function plot() {
		const ctx = document.getElementById('{{ data.elem_id }}');

		new Chart(ctx, {
			type: '{{ data.ctype }}',
			data: {
				labels: [
                {% for x in data.x_labels %}
                    '{{ x }}',
                {% endfor %}
				],
				datasets: [
                {% for ds in data.datasets %}
					{
						label: '{{ ds.label }}',
						data: [
                        {% for d in ds.data %}
                            {{ d }},
                        {% endfor %}
						],
						borderWidth: 2,
						borderColor: '{{ ds.color }}',
						backgroundColor: '{{ ds.color }}'
					},
				{% endfor %}
				]
			}
		});		
	}
	window.onload = plot;
</script>        
    "#,
    )
    .context("add template error")?;
    let tmpl = env.get_template("chart").context("get template error")?;

    tmpl.render(context!(data => data))
        .map_err(|e| anyhow!(e))
        .context("render template error")
}
