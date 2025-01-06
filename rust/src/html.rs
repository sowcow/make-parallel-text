const THREE_COLUMNS: &str = r#"<!DOCTYPE html>
<html>
<head>
  <meta charset="UTF-8">
  <style>
    @page {
      size: A4 landscape;
      margin: 0mm; /* Removes all page margins */
      padding: 0mm;
    }

    body {
      margin: 0; /* Removes default body margins */
      font-family: Arial, sans-serif;
      font-size: 18pt;
    }

    table {
      width: 100%;
      border-collapse: collapse;
    }
    td {
      /*width: 50%;*/
      width: 33%;
      vertical-align: top;
      padding: 5px;
      padding-right: 0px;
      border: none;
    }
    * {
      margin: 0;
      padding: 0;
      box-sizing: border-box;
    }
  </style>
</head>
<body>
"#;

const TWO_COLUMNS: &str = r#"<!DOCTYPE html>
<html>
<head>
  <meta charset="UTF-8">
  <style>
    @page {
      size: A4 landscape;
      margin: 0mm; /* Removes all page margins */
      padding: 0mm;
    }

    body {
      margin: 0; /* Removes default body margins */
      font-family: Arial, sans-serif;
      font-size: 18pt;
    }

    table {
      width: 100%;
      border-collapse: collapse;
    }
    td {
      width: 50%;
      vertical-align: top;
      padding: 5px;
      padding-right: 0px;
      border: none;
    }
    * {
      margin: 0;
      padding: 0;
      box-sizing: border-box;
    }
  </style>
</head>
<body>
"#;

const ONE_COLUMN: &str = r#"<!DOCTYPE html>
<html>
<head>
  <meta charset="UTF-8">
  <style>
    @page {
      size: A4;
      margin: 0mm; /* Removes all page margins */
      padding: 0mm;
    }

    body {
      margin: 0; /* Removes default body margins */
      font-family: Arial, sans-serif;
      font-size: 18pt;
    }

    * {
      margin: 0;
      padding: 0;
      box-sizing: border-box;
    }

    .l {
      text-align: start;
      color: green;
    }

    .r {
      text-align: end;
      font-weight: bold;
    }
  </style>
</head>
<body>
"#;

pub fn produce_html(
    left_sentences: &Vec<String>,
    right_sentences: &Vec<String>,
    path: &Vec<(usize, usize)>,
    columns: usize,
) -> String {
    let alignment = get_sequence(path);
    if columns == 3 {
        build_html_from_sequence(&left_sentences, &right_sentences, &alignment, THREE_COLUMNS)
    } else if columns == 2 {
        build_html_from_sequence(&left_sentences, &right_sentences, &alignment, TWO_COLUMNS)
    } else if columns == 1 {
        build_html_one_column(&left_sentences, &right_sentences, &alignment)
    } else {
        todo!();
    }
}

use std::collections::HashSet;

#[derive(Debug, Clone)]
pub struct LRPair {
    pub l: usize,
    pub r: usize,
}

// this should produce alignments such that
// sequential vertical or horizontal segments of the path are part of single 1:N alignment segment
// while diagonal steps are own alignments 1:1
// and the whole path is divided into such alignments
// (some ruby->rust by chatgpt, but it works)
//
pub fn get_sequence(path: &[(usize, usize)]) -> Vec<Vec<LRPair>> {
    let mut pairs: Vec<Vec<LRPair>> = Vec::new();

    for (l, r) in path {
        let tail_opt = pairs.last_mut();

        match tail_opt {
            None => {
                // If there is no tail yet, create a new "block" with this first LRPair
                pairs.push(vec![LRPair { l: *l, r: *r }]);
            }
            Some(tail) => {
                // Tentatively add the new pair to a copy of the tail
                let mut new_tail = tail.clone();
                new_tail.push(LRPair { l: *l, r: *r });

                // Count how many unique L's and R's in this "tentative" tail
                let unique_ls = new_tail.iter().map(|x| x.l).collect::<HashSet<_>>().len();
                let unique_rs = new_tail.iter().map(|x| x.r).collect::<HashSet<_>>().len();

                // If adding this new pair causes both L's and R's to have more than 1 unique value,
                // then start a new block. Otherwise, continue in the same tail.
                if unique_ls > 1 && unique_rs > 1 {
                    // Create a new block with just this new pair
                    pairs.push(vec![LRPair { l: *l, r: *r }]);
                } else {
                    // Otherwise, append to the existing tail
                    tail.push(LRPair { l: *l, r: *r });
                }
            }
        }
    }

    pairs
}

/// Builds the final HTML
///
/// - `left_sentences`: text that would normally go on the left.
/// - `right_sentences`: text that would normally go on the right.
/// - `sequence`: the output from `get_sequence`.
///
/// This replicates your original Ruby structure
pub fn build_html_from_sequence(
    left_sentences: &[String],
    right_sentences: &[String],
    sequence: &[Vec<LRPair>],
    html_header: &str,
) -> String {
    let mut html_content = html_header.to_owned();

    // For each "block" in the sequence
    for xs in sequence {
        // Collect all L values
        let mut ls: Vec<usize> = xs.iter().map(|lr| lr.l).collect();
        ls.sort_unstable();
        ls.dedup();

        // Collect all R values
        let mut rs: Vec<usize> = xs.iter().map(|lr| lr.r).collect();
        rs.sort_unstable();
        rs.dedup();

        // Build the final strings for each side
        let left: Vec<&str> = ls
            .iter()
            .filter_map(|&index| left_sentences.get(index))
            .map(|s| s.trim())
            .collect();
        let right: Vec<&str> = rs
            .iter()
            .filter_map(|&index| right_sentences.get(index))
            .map(|s| s.trim())
            .collect();

        let right_joined = right.join("<br />");
        let left_joined = left.join("<br />");

        html_content.push_str(&format!(
            r#"
  <hr />
  <table>
    <tr>
      <td>
        <p>{left_part}</p>
      </td>
      <td>
        <p>{right_part}</p>
      </td>
      <td></td>
    </tr>
  </table>
"#,
            right_part = right_joined,
            left_part = left_joined
        ));
    }

    // Close the HTML body and tags
    html_content.push_str(
        r#"
</body>
</html>
"#,
    );

    html_content
}

pub fn build_html_one_column(
    left_sentences: &[String],
    right_sentences: &[String],
    sequence: &[Vec<LRPair>],
) -> String {
    let mut html_content = ONE_COLUMN.to_owned();

    // For each "block" in the sequence
    for xs in sequence {
        // Collect all L values
        let mut ls: Vec<usize> = xs.iter().map(|lr| lr.l).collect();
        ls.sort_unstable();
        ls.dedup();

        // Collect all R values
        let mut rs: Vec<usize> = xs.iter().map(|lr| lr.r).collect();
        rs.sort_unstable();
        rs.dedup();

        // Build the final strings for each side
        let left: Vec<&str> = ls
            .iter()
            .filter_map(|&index| left_sentences.get(index))
            .map(|s| s.trim())
            .collect();
        let right: Vec<&str> = rs
            .iter()
            .filter_map(|&index| right_sentences.get(index))
            .map(|s| s.trim())
            .collect();

        let right_joined = right.join("<br />");
        let left_joined = left.join("<br />");

        html_content.push_str(&format!(
            r#"
    <hr />
    <p class=r>{right_part}</p>
    <p class=l>{left_part}</p>
"#,
            right_part = right_joined,
            left_part = left_joined
        ));
    }

    // Close the HTML body and tags
    html_content.push_str(
        r#"
</body>
</html>
"#,
    );

    html_content
}
