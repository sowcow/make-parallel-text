mod align;
mod context;
mod html;
mod search;
mod similarity;
mod split;

use align::*;
use anyhow::*;
use clap::Parser;
use glob::glob;
use html::*;
use regex::Regex;
use search::*;
use similarity::*;
use split::*;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use tch::Device;

fn sentences_from_file(file: &str) -> Result<Vec<String>> {
    let text = std::fs::read_to_string(file)?;
    Ok(split_into_sentences(&text))
}

fn produce_html_from_paths(context: &context::Context) -> Result<()> {
    let left_sentences = sentences_from_file(&context.left)?;
    let right_sentences = sentences_from_file(&context.right)?;

    let path = joined_path(&context)?;
    let result_file = context.context.clone() + "/result.json";
    let data = (
        path.clone(),
        left_sentences.clone(),
        right_sentences.clone(),
    );
    std::fs::write(&result_file, serde_json::to_string(&data).unwrap()).unwrap();

    let html = produce_html(&left_sentences, &right_sentences, &path, 3);
    let file = context.context.clone() + "/3-columns.html";
    std::fs::write(&file, html).unwrap();

    let html = produce_html(&left_sentences, &right_sentences, &path, 2);
    let file = context.context.clone() + "/2-columns.html";
    std::fs::write(&file, html).unwrap();

    let html = produce_html(&left_sentences, &right_sentences, &path, 1);
    let file = context.context.clone() + "/1-column.html";
    std::fs::write(&file, html).unwrap();

    Ok(())
}

fn joined_path(context: &context::Context) -> Result<Vec<(usize, usize)>> {
    // Compile the regex to capture X, Y, Z from filenames like "path-X-Y-Z.json"
    let re = Regex::new(r"path-(\d+)-(\d+)-(\d+)\.json$")?;

    // A small struct to hold the filename parts + the actual path
    #[derive(Debug)]
    struct FileMeta {
        path: String,
        x: usize,
        y: usize,
        z: usize,
    }

    // Collect all files matching the pattern (using a wildcard).
    // Adjust the pattern to your actual directory if needed, e.g. "some_dir/path-*-*-*.json"
    let mut files = Vec::new();
    let pattern = context.context.clone() + "/path-*-*-*.json";
    for entry in glob(&pattern)? {
        let path_str = entry?.to_string_lossy().to_string();
        if let Some(filename) = Path::new(&path_str).file_name().and_then(|s| s.to_str()) {
            // Use the regex to parse X, Y, Z
            if let Some(caps) = re.captures(filename) {
                let x = caps[1].parse::<usize>()?;
                let y = caps[2].parse::<usize>()?;
                let z = caps[3].parse::<usize>()?;
                files.push(FileMeta {
                    path: path_str,
                    x,
                    y,
                    z,
                });
            }
        }
    }

    // Sort by X ascending
    files.sort_by(|a, b| a.x.cmp(&b.x));

    // This will store the final joined vector
    let mut all_data: Vec<(usize, usize)> = Vec::new();

    // Process each file in sorted order
    for file_meta in files {
        let file = File::open(&file_meta.path)?;
        let reader = BufReader::new(file);

        // The file contains Vec<(usize, usize)>
        let data: Vec<(usize, usize)> = serde_json::from_reader(reader)?;

        // Apply the Y and Z deltas. If the item is (a, b), then the updated item is (a + Y, b + Z).
        let updated_data = data
            .into_iter()
            .map(|(a, b)| (a + file_meta.y, b + file_meta.z))
            .collect::<Vec<_>>();

        // Push into the final collection
        all_data.extend(updated_data);
    }

    // overlaps caused by half-window movement between each path found
    let all_data = remove_backtracks(&all_data);

    Ok(all_data)
}

/// Removes branches in a path (represented by `(usize, usize)` coordinates) that
/// were invalidated by backtracking.
///
/// # Example
///
/// ```
/// // Suppose we have a path where (2,0) gets revisited, causing everything
/// // after the *first* (2,0) to be truncated before continuing.
/// let input = vec![
///     (0,0),
///     (1,0),
///     (2,0),
///     (3,0),
///     (2,0), // backtrack to the point at index 2
///     (2,1),
///     (2,2),
///     (2,0), // backtrack again to the point at index 2 (in the truncated path)
///     (2,3)
/// ];
///
/// let output = remove_backtracks(&input);
/// assert_eq!(output, vec![
///     (0,0),
///     (1,0),
///     (2,0),
///     (2,3)  // all invalidated branches between the old (2,0) and this new (2,0) got removed
/// ]);
/// ```
pub fn remove_backtracks(path: &[(usize, usize)]) -> Vec<(usize, usize)> {
    let mut result = Vec::new();

    for &p in path {
        // Check if `p` already appears in `result`; if so, truncate everything
        // after its *first* occurrence. This effectively discards the old path
        // from that point forward, replacing it with the “new” path.
        if let Some(i) = result.iter().position(|&q| q == p) {
            result.truncate(i);
        }
        // Now push the current point onto our canonical path
        result.push(p);
    }

    result
}

fn main() -> Result<()> {
    println!("Device: {:?}", Device::cuda_if_available());
    let context = context::Context::parse();

    let left_sentences = sentences_from_file(&context.left)?;
    let right_sentences = sentences_from_file(&context.right)?;

    let ctx = AlignContext::new();

    let mut iteration = 0;

    // for iteration
    let mut left_start: usize = 0;
    let mut right_start: usize = 0;

    // 4090 seem to fit 1000-1500, but dijkstra is too slow above 500
    //let score_batch = 300 as usize;
    let score_batch = context.window_size as usize;

    loop {
        println!("iteration: {}...", iteration);
        let flexible_start = iteration == 0;

        let path_file_name = format!(
            "{}/path-{}-{}-{}.json",
            context.context, iteration, left_start, right_start
        );
        let path = if std::fs::metadata(path_file_name.clone()).is_ok() {
            println!("=> skipped");
            let path: Vec<(usize, usize)> =
                serde_json::from_str(&std::fs::read_to_string(path_file_name).unwrap()).unwrap();
            path
        } else {
            let left_xs: Vec<&str> = left_sentences
                .iter()
                .skip(left_start as usize)
                .take(score_batch.into())
                .map(|s| s.as_str())
                .collect();
            let right_xs: Vec<&str> = right_sentences
                .iter()
                .skip(right_start as usize)
                .take(score_batch.into())
                .map(|s| s.as_str())
                .collect();

            let (path, similarity_matrix) =
                alignment_path(&left_xs, &right_xs, flexible_start, &ctx);

            std::fs::write(
                format!(
                    "{}/matrix-{}-{}-{}.json",
                    context.context, iteration, left_start, right_start
                ),
                serde_json::to_string(&similarity_matrix).unwrap(),
            )
            .unwrap();

            std::fs::write(path_file_name, serde_json::to_string(&path).unwrap()).unwrap();
            println!("=> found path of {} steps", path.len());
            path
        };

        // reached any border is exit condition
        if let Some(last) = path.last() {
            if left_start + last.0 + 1 == left_sentences.len()
                || right_start + last.1 + 1 == right_sentences.len()
            {
                break;
            }
        } else {
            panic!("ain't");
        }

        let mid = path.get(path.len() / 2).unwrap();

        left_start += mid.0;
        right_start += mid.1;

        iteration += 1;
    }

    produce_html_from_paths(&context)?;

    Ok(())
}
