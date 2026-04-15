//! Signal visualization using plotters.
//!
//! Gated behind the `viz` feature flag.
//! Enable with: `cargo build -p nexcore-vigilance --features viz`
//!
//! Generates SVG charts for pharmacovigilance signal detection results.

use crate::pv::{ContingencyTable, SignalCriteria, calculate_prr, calculate_ror};
use plotters::prelude::*;

/// Render a PRR timeline chart as SVG string.
///
/// Takes a series of (label, ContingencyTable) pairs and renders PRR values
/// with the signal threshold line.
///
/// # Errors
/// Returns error if chart rendering fails.
pub fn render_prr_timeline_svg(
    title: &str,
    data: &[(&str, &ContingencyTable)],
    criteria: &SignalCriteria,
    width: u32,
    height: u32,
) -> Result<String, nexcore_error::NexError> {
    let mut svg_buf = String::new();

    let result: Result<(), nexcore_error::NexError> = (|| {
        let root = SVGBackend::with_string(&mut svg_buf, (width, height)).into_drawing_area();
        root.fill(&WHITE).map_err(|e| e.to_string())?;

        let prr_values: Vec<(f64, f64)> = data
            .iter()
            .enumerate()
            .map(|(i, (_, table))| {
                let result = calculate_prr(table, criteria);
                (i as f64, result.point_estimate)
            })
            .collect();

        let max_prr = prr_values
            .iter()
            .map(|(_, v)| *v)
            .fold(0.0_f64, f64::max)
            .max(3.0);

        let mut chart = ChartBuilder::on(&root)
            .caption(title, ("sans-serif", 20))
            .margin(10)
            .x_label_area_size(40)
            .y_label_area_size(60)
            .build_cartesian_2d(0.0..(data.len() as f64), 0.0..max_prr * 1.1)
            .map_err(|e| e.to_string())?;

        chart
            .configure_mesh()
            .x_desc("Drug-Event Pair")
            .y_desc("PRR")
            .draw()
            .map_err(|e| e.to_string())?;

        // PRR threshold line at 2.0
        chart
            .draw_series(LineSeries::new(
                vec![(0.0, 2.0), (data.len() as f64, 2.0)],
                &RED,
            ))
            .map_err(|e| e.to_string())?
            .label("Threshold (PRR=2.0)")
            .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], RED));

        // PRR values
        chart
            .draw_series(LineSeries::new(prr_values.clone(), &BLUE))
            .map_err(|e| e.to_string())?
            .label("PRR")
            .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], BLUE));

        // Data points
        chart
            .draw_series(
                prr_values
                    .iter()
                    .map(|(x, y)| Circle::new((*x, *y), 3, BLUE.filled())),
            )
            .map_err(|e| e.to_string())?;

        chart
            .configure_series_labels()
            .border_style(BLACK)
            .draw()
            .map_err(|e| e.to_string())?;

        root.present().map_err(|e| e.to_string())?;
        Ok(())
    })();

    result?;
    Ok(svg_buf)
}

/// Render a multi-algorithm comparison chart as SVG string.
///
/// Shows PRR and ROR side-by-side for a set of drug-event pairs.
///
/// # Errors
/// Returns error if chart rendering fails.
pub fn render_signal_comparison_svg(
    title: &str,
    data: &[(&str, &ContingencyTable)],
    criteria: &SignalCriteria,
    width: u32,
    height: u32,
) -> Result<String, nexcore_error::NexError> {
    let mut svg_buf = String::new();

    let result: Result<(), nexcore_error::NexError> = (|| {
        let root = SVGBackend::with_string(&mut svg_buf, (width, height)).into_drawing_area();
        root.fill(&WHITE).map_err(|e| e.to_string())?;

        let prr_values: Vec<(f64, f64)> = data
            .iter()
            .enumerate()
            .map(|(i, (_, table))| {
                let result = calculate_prr(table, criteria);
                (i as f64, result.point_estimate)
            })
            .collect();

        let ror_values: Vec<(f64, f64)> = data
            .iter()
            .enumerate()
            .map(|(i, (_, table))| {
                let result = calculate_ror(table, criteria);
                (i as f64, result.point_estimate)
            })
            .collect();

        let max_val = prr_values
            .iter()
            .chain(ror_values.iter())
            .map(|(_, v)| *v)
            .fold(0.0_f64, f64::max)
            .max(3.0);

        let mut chart = ChartBuilder::on(&root)
            .caption(title, ("sans-serif", 20))
            .margin(10)
            .x_label_area_size(40)
            .y_label_area_size(60)
            .build_cartesian_2d(0.0..(data.len() as f64), 0.0..max_val * 1.1)
            .map_err(|e| e.to_string())?;

        chart
            .configure_mesh()
            .x_desc("Drug-Event Pair")
            .y_desc("Signal Strength")
            .draw()
            .map_err(|e| e.to_string())?;

        // Threshold line
        chart
            .draw_series(LineSeries::new(
                vec![(0.0, 2.0), (data.len() as f64, 2.0)],
                &RED,
            ))
            .map_err(|e| e.to_string())?
            .label("Threshold")
            .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], RED));

        // PRR series
        chart
            .draw_series(LineSeries::new(prr_values, &BLUE))
            .map_err(|e| e.to_string())?
            .label("PRR")
            .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], BLUE));

        // ROR series
        chart
            .draw_series(LineSeries::new(ror_values, &GREEN))
            .map_err(|e| e.to_string())?
            .label("ROR")
            .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], GREEN));

        chart
            .configure_series_labels()
            .border_style(BLACK)
            .draw()
            .map_err(|e| e.to_string())?;

        root.present().map_err(|e| e.to_string())?;
        Ok(())
    })();

    result?;
    Ok(svg_buf)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pv::ContingencyTable;

    #[test]
    fn test_prr_timeline_svg_renders() {
        let tables = vec![
            ContingencyTable::new(10, 90, 100, 9800),
            ContingencyTable::new(20, 80, 200, 9700),
            ContingencyTable::new(5, 95, 50, 9850),
        ];
        let data: Vec<(&str, &ContingencyTable)> = vec![
            ("DrugA-EventX", &tables[0]),
            ("DrugB-EventY", &tables[1]),
            ("DrugC-EventZ", &tables[2]),
        ];
        let criteria = SignalCriteria::evans();

        let svg = render_prr_timeline_svg("Test PRR Timeline", &data, &criteria, 800, 400);
        assert!(svg.is_ok());
        let svg_str = svg.expect("should render");
        assert!(svg_str.contains("<svg"));
    }

    #[test]
    fn test_signal_comparison_svg_renders() {
        let tables = vec![
            ContingencyTable::new(10, 90, 100, 9800),
            ContingencyTable::new(20, 80, 200, 9700),
        ];
        let data: Vec<(&str, &ContingencyTable)> =
            vec![("DrugA-EventX", &tables[0]), ("DrugB-EventY", &tables[1])];
        let criteria = SignalCriteria::evans();

        let svg = render_signal_comparison_svg("Test Comparison", &data, &criteria, 800, 400);
        assert!(svg.is_ok());
        let svg_str = svg.expect("should render");
        assert!(svg_str.contains("<svg"));
    }
}
