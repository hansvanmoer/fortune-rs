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
 * A view that removes one column and one row of its source matrix
 */

#[derive(Debug)]
pub struct MinorMatrixView<'a, M: Matrix>{
    /**
     * The original matrix
     */
    src: &'a mut M,

    /**
     * The minor row
     */
    row: usize,

    /**
     * The minor column
     */
    col: usize,
    
}

impl<'a, M: Matrix> MinorMatrixView<'a, M> {

    /**
     * Creates a new minor matrix view for the given row and column
     */
    pub fn from_matrix(src: &'a mut M, row: usize, col: usize) -> Result<Self, OperationError> {
	if row < src.rows() && col < src.cols() {
	    Ok(MinorMatrixView{
		src,
		row,
		col,
	    })
	} else {
	    Err(OperationError::BadDim)
	}
    }

}

impl<'a, M: Matrix> Matrix for MinorMatrixView<'a, M> {

    type Value = M::Value;

    fn rows(&self) -> usize {
	self.src.rows() - self.row - 1
    }

    fn cols(&self) -> usize {
	self.src.cols() - self.row - 1
    }
    
    fn get(&self, row: usize, col: usize) -> Self::Value {
	let ncol = if col < self.col {
	    col
	} else {
	    col + 1
	};
	self.src.get(self.row + row, ncol)
    }

    fn set(&mut self, row: usize, col: usize, value: Self::Value) {
	let ncol = if col < self.col {
	    col
	} else {
	    col + 1
	};
	self.src.set(self.row + row, ncol, value);
    } 
    
}

#[cfg(test)]
mod test {

    use super::*;

    use crate::matrix::{Matrix, MatrixVarF64, VariableMatrix};

    
}
