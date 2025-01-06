use crate::*;

pub struct AlignContext {
    pub similarity: Similarity,
}

// default+-
impl AlignContext {
    pub fn new() -> Self {
        Self {
            similarity: Similarity::new(),
        }
    }
}

// Given two string vectors,
// returns generally diagonal path in matrix (as X bs) of semantic similarity,
// that has in it most semantic alignment captured.
// The path can connect any two sides of matrix so it is like a local alignment,
// as opposed to global, that would connect two opposing corners.
//
// Simplistic alignment implementation should work in general case (Dijkstra search).
//
///
/// Simplest square diagonal case, each item gets aligned to the corresponding item.
/// ```
/// use translate::*;
/// let ctx = AlignContext::new();
///
/// let xs = vec!["content text", "second piece", "third part"];
/// let ys = vec!["content water", "second thing", "third chunk"];
/// let (got, _) = alignment_path(&xs, &ys, true, &ctx);
/// assert_eq!(got, vec![
///   (0, 0),
///   (1, 1),
///   (2, 2),
/// ]);
/// ```
///
/// Non-square case
/// ```
/// use translate::*;
/// let ctx = AlignContext::new();
///
/// let xs = vec!["trash", "garbage", "content text", "second piece", "third part", "ignorable"];
/// let ys = vec!["content water", "second thing", "third chunk"];
/// let (got, _) = alignment_path(&xs, &ys, true, &ctx);
/// assert_eq!(got, vec![
///   (2, 0),
///   (3, 1),
///   (4, 2),
/// ]);
/// ```
///
pub fn alignment_path(
    xs: &Vec<&str>,
    ys: &Vec<&str>,
    flexible_start: bool,
    ctx: &AlignContext,
) -> (Vec<(usize, usize)>, Vec<Vec<f32>>) {
    // first joining xs and ys into a vector
    // then computing similarity matrix: vector X vector
    // then reconstucting matrix into: xs X ys
    let mut joined: Vec<&str> = vec![];
    joined.extend(xs);
    joined.extend(ys);
    let joined_similarity_matrix = ctx.similarity.get_many(&joined[..]);
    let xs_to_ys = reconstruct(joined_similarity_matrix, xs.len());

    // similarity(-1..1) -> cost (0..1)
    let cost_matrix: Vec<Vec<f32>> = xs_to_ys
        .iter()
        .map(|row| row.into_iter().map(|value| (1. - value) / 2.).collect())
        .collect();

    // makes it more consistent for cases with noise regularity, and weak signal that tends to be
    // ignored in favor of direct diagonal movement instead
    // (not that I actually inspected what that does)
    //
    let cost_matrix = row_col_blended_normalization(&cost_matrix);

    let path = find_path(cost_matrix, flexible_start);

    (path, xs_to_ys)
}

/// Normalize each row of `matrix` to [0,1].
/// Returns a new matrix with the same dimensions.
pub fn normalize_rows(matrix: &Vec<Vec<f32>>) -> Vec<Vec<f32>> {
    let mut output = Vec::with_capacity(matrix.len());

    for row in matrix.iter() {
        // Find min and max in this row
        if let (Some(&row_min), Some(&row_max)) = (
            row.iter().min_by(|a, b| a.partial_cmp(b).unwrap()),
            row.iter().max_by(|a, b| a.partial_cmp(b).unwrap()),
        ) {
            let range = row_max - row_min;
            if range > 0.0 {
                let normalized_row: Vec<f32> =
                    row.iter().map(|&val| (val - row_min) / range).collect();
                output.push(normalized_row);
            } else {
                // If range == 0, all values are the same; just push zeros (or all 0.5, your choice)
                output.push(vec![0.0; row.len()]);
            }
        } else {
            // If row is empty (unusual), just push an empty vec
            output.push(Vec::new());
        }
    }

    output
}

/// Normalize each column of `matrix` to [0,1].
/// Returns a new matrix with the same dimensions.
pub fn normalize_columns(matrix: &Vec<Vec<f32>>) -> Vec<Vec<f32>> {
    if matrix.is_empty() || matrix[0].is_empty() {
        return matrix.clone(); // or return an empty matrix, as appropriate
    }

    let rows = matrix.len();
    let cols = matrix[0].len();

    // We will first find the min and max of each column.
    // Store them in vectors so we can do a quick lookup later.
    let mut col_mins = vec![f32::MAX; cols];
    let mut col_maxs = vec![f32::MIN; cols];

    // Determine column-wise min and max
    for row in 0..rows {
        for col in 0..cols {
            let val = matrix[row][col];
            if val < col_mins[col] {
                col_mins[col] = val;
            }
            if val > col_maxs[col] {
                col_maxs[col] = val;
            }
        }
    }

    // Now produce the normalized matrix
    let mut output = Vec::with_capacity(rows);
    for r in 0..rows {
        let mut new_row = Vec::with_capacity(cols);
        for c in 0..cols {
            let range = col_maxs[c] - col_mins[c];
            if range > 0.0 {
                let val = matrix[r][c];
                let norm_val = (val - col_mins[c]) / range;
                new_row.push(norm_val);
            } else {
                // If range == 0 for this column, all values in the column are the same
                new_row.push(0.0);
            }
        }
        output.push(new_row);
    }

    output
}

/// Compute the element-wise average of two matrices of the same size.
pub fn average_matrices(a: &Vec<Vec<f32>>, b: &Vec<Vec<f32>>) -> Vec<Vec<f32>> {
    let rows = a.len();
    let mut output = Vec::with_capacity(rows);

    for i in 0..rows {
        let cols = a[i].len();
        let mut new_row = Vec::with_capacity(cols);
        for j in 0..cols {
            // simple 0.5 * (a + b)
            let val = 0.5 * (a[i][j] + b[i][j]);
            new_row.push(val);
        }
        output.push(new_row);
    }

    output
}

/// Blend row-wise and column-wise min–max normalization by averaging them.
///
/// Returns a new `Vec<Vec<f32>>` whose (i,j) entry is the average of the
/// row-normalized (i,j) and the column-normalized (i,j).
pub fn row_col_blended_normalization(matrix: &Vec<Vec<f32>>) -> Vec<Vec<f32>> {
    let row_norm = normalize_rows(matrix);
    let col_norm = normalize_columns(matrix);
    average_matrices(&row_norm, &col_norm)
}

// changes similarity
// reconstructs matrix from being mirrorred by diagonal into more usable form
// and scales costs so shorter diagonals do not have advantage to the middle longest one
//
// costs are (best)0..1(worst)
// also converts into floats with orderingjj

// takes square matrix with mirroring over diagonal and diagonal of 1's
// makes usable shape, discards diagonal and half of other items
//
fn reconstruct(joined_matrix: Vec<Vec<f32>>, xs_len: usize) -> Vec<Vec<f32>> {
    let ys_len = joined_matrix.len() - xs_len;

    (0..ys_len)
        .map(|y| {
            (0..xs_len)
                .map(|x| {
                    let joined_y = x;
                    let joined_x = xs_len + y;
                    joined_matrix[joined_y][joined_x]
                })
                .collect::<Vec<f32>>()
        })
        .collect()
}
