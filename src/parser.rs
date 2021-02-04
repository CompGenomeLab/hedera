use std::{
    cmp::{max, min},
    collections::HashMap,
    fs::File,
    io::Read,
};

use csv::ReaderBuilder;
use rust_lapper::{Interval, Lapper};
use serde::Deserialize;

#[derive(Deserialize)]
struct BED<'a> {
    chrom: &'a str,
    start: u64,
    stop: u64,
    rest: Vec<&'a str>,
}

type Iv = Interval<u64, char>;

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

pub fn intersect(
    regions: &str,
    reads: &[&str],
    rp: &str,
    left: u64,
    right: u64,
) -> anyhow::Result<Vec<Vec<(Iv, Iv)>>> {
    let mut regions_reader = ReaderBuilder::new()
        .delimiter(b'\t')
        .has_headers(false)
        .from_path(regions)?;

    let mut regions_map: HashMap<String, Vec<Iv>> = HashMap::new();
    for el in regions_reader.records() {
        let el = el?;
        let mut row: BED = el.deserialize(None)?;
        match rp {
            "center" => {
                let rcenter = (row.stop - row.start) / 2 + row.start;
                row.start = rcenter - left;
                row.stop = rcenter + right;
            }
            "start" => {
                row.stop = row.start + right;
                row.start -= left;
            }
            "end" => {
                row.start = row.stop - left;
                row.stop += right;
            }
            _ => unreachable!(),
        }
        let iv = Iv {
            start: row.start,
            stop: row.stop,
            val: row.rest.get(2).unwrap_or(&".").parse()?,
        };

        if let Some(region_vec) = regions_map.get_mut(row.chrom) {
            region_vec.push(iv);
        } else {
            regions_map.insert(row.chrom.to_string(), vec![iv]);
        }
    }
    // When creating lapper object this vector will be sorted immediately
    // for el in regions_map.values_mut() {
    //     el.sort_unstable();
    // }

    // (read, region)
    let mut intersects: Vec<Vec<(Iv, Iv)>> = Vec::new();

    for read in reads {
        let mut read_reader = ReaderBuilder::new()
            .delimiter(b'\t')
            .has_headers(false)
            .from_path(read)?;
        let mut read_map: HashMap<String, Vec<Iv>> = HashMap::new();
        for el in read_reader.records() {
            let el = el?;
            let row: BED = el.deserialize(None)?;
            let iv = Iv {
                start: row.start,
                stop: row.stop,
                val: row.rest.get(2).unwrap_or(&".").parse()?,
            };

            if let Some(read_vec) = read_map.get_mut(row.chrom) {
                read_vec.push(iv);
            } else {
                read_map.insert(row.chrom.to_string(), vec![iv]);
            }
        }
        for el in read_map.values_mut() {
            el.sort_unstable();
        }

        let mut intersect: Vec<(Iv, Iv)> = Vec::new();

        for (key, value) in regions_map.iter() {
            if let Some(read_vec) = read_map.get(key) {
                let mut cursor = 0;
                let regions_lapper = Lapper::new(value.to_owned());
                for el in read_vec {
                    regions_lapper
                        .seek(el.start, el.start, &mut cursor)
                        .for_each(|i| {
                            intersect.push((el.to_owned(), i.to_owned()));
                        })
                }
            }
        }
        intersects.push(intersect);
    }

    Ok(intersects)
}

pub fn coverage(input: &[(Iv, Iv)], left: u64, right: u64) -> anyhow::Result<Vec<f64>> {
    let size = (left + right) as usize;
    let mut cov = vec![0f64; size + 1];
    for (read, region) in input {
        let s = max(read.start, region.start);
        let e = min(read.stop, region.stop);
        let ss = s - region.start;
        let ee = e - s + 1 + ss;
        let mut s_cov = vec![0f64; size + 1];
        for item in s_cov.iter_mut().take(ee as usize).skip(ss as usize) {
            *item += 1f64;
        }
        if region.val == '-' {
            s_cov.reverse();
        }
        cov.iter_mut().enumerate().for_each(|(i, elem)| {
            *elem += s_cov[i];
        })
    }

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
