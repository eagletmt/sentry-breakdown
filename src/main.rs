use plotters::prelude::*;
use wasm_bindgen::JsCast as _;
use wasm_bindgen::UnwrapThrowExt as _;

#[derive(Debug, serde::Deserialize)]
struct Record {
    #[serde(rename = "Date")]
    date: chrono::NaiveDate,
    #[serde(rename = "Project Slug")]
    project_slug: String,
    #[serde(rename = "Total Errors")]
    total_errors: u64,
}

fn main() {
    let window = web_sys::window().expect_throw("window is not defined");
    let document = window
        .document()
        .expect_throw("window.document is not defined");
    let csv_input = document
        .get_element_by_id("csv-input")
        .expect_throw("#csv-input is not found");

    let (tx, mut rx) = futures_channel::mpsc::unbounded();
    let listener =
        gloo_events::EventListener::new(&csv_input, "change", move |evt: &web_sys::Event| {
            tx.unbounded_send(
                evt.current_target()
                    .expect_throw("Event#currentTarget is not defined")
                    .dyn_into::<web_sys::HtmlInputElement>()
                    .expect_throw("Event#currentTarget is not HtmlInputElement"),
            )
            .expect_throw("failed to send change event");
        });
    wasm_bindgen_futures::spawn_local(async move {
        let _guard = listener;
        use futures_util::StreamExt as _;
        while let Some(input) = rx.next().await {
            handle_csv_input(input).await;
        }
    });
}

async fn handle_csv_input(csv_input: web_sys::HtmlInputElement) {
    let file_list = gloo_file::FileList::from(
        csv_input
            .files()
            .expect_throw("HtmlInputElement#files is not defined"),
    );
    let text = gloo_file::futures::read_as_bytes(&file_list[0])
        .await
        .expect_throw("File#text() failed");
    let iter = csv::Reader::from_reader(text.as_slice()).into_deserialize();

    let mut series_hash = std::collections::HashMap::new();
    let mut min_date = chrono::naive::MAX_DATE;
    let mut max_date = chrono::naive::MIN_DATE;
    let mut max_errors = 0u64;
    for result in iter {
        let row: Record = result.expect_throw("failed to deserialize CSV row");
        let series: &mut std::collections::HashMap<chrono::NaiveDate, u64> =
            series_hash.entry(row.project_slug).or_default();
        series.insert(row.date, row.total_errors);
        min_date = min_date.min(row.date);
        max_date = max_date.max(row.date);
        max_errors = max_errors.max(row.total_errors);
    }

    let window = web_sys::window().expect_throw("window is not defined");
    let document = window
        .document()
        .expect_throw("window.document is not defined");

    let header = document
        .create_element("tr")
        .expect_throw("failed to create tr element");
    for key in ["Date", "Project slug", "Total errors"] {
        let th = document
            .create_element("th")
            .expect_throw("failed to create th element");
        th.append_child(&document.create_text_node(key))
            .expect_throw("failed to append text node to th element");
        header
            .append_child(&th)
            .expect_throw("failed to append th element to tr element");
    }
    let thead = document
        .create_element("thead")
        .expect_throw("failed to create thead element");
    thead
        .append_child(&header)
        .expect_throw("failed to append tr element to thead element");

    let tbody = document
        .create_element("tbody")
        .expect_throw("failed to create tbody element");

    let canvas = plotters_canvas::CanvasBackend::new("chart")
        .expect_throw("failed to find #chart element")
        .into_drawing_area();
    canvas
        .fill(&WHITE)
        .expect_throw("failed to fill with WHITE");
    let mut chart = ChartBuilder::on(&canvas)
        .margin(20)
        .caption("Sentry breakdown", FontFamily::SansSerif)
        .x_label_area_size(60)
        .y_label_area_size(90)
        .build_cartesian_2d(min_date..max_date, 0..max_errors)
        .expect_throw("failed to build chart");
    chart
        .configure_mesh()
        .x_desc("Date")
        .y_desc("Total errors")
        .draw()
        .expect_throw("failed to draw mesh");
    let mut all_series: Vec<(
        String,
        std::collections::HashMap<chrono::NaiveDate, u64>,
        u64,
    )> = series_hash
        .into_iter()
        .map(|(name, series)| {
            let sum = series.values().sum();
            (name, series, sum)
        })
        .collect();
    all_series.sort_unstable_by(|a, b| a.2.cmp(&b.2).reverse().then(a.0.cmp(&b.0)));
    all_series.truncate(50);
    for (idx, (name, series, _)) in all_series.iter().enumerate() {
        let color = Palette99::pick(idx);
        let mut series: Vec<(chrono::NaiveDate, u64)> =
            series.iter().map(|(date, count)| (*date, *count)).collect();
        series.sort_unstable();

        for (date, total_errors) in series.iter() {
            let tr = document
                .create_element("tr")
                .expect_throw("failed to create tr element in tbody");
            let date_td = document
                .create_element("td")
                .expect_throw("failed to create td element for date");
            date_td
                .append_child(&document.create_text_node(&format!("{}", date)))
                .expect_throw("failed to append date text to td element");
            let name_td = document
                .create_element("td")
                .expect_throw("failed to create td element for project_slug");
            name_td
                .append_child(&document.create_text_node(name))
                .expect_throw("failed to append project_slug text to td element");
            let total_errors_td = document
                .create_element("td")
                .expect_throw("failed to create td element for total_errors");
            total_errors_td
                .append_child(&document.create_text_node(&format!("{}", total_errors)))
                .expect_throw("failed to append total_errors text to td element");
            tr.append_child(&date_td)
                .expect_throw("failed to append date to tr element");
            tr.append_child(&name_td)
                .expect_throw("failed to append date to project_slug element");
            tr.append_child(&total_errors_td)
                .expect_throw("failed to append date to total_errors element");
            tbody
                .append_child(&tr)
                .expect_throw("failed to append tr element to tbody element");
        }

        chart
            .draw_series(LineSeries::new(series, color.stroke_width(3)))
            .expect_throw("failed to draw line")
            .label(name)
            .legend(move |(x, y)| Rectangle::new([(x, y - 5), (x + 10, y + 5)], color.filled()));
    }
    chart
        .configure_series_labels()
        .draw()
        .expect_throw("failed to draw series label");
    canvas.present().expect_throw("failed to present chart");
    let coord_trans = chart.into_coord_trans();
    let tooltip = document
        .get_element_by_id("tooltip")
        .expect_throw("failed to find #tooltip element");
    let chart_element = document
        .get_element_by_id("chart")
        .expect_throw("failed to find #chart element")
        .dyn_into::<web_sys::HtmlCanvasElement>()
        .expect_throw("failed to cast #chart into HtmlCanvasElement");

    let (tx, mut rx) = futures_channel::mpsc::unbounded();
    let listener =
        gloo_events::EventListener::new(&window, "mousemove", move |evt: &web_sys::Event| {
            let evt = evt
                .dyn_ref::<web_sys::MouseEvent>()
                .expect_throw("failed to cast Event to MouseEvent");
            tx.unbounded_send((evt.client_x(), evt.client_y()))
                .expect_throw("failed to send MouseEvent");
        });

    wasm_bindgen_futures::spawn_local(async move {
        let _guard = listener;
        use futures_util::StreamExt as _;
        while let Some((client_x, client_y)) = rx.next().await {
            if let Some((date, _)) = coord_trans((
                client_x - chart_element.offset_left(),
                client_y - chart_element.offset_top(),
            )) {
                let mut ordered_series: Vec<(&str, u64, usize)> = all_series
                    .iter()
                    .enumerate()
                    .filter_map(|(idx, (name, series, _))| {
                        series.get(&date).map(|&count| (name.as_str(), count, idx))
                    })
                    .collect();
                if ordered_series.is_empty() {
                    tooltip.set_inner_html("");
                    continue;
                }
                ordered_series.sort_unstable_by(|a, b| a.1.cmp(&b.1).reverse().then(a.0.cmp(&b.0)));
                let window = web_sys::window().expect_throw("window is not defined");
                let document = window
                    .document()
                    .expect_throw("window.document is not defined");
                let div = document
                    .create_element("div")
                    .expect_throw("failed to create div element")
                    .dyn_into::<web_sys::HtmlElement>()
                    .unwrap();
                div.set_text_content(Some(&format!("{}", date)));
                let ol = document
                    .create_element("ol")
                    .expect_throw("failed to create ol element");
                for (name, count, idx) in ordered_series {
                    let li = document
                        .create_element("li")
                        .expect_throw("failed to create li element");
                    let span = document
                        .create_element("span")
                        .expect_throw("failed to create span element")
                        .dyn_into::<web_sys::HtmlElement>()
                        .unwrap();
                    span.set_text_content(Some("■"));
                    let (r, g, b) = Palette99::pick(idx).rgb();
                    span.style()
                        .set_css_text(&format!("color: rgb({}, {}, {});", r, g, b));
                    li.append_child(&span)
                        .expect_throw("failed to append span element to li element");
                    li.append_child(&document.create_text_node(&format!("{} {}", name, count)))
                        .expect_throw("failed to append text node to li element");
                    ol.append_child(&li)
                        .expect_throw("failed to append li element to ol element");
                }
                div.append_child(&ol)
                    .expect_throw("failed to append ol element to div element");
                div.style()
                .set_css_text(&format!("position: absolute; padding: 10px; color: rgb(255, 255, 255); background-color: rgba(0, 0, 0, 0.7); left: {}px; top: {}px;", client_x, client_y));
                tooltip.set_inner_html("");
                tooltip
                    .append_child(&div)
                    .expect_throw("failed to append div element to #tooltip");
            } else {
                tooltip.set_inner_html("");
            }
        }
    });

    let table = document
        .create_element("table")
        .expect_throw("failed to create table element");
    table
        .append_child(&thead)
        .expect_throw("failed to append thead element to table element");
    table
        .append_child(&tbody)
        .expect_throw("failed to append thead element to tbody element");

    let root = document
        .get_element_by_id("table")
        .expect_throw("failed to find #table element");
    root.set_inner_html("");
    root.append_child(&table)
        .expect_throw("failed to append table element to #table");
}
