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

/**
 * A view on a matrix that allows for fast row permutations
 */
pub struct RowMatrixView<'a, M: Matrix> {
    /**
     * The source matrix
     */
    src: &'a mut M,

    /**
     * The row permutation table
     */
    rows: Vec<usize>,
}

impl<'a, M: Matrix> RowMatrixView<'a, M> {

    /**
     * Creates a new row matrix based on the supplied reference
     */
    pub fn from_matrix(src: &'a mut M) -> RowMatrixView<'a, M> {
        let rows = (0..src.rows()).into_iter().collect();
        RowMatrixView { src, rows }
    }

    /**
     * Swaps two rows
     */
    pub fn swap_rows(&mut self, first: usize, second: usize) {
        self.rows.swap(first, second);
    }
    
}

impl<'a, M: Matrix> Matrix for RowMatrixView<'a, M> {

    type Value = M::Value;
    
    fn rows(&self) -> usize {
        self.src.rows()
    }

    fn cols(&self) -> usize {
        self.src.cols()
    }
    
    fn get(&self, row: usize, col: usize) -> Self::Value {
        self.src.get(self.rows[row], col)
    }
    
    fn set(&mut self, row: usize, col: usize, value: Self::Value) {
        self.src.set(self.rows[row], col, value);
    }
}

#[cfg(test)]
mod test {

    use crate::matrix::{MatrixVarF64, VariableMatrix};
    use super::*;
    
    #[test]
    fn test_row_matrix_view_from_matrix() {
        let mut buf = MatrixVarF64::new(2, 3);
        buf.set(0, 0, 1.0);

        let rows = RowMatrixView::from_matrix(&mut buf);
        assert_eq!(rows.rows(), 2);
	assert_eq!(rows.cols(), 3);
        assert_eq!(rows.get(0, 0), 1.0);
        assert_eq!(rows.get(0, 1), 0.0);
        assert_eq!(rows.get(0, 2), 0.0);
        assert_eq!(rows.get(1, 0), 0.0);
        assert_eq!(rows.get(0, 1), 0.0);
        assert_eq!(rows.get(0, 2), 0.0);
    }

    #[test]
    fn test_row_matrix_view_swap() {
        let mut buf = MatrixVarF64::new(2, 3);
        buf.set(0, 0, 1.0);

        let mut rows = RowMatrixView::from_matrix(&mut buf);
        rows.swap_rows(0, 1);
        assert_eq!(rows.rows(), 2);
	assert_eq!(rows.cols(), 3);
        assert_eq!(rows.get(0, 0), 0.0);
        assert_eq!(rows.get(0, 1), 0.0);
        assert_eq!(rows.get(0, 2), 0.0);
        assert_eq!(rows.get(1, 0), 1.0);
        assert_eq!(rows.get(0, 1), 0.0);
        assert_eq!(rows.get(0, 2), 0.0);
    }
    
}
