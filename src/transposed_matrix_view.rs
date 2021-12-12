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
 * A transposed view on a matrix
 */

pub struct TransposedMatrixView<'a, M: Matrix> {

    /**
     * The original matrix
     */
    src: &'a mut M,
}

impl<'a, M: Matrix> TransposedMatrixView<'a, M> {

    pub fn from_matrix(src: &'a mut M) -> Self {
	TransposedMatrixView{
	    src,
	}
    }
    
}

impl<'a, M: Matrix> Matrix for TransposedMatrixView<'a, M> {

    type Value = M::Value;
    
    fn rows(&self) -> usize {
	self.src.cols()
    }

    fn cols(&self) -> usize {
	self.src.rows()
    }

    fn get(&self, row: usize, col: usize) -> Self::Value {
	self.src.get(col, row)
    }

    fn set(&mut self, row: usize, col: usize, value: Self::Value) {
	self.src.set(col, row, value);
    }
    
}

#[cfg(test)]
mod test {

    use super::*;
    use crate::matrix::{FixedMatrix, Matrix, Matrix2x3F64};

    #[test]
    fn transposed_matrix_view(){
	let mut matrix = Matrix2x3F64::from_values(0.0, 1.0, 2.0, 3.0, 4.0, 5.0);
	let mut view = TransposedMatrixView::from_matrix(&mut matrix);

	assert_eq!(3, view.rows());
	assert_eq!(2, view.cols());

	assert_eq!(0.0, view.get(0,0));	
	assert_eq!(1.0, view.get(1,0));
	assert_eq!(2.0, view.get(2,0));
	assert_eq!(3.0, view.get(0,1));
	assert_eq!(4.0, view.get(1,1));
	assert_eq!(5.0, view.get(2,1));

	view.set(2, 1, 15.0);
	assert_eq!(15.0, matrix.get(1, 2));
    }
}
