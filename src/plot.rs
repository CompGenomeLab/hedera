use plotters::prelude::*;

trait FloatIterExt {
    fn float_min(&mut self) -> f64;
    fn float_max(&mut self) -> f64;
}

impl<T> FloatIterExt for T
where
    T: Iterator<Item = f64>,
{
    fn float_max(&mut self) -> f64 {
        self.fold(f64::NAN, f64::max)
    }

    fn float_min(&mut self) -> f64 {
        self.fold(f64::NAN, f64::min)
    }
}
// TODO: Change fields to &str
pub struct PlotInfo {
    pub path: String,
    pub title: String,
    pub size: (u32, u32),
}

pub fn plot_profile(
    input: &[(Vec<f64>, &str)],
    left: u64,
    right: u64,
    bs: usize,
    plot_info: PlotInfo,
) -> anyhow::Result<()> {
    let y_high = input
        .iter()
        .map(|line| line.0.iter().cloned().float_max())
        .float_max();
    let y_low = input
        .iter()
        .map(|line| line.0.iter().cloned().float_min())
        .float_min();
    let root = BitMapBackend::new(plot_info.path.as_str(), plot_info.size).into_drawing_area();
    root.fill(&WHITE)?;
    // TODO: Custom x-axis
    let mut chart = ChartBuilder::on(&root)
        .caption(plot_info.title.as_str(), ("Arial", 15).into_font())
        .margin(30)
        .x_label_area_size(30)
        .y_label_area_size(30)
        .build_cartesian_2d(
            (left as f64 * -1f64) - 1f64..right as f64,
            y_low..y_high + (y_high - y_low) / 10f64,
        )?;

    chart.configure_mesh().light_line_style(&WHITE).draw()?;

    for (line_index, line) in input.iter().enumerate() {
        chart
            .draw_series(LineSeries::new(
                line.0[1..line.0.len() - 1]
                    .iter()
                    .enumerate()
                    .map(|(x, y)| ((((x + 1) as f64 * bs as f64) - left as f64), *y)),
                &Palette99::pick(line_index),
            ))?
            .label(line.1)
            .legend(move |(x, y)| {
                PathElement::new(vec![(x, y), (x + 20, y)], &Palette99::pick(line_index))
            });
    }

    chart
        .configure_series_labels()
        .background_style(&WHITE.mix(0.8))
        .border_style(&BLACK)
        .draw()?;

    Ok(())
}
