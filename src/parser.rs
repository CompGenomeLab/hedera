use std::{
    cmp::{max, min},
    fs::{remove_file, File},
    io::Read,
    process::{Command, Stdio},
};

use anyhow::Context;
use csv::{ReaderBuilder, WriterBuilder};
use serde::{Deserialize, Serialize};
// TODO: fields after first 3 are not required, example implementation from rust-bio
#[derive(Debug, Deserialize, Serialize)]
struct BED6<'a> {
    chrom: &'a str,
    start: u64,
    end: u64,
    name: &'a str,
    score: u16,
    strand: char,
}

const BUF_SIZE: usize = 8 * 1024;

fn count_lines(input: &str) -> anyhow::Result<usize> {
    let mut f = File::open(input)?;
    let mut buf: [u8; BUF_SIZE] = [0; BUF_SIZE];
    let mut line_count: usize = 0;
    while f.read(&mut buf)? != 0 {
        line_count += bytecount::count(&buf, b'\n');
        buf = [0; BUF_SIZE];
    }
    Ok(line_count)
}

pub fn extend_reads(input: &str, rp: &str, left: u64, right: u64) -> anyhow::Result<String> {
    let mut reader = ReaderBuilder::new()
        .delimiter(b'\t')
        .has_headers(false)
        .from_path(input)?;
    let extended_path = format!("{}.extended", input);
    let mut writer = WriterBuilder::new()
        .delimiter(b'\t')
        .has_headers(false)
        .from_path(&extended_path)?;
    for (peak_id, record) in reader.records().enumerate() {
        let record = record?;
        let mut row: BED6 = record.deserialize(None)?;
        let name = format!("peak_{}", peak_id);
        row.name = name.as_str();
        // TODO: Change with ENUM
        match rp {
            "center" => {
                let rcenter = (row.end - row.start) / 2 + row.start;
                row.start = rcenter - left;
                row.end = rcenter + right;
            }
            "start" => {
                row.end = row.start + right;
                row.start -= left;
            }
            "end" => {
                row.start = row.end - left;
                row.end += right;
            }
            _ => unreachable!(),
        }

        writer.serialize(row)?;
    }
    writer.flush()?;
    Ok(extended_path)
}

pub fn coverage(input: &str, left: u64, right: u64) -> anyhow::Result<Vec<f64>> {
    let mut reader = ReaderBuilder::new()
        .delimiter(b'\t')
        .has_headers(false)
        .from_path(input)?;
    let size = (left + right) as usize;
    let mut cov = vec![0f64; size + 1];
    for record in reader.records() {
        let row = record?;
        let read_start: usize = row
            .get(1)
            .context("Failed to parse Read Start from intersect bed")?
            .parse()?;
        let read_end: usize = row
            .get(2)
            .context("Failed to parse Read End from intersect bed")?
            .parse()?;
        let region_start: usize = row
            .get(7)
            .context("Failed to parse Region Start from intersect bed")?
            .parse()?;
        let region_end: usize = row
            .get(8)
            .context("Failed to parse Region End from intersect bed")?
            .parse()?;
        let region_strand: char = row
            .get(11)
            .context("Failed to parse Region Strand from intersect bed")?
            .parse()?;
        let s = max(read_start, region_start);
        let e = min(read_end, region_end);
        let ss = s - region_start;
        let ee = e - s + 1 + ss;
        let mut s_cov = vec![0f64; size + 1];
        for item in s_cov.iter_mut().take(ee).skip(ss) {
            *item += 1f64;
        }
        if region_strand == '-' {
            s_cov.reverse();
        }
        cov.iter_mut().enumerate().for_each(|(i, elem)| {
            *elem += s_cov[i];
        })
    }

    remove_file(input)?;
    Ok(cov[..size].to_vec())
}

// TODO: add different normalization methods
pub fn normalize(
    input: &[f64],
    reads: &str,
    regions: &str,
    bin_size: usize,
) -> anyhow::Result<Vec<f64>> {
    let read_count = count_lines(reads)?;
    let region_count = count_lines(regions)? as f64;
    let tl = bin_size as f64 / 1000f64;
    let tpm_sf = (read_count as f64 / tl) / 1000000f64;
    let sf = 1.0 / (tpm_sf * tl);

    let norm: Vec<f64> = input.iter().map(|elem| elem / region_count).collect();
    let binned = norm
        .chunks(bin_size)
        .map(|chunk| (chunk.iter().sum::<f64>() / bin_size as f64) * sf)
        .collect::<Vec<f64>>();

    Ok(binned)
}

// TODO: Without bedtols ? https://github.com/sstadick/rust-lapper
// TODO: Optional bedtools path
pub fn intersect(regions: &str, reads: &str) -> anyhow::Result<String> {
    let intersect_path = format!("{}.intersect", reads);
    let intersect = File::create(&intersect_path)?;
    Command::new("bedtools")
        .args(&[
            "intersect",
            "-wa",
            "-wb",
            "-f",
            "0.5",
            "-a",
            reads,
            "-b",
            regions,
        ])
        .stdout(Stdio::from(intersect))
        .spawn()?
        .wait_with_output()?;
    remove_file(regions)?;
    Ok(intersect_path)
}
