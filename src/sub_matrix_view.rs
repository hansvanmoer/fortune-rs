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

use crate::matrix::{Matrix, OperationError};

/**
 * A view on a matrix as a submatrix
 */
#[derive(Debug)]
pub struct SubMatrixView<'a, M: Matrix> {

    /**
     * The original matrix
     */
    src: &'a mut M,

    /**
     * The first row in the view
     */
    first_row: usize,

    /**
     * The first column in the view
     */
    first_col: usize,

    /**
     * The number of rows in the view
     */
    rows: usize,

    /**
     * The number of columns in the view
     */
    cols: usize,
}

impl<'a, M: Matrix> SubMatrixView<'a, M> {

    /**
     * Creates a new submatrix view
     */
    pub fn from_matrix(src: &'a mut M, first_row: usize, rows: usize, first_col: usize, cols: usize) -> Result<SubMatrixView<'a, M>, OperationError> {
	if first_row + rows > src.rows() || first_col + cols > src.cols() {
	    Err(OperationError::BadDim)
	} else {
	    Ok(SubMatrixView {
		src,
		first_row,
		first_col,
		rows,
		cols,
	    })
	}
    }
    
}

impl<'a, M: Matrix> Matrix for SubMatrixView<'a, M> {

    type Value = M::Value;
    
    fn rows(&self) -> usize {
        self.rows
    }

    fn cols(&self) -> usize {
        self.cols
    }
    
    fn get(&self, row: usize, col: usize) -> Self::Value {
        self.src.get(self.first_row + row, self.first_col + col)
    }

    fn set(&mut self, row: usize, col: usize, value: Self::Value) {
        self.src.set(self.first_row + row, self.first_col + col, value);
    }

}

#[cfg(test)]
mod test {

    use crate::matrix::{MatrixVarF64, OperationError, VariableMatrix};
    use super::*;
    
    #[test]
    fn test_sub_matrix_view_too_few_rows() {
	let mut buf = MatrixVarF64::from_vec(2, 3, vec![0.0, 1.0, 2.0, 3.0, 4.0, 5.0]).unwrap();
	let view = SubMatrixView::from_matrix(&mut buf, 1, 3, 1, 1).unwrap_err();
	assert_eq!(view, OperationError::BadDim);
    }

    #[test]
    fn test_sub_matrix_view_too_few_cols() {
	let mut buf = MatrixVarF64::from_vec(2, 3, vec![0.0, 1.0, 2.0, 3.0, 4.0, 5.0]).unwrap();
	let view = SubMatrixView::from_matrix(&mut buf, 1, 1, 1, 3).unwrap_err();
	assert_eq!(view, OperationError::BadDim);
    }

    #[test]
    fn test_sub_matrix_get() {
	let mut buf = MatrixVarF64::from_vec(3, 4, vec![0.0, 1.0, 2.0, 3.0,
						     4.0, 5.0, 6.0, 7.0,
						     8.0, 9.0, 10.0, 11.0]).unwrap();
	let view = SubMatrixView::from_matrix(&mut buf, 1, 2, 2, 2).unwrap();

	assert_eq!(view.rows(), 2);
	assert_eq!(view.cols(), 2);
        assert_eq!(view.get(0, 0), 6.0);
        assert_eq!(view.get(0, 1), 7.0);
        assert_eq!(view.get(1, 0), 10.0);
        assert_eq!(view.get(1, 1), 11.0);
    }

    #[test]
    fn test_sub_matrix_set() {
	let mut buf = MatrixVarF64::from_vec(3, 4, vec![0.0, 1.0, 2.0, 3.0,
						     4.0, 5.0, 6.0, 7.0,
						     8.0, 9.0, 10.0, 11.0]).unwrap();
	let mut view = SubMatrixView::from_matrix(&mut buf, 1, 2, 2, 2).unwrap();

        view.set(1, 1, 13.0);
        assert_eq!(buf.get(2, 3), 13.0);
    }
    
}
