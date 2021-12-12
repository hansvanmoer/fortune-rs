/*
 * This file is part of pop.
 *
 * Pop is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * Pop is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with pop.  If not, see <https://www.gnu.org/licenses/>.
*/

use crate::matrix::Matrix;
use crate::row_matrix_view::RowMatrixView;

use std::ops::{Add, Div, Mul, Neg, Sub};

/**
 * Utility functions and structs for linear aritmetic
 */

/**
 * Attempts to triangulate the matrix
 * Returns the order of the matrix (i.e. the number of triangulated rows)
 */

fn triangulate_upper<'a, M: Matrix>(matrix: &mut RowMatrixView<'a, M>) -> usize where
    M: Matrix,
    M::Value: Add<Output = M::Value> + Mul<Output = M::Value> + Neg<Output = M::Value> + PartialEq + PartialOrd + Sub<Output = M::Value>{

    let cols = matrix.cols();
    if cols == 0 {
	// trivial:  no columns
	0
    } else {
	let mut start = 0;
	for col in 0..matrix.cols() - 1 {
	    if find_pivot(matrix, start, col) {
		eliminate_col(matrix, start, col);
		start = start + 1;
	    }
	}
	start
    }
}

/**
 * Finds a pivot and permutates the rows to place it at the required index
 *  True on success, false otherwise
 */

fn find_pivot<'a, M: Matrix>(matrix: &mut RowMatrixView<'a, M>, start: usize, col: usize) -> bool where
    M: Matrix,
    M::Value: Default + Neg<Output = M::Value> + PartialEq + PartialOrd {
    
    let rows = matrix.rows();
    if start >= rows {
	// no more rows => no pivot
	false
    } else {
	let mut index = start;
        let mut value = abs_pivot_value(matrix.get(start, col));
        for row in start + 1..rows {
	    let row_value = abs_pivot_value(matrix.get(row, col));
	    if row_value > value {
                index = row;
                value = row_value;
	    }
        }
        if value == M::Value::default() {
	    false
        } else {
	    if index != start {
                matrix.swap_rows(index, start);
	    }
	    true
        }
    }
}

fn abs_pivot_value<T>(value: T) -> T where T: Default + Neg<Output = T> + PartialOrd {
    if value < T::default() {
	- value
    } else {
	value
    }
}

/**
 * Partial Gaussian elminination for a column
 */

fn eliminate_col<M>(matrix: &mut M, pivot_row: usize, pivot_col: usize) where
    M: Matrix,
    M::Value: Add<Output = M::Value> + Default + Mul<Output = M::Value> + PartialEq + Sub<Output = M::Value> {
    
    let pivot_value = matrix.get(pivot_row, pivot_col);
    for row in pivot_row + 1..matrix.rows() {
        let row_value = matrix.get(row, pivot_col);
        if row_value != M::Value::default() {
            matrix.set(row, pivot_col, M::Value::default());
            for col in pivot_col + 1..matrix.cols() {
                let value = matrix.get(pivot_row, col) * row_value
		    - matrix.get(row, col) * pivot_value;
                matrix.set(row, col, value);
            }
        }
    }
}


/**
 * Solves the system from a row matrix
 * Equations must be given in canonical form:
 * 2x + 3y - 3 = 0 -> [2 3 -3]
 */
fn solve_from_row_matrix_view<'a, M>(matrix: &mut RowMatrixView<'a, M>) -> Result<Vec<M::Value>, SolveError> where
    M: Matrix,
    M::Value: Add<Output = M::Value> + Default + Div<Output = M::Value> + Mul<Output = M::Value> + Neg<Output = M::Value> + PartialEq + PartialOrd + Sub<Output = M::Value> {
    
    let rows = matrix.rows();
    let cols = matrix.cols();
    if cols == 0 {
	//trivial
	Ok(Vec::new())
    } else if rows + 1 < cols {
	// can never have a unique solution
	Err(SolveError::NoUniqueSolution)
    } else {
	// create the upper triangulated matrix
	let order = triangulate_upper(matrix);
	if order == cols - 1 {
            // substitution from the bottom up
            let mut solution = vec![M::Value::default(); order];
            for row in (0..order).rev() {
		let mut value = -matrix.get(row, order);
		for col in row + 1..order {
                    value = value - matrix.get(row, col) * solution[col];
		}
		solution[row] = value / matrix.get(row, row);
            }
            Ok(solution)
	} else {
	    // could not triangulate matrix => no unique solution
            Err(SolveError::NoUniqueSolution)
	}
    }
}

/**
 * Enumerates errors that can occur when solving a system of linear equations
 */

#[derive(Debug, PartialEq)]
pub enum SolveError {
    /**
     * There is no unique solution
     */
    NoUniqueSolution,
}

/**
 * Solves the system from a mutable matrix
 * Equations must be given in canonical form:
 * 2x + 3y - 3 = 0 -> [2 3 -3]
 */
pub fn solve_from_matrix<M>(matrix: &mut M) -> Result<Vec<M::Value>, SolveError>  where
    M: Matrix,
    M::Value: Add<Output = M::Value> + Default + Div<Output = M::Value> + Mul<Output = M::Value> + Neg<Output = M::Value> + PartialOrd + PartialEq + Sub<Output = M::Value> {
    
    solve_from_row_matrix_view(&mut RowMatrixView::from_matrix(matrix))
}

#[cfg(test)]
mod test {

    use crate::matrix::{VariableMatrix, MatrixVarF64};
    use super::*;
    
    #[test]
    fn test_solve_zero_size_system() {
        let mut buf = MatrixVarF64::new(0, 0);
        assert_eq!(solve_from_matrix(&mut buf), Ok(Vec::new()));
    }

     #[test]
    fn test_solve_too_few_rows() {
        let mut buf = MatrixVarF64::new(2, 4);
        assert_eq!(solve_from_matrix(&mut buf), Err(SolveError::NoUniqueSolution));
    }
    
    #[test]
    fn test_solve_system(){
	let mut sys = MatrixVarF64::new(3, 4);
	// x = 2
	// y = 3
	// z = 5
	// 2x - 3y + z + 0 = 0
	// -x + 6y + 2z - 26 = 0
	// -x - y - z + 10 = 0
	//
	sys.set(0, 0, 2.0);
	sys.set(0, 1, -3.0);
	sys.set(0, 2, 1.0); 
	sys.set(0, 3, 0.0);
	sys.set(1, 0, -1.0);
	sys.set(1, 1, 6.0);
	sys.set(1, 2, 2.0);
	sys.set(1, 3, -26.0);
	sys.set(2, 0, -1.0);
	sys.set(2, 1, -1.0);
	sys.set(2, 2, -1.0);
	sys.set(2, 3, 10.0);

	assert_eq!(solve_from_matrix(&mut sys), Ok(vec![2.0, 3.0, 5.0]));
    }

    
    #[test]
    fn test_solve_permutated_system(){
	let mut sys = MatrixVarF64::new(3, 4);
	// x = 2
	// y = 3
	// z = 5
	// -x - y - z + 10 = 0
	// -x + 6y + 2z - 26 = 0
	// 2x - 3y + z + 0 = 0
	//
	sys.set(0, 0, -1.0);
	sys.set(0, 1, -1.0);
	sys.set(0, 2, -1.0);
	sys.set(0, 3, 10.0);
	sys.set(1, 0, -1.0);
	sys.set(1, 1, 6.0);
	sys.set(1, 2, 2.0);
	sys.set(1, 3, -26.0);
	sys.set(2, 0, 2.0);
	sys.set(2, 1, -3.0);
	sys.set(2, 2, 1.0); 
	sys.set(2, 3, 0.0);

	assert_eq!(solve_from_matrix(&mut sys), Ok(vec![2.0, 3.0, 5.0]));
    }

    #[test]
    fn test_solve_no_unique_solution(){
	let mut sys = MatrixVarF64::new(3, 4);
	// x = 2
	// y = 3
	// z = 5
	// -x - y - z + 10 = 0
	// -x + 6y + 2z - 26 = 0
	// -3x + 4y + 0z - 6 = 0
	//
	sys.set(0, 0, -1.0);
	sys.set(0, 1, -1.0);
	sys.set(0, 2, -1.0);
	sys.set(0, 3, 10.0);
	sys.set(1, 0, -1.0);
	sys.set(1, 1, 6.0);
	sys.set(1, 2, 2.0);
	sys.set(1, 3, -26.0);
	sys.set(2, 0, -3.0);
	sys.set(2, 1, 4.0);
	sys.set(2, 2, 0.0); 
	sys.set(2, 3, -6.0);

	assert_eq!(solve_from_matrix(&mut sys), Err(SolveError::NoUniqueSolution));
    }
}
