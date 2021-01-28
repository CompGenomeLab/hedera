use std::path::Path;

use anyhow::{anyhow, Context};
use clap::{load_yaml, App};
pub mod parser;
pub mod plot;

fn main() -> anyhow::Result<()> {
    let yaml = load_yaml!("cli.yml");
    let matches = App::from(yaml).get_matches();

    match matches.subcommand() {
        Some(("reference-point", sub_m)) => {
            let regions: &str = sub_m.value_of("regions").unwrap();
            let reads: Vec<&str> = sub_m.values_of("reads").unwrap().collect();
            let up: u64 = sub_m.value_of_t("upstream").unwrap();
            let down: u64 = sub_m.value_of_t("downstream").unwrap();
            let bin_size: usize = sub_m.value_of_t("binSize").unwrap();
            let referance_point = sub_m.value_of("referencePoint").unwrap();
            let plot_path = sub_m.value_of("outFileName").unwrap();
            let plot_title = sub_m.value_of("plotTitle").unwrap_or(
                Path::new(regions)
                    .file_stem()
                    .with_context(|| format!("{} is not a file", regions))?
                    .to_str()
                    .unwrap(),
            );
            let plot_height: u32 = sub_m.value_of_t("plotHeight").unwrap();
            let plot_width: u32 = sub_m.value_of_t("plotWidth").unwrap();

            // TODO: Allow more than 2 reads for relative mode
            if sub_m.is_present("relative") {
                if reads.len() == 2 {
                    let mut plot_data = Vec::new();
                    let extended_regions_0 =
                        parser::extend_reads(regions, referance_point, up, down)?;
                    let intersect_0 = parser::intersect(extended_regions_0.as_str(), reads[0])?;
                    let coverage_0 = parser::coverage(intersect_0.as_str(), up, down)?;
                    let norm_reads_0 = parser::normalize(&coverage_0, reads[0], regions, bin_size)?;
                    let plot_label_0 = Path::new(reads[0])
                        .file_stem()
                        .with_context(|| format!("{} is not a file", reads[0]))?
                        .to_str()
                        .unwrap();

                    let extended_regions_1 =
                        parser::extend_reads(regions, referance_point, up, down)?;
                    let intersect_1 = parser::intersect(extended_regions_1.as_str(), reads[1])?;
                    let coverage_1 = parser::coverage(intersect_1.as_str(), up, down)?;
                    let norm_reads_1 = parser::normalize(&coverage_1, reads[1], regions, bin_size)?;
                    let plot_label_1 = Path::new(reads[1])
                        .file_stem()
                        .with_context(|| format!("{} is not a file", reads[1]))?
                        .to_str()
                        .unwrap();

                    let norm_reads: Vec<f64> = norm_reads_0
                        .iter()
                        .zip(norm_reads_1.iter())
                        .map(|(n1, n2)| n1 / n2)
                        .collect();
                    let plot_label = format!("{} / {}", plot_label_0, plot_label_1);
                    plot_data.push((norm_reads, plot_label.as_str()));

                    let plot_info = plot::PlotInfo {
                        path: plot_path.to_string(),
                        title: plot_title.to_string(),
                        size: (plot_width, plot_height),
                    };
                    plot::plot_profile(&plot_data, up, down, bin_size, plot_info)?;
                } else {
                    return Err(anyhow!("Only 2 reads are supported in relative mode"));
                }
            } else {
                let mut plot_data = Vec::new();
                for read in reads {
                    let extended_regions =
                        parser::extend_reads(regions, referance_point, up, down)?;
                    let intersect = parser::intersect(extended_regions.as_str(), read)?;
                    let coverage = parser::coverage(intersect.as_str(), up, down)?;
                    let norm_reads = parser::normalize(&coverage, read, regions, bin_size)?;
                    let plot_label = Path::new(read)
                        .file_stem()
                        .with_context(|| format!("{} is not a file", read))?
                        .to_str()
                        .unwrap();
                    plot_data.push((norm_reads, plot_label));
                }

                // TODO: Get more info about plot maybe without arguments
                let plot_info = plot::PlotInfo {
                    path: plot_path.to_string(),
                    title: plot_title.to_string(),
                    size: (plot_width, plot_height),
                };
                plot::plot_profile(&plot_data, up, down, bin_size, plot_info)?;
            }
        }
        // Some(("scale-regions", _sub_m)) => {
        //     println!("Not implemented!")
        // }
        _ => unreachable!(),
    }

    Ok(())
}
