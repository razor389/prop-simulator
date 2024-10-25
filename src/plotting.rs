use plotters::prelude::*;

/// Generate a histogram of final account balances with y-axis scaled as a percentage
pub fn plot_histogram(data: &[f64], file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let root = BitMapBackend::new(file_path, (800, 600)).into_drawing_area();
    root.fill(&WHITE)?;

    let min_balance = *data.iter().min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap();
    let max_balance = *data.iter().max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap();

    let total_data_count = data.len() as f64;

    let mut chart = ChartBuilder::on(&root)
        .caption("Histogram of Final Account Balances", ("sans-serif", 20))
        .margin(20)
        .x_label_area_size(30)
        .y_label_area_size(40)
        .build_cartesian_2d(min_balance..max_balance, 0.0..100.0)?; // Set y-axis from 0% to 100%

    chart.configure_mesh()
        .x_desc("Total Payouts - Account Cost") // Set the x-axis label
        .y_desc("Percentage (%)") // Label the y-axis as percentage
        .draw()?;

    // Calculate histogram bins
    let bin_count = 50;
    let bin_width = (max_balance - min_balance) / bin_count as f64;
    let mut histogram = vec![0; bin_count];

    for &balance in data {
        let bin = ((balance - min_balance) / bin_width).floor() as usize;
        if bin < bin_count {
            histogram[bin] += 1;
        }
    }

    // Draw bars for each bin as percentages
    chart.draw_series(
        histogram.iter().enumerate().map(|(i, &count)| {
            let x0 = min_balance + i as f64 * bin_width;
            let x1 = x0 + bin_width;
            let percent = (count as f64 / total_data_count) * 100.0; // Convert count to percentage
            Rectangle::new(
                [(x0, 0.0), (x1, percent)],
                BLUE.filled(),
            )
        }),
    )?;

    root.present()?;
    Ok(())
}
