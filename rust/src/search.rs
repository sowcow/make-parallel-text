use float_next_after::NextAfter;
use ordered_float::OrderedFloat;
use pathfinding::prelude::dijkstra;

use std::rc::Rc;

// could go imlp type instead of specifics for starting node
// not sure about effects of decision so keeping it primitive

// i32 into isize?? or shrink instead?

#[derive(Clone, Eq, Hash, Ord, PartialEq, PartialOrd, Debug)]
struct Pos(i32, i32, Rc<Vec<Vec<OrderedFloat<f32>>>>, OrderedFloat<f32>);
// asking for normal {} struct
// could have .translate + .join for result building vs keeping it in function

/// Finds the least cost mostly diagonal path in the given cost matrix.
///
/// # Arguments
///
/// * `matrix` - A 2D vector of `f32` representing the cost matrix.
///
/// # Returns
///
/// A vector of `(usize, usize)` tuples representing the coordinates of the path with the least cost.
///
/// # Examples
///
/// ## Primitive square case
///
/// ```
/// use translate::*;
///
/// let matrix = vec![
///     vec![0.0, 1.0],
///     vec![1.0, 0.0],
/// ];
/// let path = find_path(matrix, true);
/// assert_eq!(path, vec![(0, 0), (1, 1)]);
/// ```
///
/// ## Next square case
///
/// ```
/// use translate::*;
///
/// let matrix = vec![
///     vec![0.0, 1.0, 1.0],
///     vec![1.0, 0.0, 1.1],
///     vec![1.0, 1.0, 0.0],
/// ];
/// let path = find_path(matrix, true);
/// assert_eq!(path, vec![(0, 0), (1, 1), (2,2)]);
/// ```
///
/// ## Avoids using cheap corner, uses median value so needs minimal size
///
/// ```
/// use translate::*;
///
/// let matrix = vec![
///     vec![1.0, 2.0, 2.0],
///     vec![2.0, 1.0, 2.0],
///     vec![0.0, 2.0, 1.0],
/// ];
/// let path = find_path(matrix, true);
/// assert_eq!(path, vec![(0, 0), (1, 1), (2,2)]);
/// ```
///
/// ## Small matrix case
/// It should prefer diagonal given all equal, but not too strongly to get biased in more realistic
/// matrix size cases.
///
/// ```
/// use translate::*;
///
/// let matrix = vec![
///     vec![0.0, 0.0],
///     vec![0.0, 0.0],
/// ];
/// let path = find_path(matrix, true);
/// assert_eq!(path, vec![(0, 0), (1, 1)]);
/// ```
///
/// ```
/// use translate::*;
///
/// let matrix = vec![
///     vec![0.0, 1.0],
///     vec![0.0, 0.0],
/// ];
/// let path = find_path(matrix, true);
/// assert_eq!(path, vec![(0, 0), (1, 1)]);
/// ```
///
/// Legit most corner case of path
/// ```
/// use translate::*;
///
/// let matrix = vec![
///     vec![0.0, 0.0],
///     vec![1.0, 1.0],
/// ];
/// let path = find_path(matrix, true);
/// assert_eq!(path, vec![(0, 0), (1, 0)]);
/// ```
///
/// Non-square case: odd case, just what it is in current calculation
/// ```
/// use translate::*;
///
/// let matrix = vec![
///     vec![0.0, 0.0, 0.0],
///     vec![1.0, 1.0, 1.0],
/// ];
/// let path = find_path(matrix, true);
/// assert_eq!(path, vec![(1, 0), (2, 0)]);
/// ```
///
/// Can force it to start exactly from the corner.
/// ```
/// use translate::*;
///
/// let matrix = vec![
///     vec![0.0, 0.0, 0.0],
///     vec![1.0, 1.0, 1.0],
/// ];
/// let path = find_path(matrix, false);
/// assert_eq!(path, vec![(0, 0), (1, 0), (2, 0)]);
/// ```
///
/// The bias does not affect clearly shortest path, even if not diagonal.
/// ```
/// use translate::*;
///
/// let matrix = vec![
///     vec![0.0, 1.0],
///     vec![0.0, 1.000000001],
/// ];
/// let path = find_path(matrix, true);
/// assert_eq!(path, vec![(0, 0), (0, 1)]);
/// ```
///
/// ## Non-square matrix: does not favor corners too
///
/// ```
/// use translate::*;
///
/// let matrix = vec![
///     vec![2.0, 2.0, 2.0], // 2.0's make for god noise median value
///     vec![2.0, 2.0, 2.0],
///     vec![2.0, 2.0, 2.0],
///     vec![1.0, 2.0, 2.0],
///     vec![0.0, 1.0, 2.0],
///     vec![0.0, 1.0, 1.0],
/// ];
/// let path = find_path(matrix, true);
/// assert_eq!(path, vec![(0, 3), (1, 4), (2,5)]);
/// ```
///
/// ## Single Row Matrix
///
/// ```
/// use translate::*;
///
/// let matrix = vec![
///     vec![1.0, 0.0, 3.0, 4.0],
/// ];
/// let path = find_path(matrix, true);
/// assert_eq!(path, vec![(1, 0)]);
/// ```
///
/// ## Single Column Matrix
///
/// ```
/// use translate::*;
///
/// let matrix = vec![
///     vec![1.0],
///     vec![2.0],
///     vec![0.0],
///     vec![4.0],
/// ];
/// let path = find_path(matrix, true);
/// assert_eq!(path, vec![(0, 2)]);
/// ```
///
/// ## Empty Matrix
///
/// ```
/// use translate::*;
///
/// let matrix: Vec<Vec<f32>> = vec![];
/// let path = find_path(matrix, true);
/// assert_eq!(path, Vec::<(usize, usize)>::new());
/// ```
///
pub fn find_path(matrix: Vec<Vec<f32>>, flexible_start: bool) -> Vec<(usize, usize)> {
    if matrix.len() == 0 {
        return vec![];
    }
    if matrix[0].len() == 0 {
        return vec![];
    }

    let matrix: Vec<Vec<OrderedFloat<f32>>> = matrix
        .into_iter()
        .map(|row| row.into_iter().map(|value| OrderedFloat(value)).collect())
        .collect();
    let matrix = Rc::new(matrix);
    let x;
    let y;
    if flexible_start {
        x = -1;
        y = -1;
    } else {
        x = 0;
        y = 0;
    }
    let pos = Pos::start(matrix, x, y);

    let result = dijkstra(&pos, |p| p.successors(), |p| p.reached()).unwrap();

    let mut path = result.0;

    if flexible_start {
        path.remove(0); // -1,-1 starting point
    }

    path.iter()
        .map(|pos| (pos.0 as usize, pos.1 as usize))
        .collect()
}

impl Pos {
    fn start(matrix: Rc<Vec<Vec<OrderedFloat<f32>>>>, x: i32, y: i32) -> Self {
        let mut flat: Vec<&OrderedFloat<f32>> = matrix.iter().flatten().collect();
        flat.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let mut median: f32 = **flat[flat.len() / 2];

        // true hairsplitting, 2x2 zeroes case + division by two
        median = median.next_after(f32::INFINITY);
        median = median.next_after(f32::INFINITY);

        let median = OrderedFloat(median);

        // median is no longer median here - old naming
        Pos(x, y, matrix, median)
    }
    fn is_flexible_start(&self) -> bool {
        let &Pos(x, y, _, _) = self;
        x == -1
    }
    fn height(&self) -> usize {
        let Pos(_, _, matrix, _) = &self;
        matrix.len()
    }
    fn width(&self) -> usize {
        let Pos(_, _, matrix, _) = &self;
        matrix[0].len()
    }
    // just reaching single pixel on the last row/column, could cut a bit there but not a problem
    fn reached(&self) -> bool {
        let &Pos(x, y, _, _) = self;
        let Pos(_, _, matrix, _) = &self;
        let w = matrix[0].len() as i32;
        let h = matrix.len() as i32;

        x == w - 1 || y == h - 1
    }

    fn cost(&self) -> OrderedFloat<f32> {
        let &Pos(x, y, _, median) = self;
        let Pos(_, _, matrix, _) = &self;

        if self.is_flexible_start() {
            panic!("nope");
        }

        if self.reached() {
            let added_cells = cells_to_extend_diagonal(x, y, matrix[0].len(), matrix.len());
            let weight: f32 = *median; // noise
            let correction = added_cells as f32 * weight * 0.5; // correction is spread between starts and ends

            let cost = matrix[y as usize][x as usize];
            return cost + correction;
        }

        matrix[y as usize][x as usize]
    }

    fn successors(&self) -> Vec<(Pos, OrderedFloat<f32>)> {
        let &Pos(x, y, _, median) = self;
        let Pos(_, _, matrix, _) = &self;
        let w = matrix[0].len() as i32;
        let h = matrix.len() as i32;

        let xs = if self.is_flexible_start() {
            // options are first row and column
            let mut xs = vec![];
            xs.extend(
                (0..w)
                    .map(|i| Pos(i, 0, matrix.clone(), median))
                    .collect::<Vec<Pos>>(),
            );
            xs.extend(
                (1..h)
                    .map(|j| Pos(0, j, matrix.clone(), median))
                    .collect::<Vec<Pos>>(),
            );

            let result = xs
                .into_iter()
                .map(|p| {
                    let x = p.0;
                    let y = p.1;
                    let added_cells = cells_to_extend_diagonal(x, y, w as usize, h as usize);
                    let weight: f32 = *median; // noise
                    let correction = added_cells as f32 * weight * 0.5; // correction is spread between starts and ends

                    let cost = p.cost() + correction;
                    (p, cost)
                })
                .collect::<Vec<(Pos, OrderedFloat<f32>)>>();
            return result;
        } else {
            vec![
                Pos(x + 1, y + 1, matrix.clone(), median), // diagonal - most probable from good position
                Pos(x + 1, y, matrix.clone(), median), // additional sentence on one text while same on another
                Pos(x, y + 1, matrix.clone(), median),
            ]
        };

        xs.into_iter()
            .map(|x| {
                let cost = x.cost();
                (x, cost)
            })
            .collect::<Vec<(Pos, OrderedFloat<f32>)>>()
    }
}

/// Given point and grid dimensions.
/// Calculate how many cells to add for diagonal through the point to have same number of cells as longest diagonal.
/// This is used to correct search from having bias for short-cutting by using corners.
///
/// Types are arbitrary.
///
/// ```
/// use translate::*;
///
/// // simle square cases
/// assert_eq!(cells_to_extend_diagonal(0, 0, 1, 1), 0);
/// assert_eq!(cells_to_extend_diagonal(2, 2, 3, 3), 0);
/// assert_eq!(cells_to_extend_diagonal(1, 2, 3, 3), 1);
/// assert_eq!(cells_to_extend_diagonal(0, 2, 3, 3), 2);
/// assert_eq!(cells_to_extend_diagonal(2, 0, 3, 3), 2);
///
/// // non-square cases
/// assert_eq!(cells_to_extend_diagonal(0, 0, 1, 3), 0);
/// assert_eq!(cells_to_extend_diagonal(0, 1, 1, 3), 0);
/// assert_eq!(cells_to_extend_diagonal(0, 2, 1, 3), 0);
///
/// assert_eq!(cells_to_extend_diagonal(0, 0, 2, 3), 0);
/// assert_eq!(cells_to_extend_diagonal(1, 1, 2, 3), 0);
///
/// assert_eq!(cells_to_extend_diagonal(1, 0, 2, 3), 1);
/// assert_eq!(cells_to_extend_diagonal(0, 2, 2, 3), 1);
/// ```
pub fn cells_to_extend_diagonal(mut x: i32, mut y: i32, mut w: usize, mut h: usize) -> i32 {
    // having form that is expected
    if w > h {
        std::mem::swap(&mut x, &mut y);
        std::mem::swap(&mut w, &mut h);
    }

    // mirrored point
    let x2 = w as i32 - x;
    let y2 = h as i32 - y;

    let result = if x > y {
        x - y
    } else if x2 > y2 {
        x2 - y2
    } else {
        0 // one of diagonals - nothnig to add
    };

    result
}
