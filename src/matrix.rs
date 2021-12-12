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

use std::ops::{Add, Div, Mul, Sub};

/**
 * Matrix traits structs and associated functions
 */

/**
 * Errors that can occur while performing operations on a matrix
 */
#[derive(Debug, PartialEq)]
pub enum OperationError {
    
    /**
     * The supplied matrices do not have the correct dimensions
     */
    BadDim
}

/**
 * A generic matrix type
 */
pub trait Matrix : Sized {

    type Value: Copy + Default;
    
    /**
     * The number of rows in the matrix
     */
    fn rows(&self) -> usize;

    /**
     * The number of columns in the matrix
     */
    fn cols(&self) -> usize;
    
    /**
     * Gets a value from the matrix
     */
    fn get(&self, row: usize, col: usize) -> Self::Value;

    /**
     * Sets a value on the matrix
     */
    fn set(&mut self, row: usize, col: usize, value: Self::Value);

    /**
     * Assign values to the matrix
     */
    fn assign_vec(&mut self, vec: Vec<Self::Value>) -> Result<(), OperationError>{
	if vec.len() == self.rows() * self.cols() {
	    for i in 0..vec.len() {
		let row = i / self.cols();
		let col = i % self.cols();
		self.set(row, col, vec[i]);
	    }
	    Ok(())
	} else {
	    Err(OperationError::BadDim)
	}
    }

    /**
     * Assigns values from a matrix
     */
    fn assign_matrix<M: Matrix<Value = Self::Value>>(&mut self, m: &M) -> Result<(), OperationError> {
	if self.rows() == m.rows() && self.cols() == m.cols() {
	    for row in 0..self.rows() {
		for col in 0..self.cols() {
		    self.set(row, col, m.get(row, col));
		}
	    }
	    Ok(())
	} else {
	    Err(OperationError::BadDim)
	}
    }
    
    /**
     * Adds another matrix to this one
     */
    fn assign_add<M: Matrix<Value = Self::Value>>(&mut self, other: &M) -> Result<(), OperationError> where Self::Value: Add<Output = Self::Value> {
	if other.rows() == self.rows() && other.cols() == self.cols() {
	    for row in 0..self.rows() {
		for col in 0..self.cols() {
		    self.set(row, col, self.get(row, col) + other.get(row, col));
		}
	    }
	    Ok(())
	} else {
	    Err(OperationError::BadDim)
	}
    }
    
    /**
     * Subtracts another matrix from this one
     */
    fn assign_sub<M: Matrix<Value = Self::Value>>(&mut self, other: &M) -> Result<(), OperationError> where Self::Value: Sub<Output = Self::Value> {
	if other.rows() == self.rows() && other.cols() == self.cols() {
	    for row in 0..self.rows() {
		for col in 0..self.cols() {
		    self.set(row, col, self.get(row, col) - other.get(row, col));
		}
	    }
	    Ok(())
	} else {
	    Err(OperationError::BadDim)
	}
    }

    /**
     * Multiples the current matrix with a scalar
     */
    fn assign_mul_scalar(&mut self, other: Self::Value) where Self::Value: Mul<Output = Self::Value> {
	for row in 0..self.rows() {
	    for col in 0..self.cols() {
		self.set(row, col, self.get(row, col) * other);
	    }
	}
    }

    /**
     * Multiples the current matrix with a scalar
     */
    fn assign_div_scalar(&mut self, other: Self::Value) where Self::Value: Div<Output = Self::Value> {
	for row in 0..self.rows() {
	    for col in 0..self.cols() {
		self.set(row, col, self.get(row, col) / other);
	    }
	}
    }
    
    /**
     * Assigns the result of a multiplication of two matrices to this matrix
     **/
    fn assign_mul_matrices<M: Matrix<Value = Self::Value>, N: Matrix<Value = Self::Value>>(&mut self, first: &M, second: &N) -> Result<(), OperationError> where Self::Value: Add<Output = Self::Value> + Mul<Output = Self::Value> {
	if first.cols() == second.rows() && self.rows() == first.rows() && self.cols() == second.cols(){
	    for row in 0..self.rows() {
		for col in 0..self.cols() {
		    let mut value = Self::Value::default();
		    for n in 0..first.cols() {
			value = value + first.get(row, n) * second.get(n, col);
		    }
		    self.set(row, col, value);
		}
	    }
	    Ok(())
	} else {
	    Err(OperationError::BadDim)
	}
    }

    /**
     * Calculates the determinant of this matrix if it is square
     */
    fn det(&self) -> Result<Self::Value, OperationError> where
	Self::Value: Default + Mul<Output = Self::Value> + Sub<Output = Self::Value> {
	if self.rows() == self.cols() {
	    match self.rows() {
		0 => Err(OperationError::BadDim),
		1 => Ok(self.get(0,0)),
		2 => Ok(self.get(0,0) * self.get(1, 1) - self.get(0, 1) * self.get(1, 0)),
		_ => {
		    let det = Self::Value::default();
		    Ok(det)
		}
	    }
	} else {
	    Err(OperationError::BadDim)
	}
    }
    
}

/**
 * A variable sized matrix
 */
pub trait VariableMatrix : Matrix {

    fn new(rows: usize, cols: usize) -> Self;

    /**
     * Creates a new matrix from a list of values
     */
    fn from_vec(rows: usize, cols: usize, vec: Vec<Self::Value>) -> Result<Self, OperationError> {
	let mut result = Self::new(rows, cols);
	result.assign_vec(vec)?;
	Ok(result)
    }

    /**
     * Creates a new matrix from a matrix
     */
    fn from_matrix<M: Matrix<Value = Self::Value>>(matrix: &M) -> Self {
	let mut result = Self::new(matrix.rows(), matrix.cols());
	result.assign_matrix(matrix).unwrap();
	result
    }
    
}

/**
 * A fixed sized matrix
 */
pub trait FixedMatrix : Matrix {

    /**
     * Create a new fixed size matrix
     */
    fn new() -> Self;
    
}

macro_rules! define_variable_matrix_type {
    ($type_name:ident, $value_type_name:ident) => {
	
	#[derive(Debug, PartialEq)]
	pub struct $type_name {
	    
	    /**
	     * The number of rows
	     */
	    rows: usize,

	    /**
	     * The number of columns
	     */
	    cols: usize,
	    
	    /**
	     * The values as a row matrix
	     */
	    values: Vec<$value_type_name>,
	}

	impl VariableMatrix for $type_name {

	    /**
	     * Creates a new matrix
	     */
	    fn new(rows: usize, cols: usize) -> Self {
		$type_name {
		    rows,
		    cols,
		    values: vec![Self::Value::default(); rows * cols],
		}
	    }

	}
	
	impl Matrix for $type_name {

	    type Value = $value_type_name;
	    
	    fn rows(&self) -> usize {
		self.rows
	    }

	    fn cols(&self) -> usize {
		self.cols
	    }

	    fn get(&self, row: usize, col: usize) -> Self::Value {
		self.values[row * self.cols + col]
	    }

	    fn set(&mut self, row: usize, col: usize, value: Self::Value) {
		self.values[row * self.cols + col] = value;
	    }

	}

    }
}

macro_rules! define_fixed_matrix_type_2x3 {

    ($type_name:ident, $value_type_name:ident) => {

	/**
	 * A simple 2x3 matrix for $value_type_name
	 */
	#[derive(Clone, Copy, Debug, PartialEq)]
	pub struct $type_name {

	    /**
	     * The matrix's values
	     */
	    values: [$value_type_name; 6],
	}

	impl $type_name {

	    /**
	     * Creates a matrix from the supplied values
	     */
	    pub fn from_values(m00: $value_type_name, m01:  $value_type_name, m02:  $value_type_name,
			       m10:  $value_type_name, m11:  $value_type_name, m12:  $value_type_name) -> Self {
		$type_name {
		    values: [m00, m01, m02, m10, m11, m12],
		}
	    }

	    /**
	     * Creates a matrix from the supplied array
	     */
	    pub fn from_array(values: [$value_type_name; 6]) -> Self {
		$type_name {
		    values: values,
		}
	    }

	    /**
	     * Sets the values from a supplied array
	     */
	    pub fn set_from_array(&mut self, values: &[$value_type_name; 6]) {
		for i in 0..6 {
		    self.values[i] = values[i];
		}
	    }
	    
	}

	impl FixedMatrix for $type_name {
	    
	    fn new() -> Self {
		$type_name {
		    values: [Self::Value::default(); 6],
		}
	    }
	    
	}

	impl Matrix for $type_name {

	    type Value = $value_type_name;
	    
	    fn rows(&self) -> usize {
		2
	    }

	    fn cols(&self) -> usize {
		3
	    }
	    
	    fn get(&self, row: usize, col: usize) -> Self::Value {
		self.values[row * 3 + col]
	    }

	    fn set(&mut self, row: usize, col: usize, value: Self::Value) {
		self.values[row * 3 + col] = value;
	    }
	    
	}

    }
}

macro_rules! define_fixed_matrix_type_3x3 {

    ($type_name:ident, $value_type_name:ident) => {

	/**
	 * A simple 3x3 matrix for $value_type_name
	 */
	#[derive(Clone, Copy, Debug, PartialEq)]
	pub struct $type_name {

	    /**
	     * The matrix's values
	     */
	    values: [$value_type_name; 9],
	}
	
	impl $type_name {
	    
	    /**
	     * Creates a matrix from the supplied values
	     */
	    pub fn from_values(m00: $value_type_name, m01: $value_type_name, m02: $value_type_name,
			       m10: $value_type_name, m11: $value_type_name, m12: $value_type_name,
			       m20: $value_type_name, m21: $value_type_name, m22: $value_type_name) -> Self {
		$type_name {
		    values: [m00, m01, m02, m10, m11, m12, m20, m21, m22],
		}
	    }

	    /**
	     * Creates a matrix from the supplied array
	     */
	    pub fn from_array(values: [$value_type_name; 9]) -> Self {
		$type_name {
		    values: values,
		}
	    }

	    /**
	     * Sets the values from a supplied array
	     */
	    pub fn set_from_array(&mut self, values: &[$value_type_name; 9]) {
		for i in 0..9 {
		    self.values[i] = values[i];
		}
	    }
	    
	}
	
	impl FixedMatrix for $type_name {
	    
	    fn new() -> Self {
		$type_name {
		    values: [Self::Value::default(); 9],
		}
	    }
	    
	}

	impl Matrix for $type_name {

	    type Value = $value_type_name;
	    
	    fn rows(&self) -> usize {
		3
	    }

	    fn cols(&self) -> usize {
		3
	    }
	    
	    fn get(&self, row: usize, col: usize) -> Self::Value {
		self.values[row * 3 + col]
	    }

	    fn set(&mut self, row: usize, col: usize, value: Self::Value) {
		self.values[row * 3 + col] = value;
	    }
	    
	}
    }
}

define_variable_matrix_type!(MatrixVarF64, f64);
define_fixed_matrix_type_2x3!(Matrix2x3F64, f64);
define_fixed_matrix_type_3x3!(Matrix3x3F64, f64);

define_fixed_matrix_type_3x3!(Matrix3x3F32, f32);

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn test_new_variable_matrix() {
        let buf = MatrixVarF64::new(2, 3);
        assert_eq!(buf.rows(), 2);
	assert_eq!(buf.cols(), 3);
        assert_eq!(buf.get(0, 0), 0.0);
        assert_eq!(buf.get(0, 1), 0.0);
        assert_eq!(buf.get(0, 2), 0.0);
        assert_eq!(buf.get(1, 0), 0.0);
        assert_eq!(buf.get(0, 1), 0.0);
        assert_eq!(buf.get(0, 2), 0.0);
    }

    #[test]
    fn test_variable_matrix_from_vec() {
	let buf = MatrixVarF64::from_vec(2, 3, vec![0.0, 1.0, 2.0, 3.0, 4.0, 5.0]).unwrap();
	assert_eq!(buf.rows(), 2);
	assert_eq!(buf.cols(), 3);
        assert_eq!(buf.get(0, 0), 0.0);
        assert_eq!(buf.get(0, 1), 1.0);
        assert_eq!(buf.get(0, 2), 2.0);
        assert_eq!(buf.get(1, 0), 3.0);
        assert_eq!(buf.get(1, 1), 4.0);
        assert_eq!(buf.get(1, 2), 5.0);
    }

    #[test]
    fn test_variable_matrix_from_vec_wrong_size() {
	assert_eq!(MatrixVarF64::from_vec(2, 3, vec![1.0]).unwrap_err(), OperationError::BadDim);
    }
    
    #[test]
    fn test_variable_matrix_from_matrix() {
        let mut buf = MatrixVarF64::new(2, 3);
        buf.set(0, 0, 1.0);

        let buf2 = MatrixVarF64::from_matrix(&buf);
        assert_eq!(buf2.rows(), 2);
	assert_eq!(buf2.cols(), 3);
        assert_eq!(buf2.get(0, 0), 1.0);
        assert_eq!(buf2.get(0, 1), 0.0);
        assert_eq!(buf2.get(0, 2), 0.0);
        assert_eq!(buf2.get(1, 0), 0.0);
        assert_eq!(buf2.get(0, 1), 0.0);
        assert_eq!(buf2.get(0, 2), 0.0);
    }

}
