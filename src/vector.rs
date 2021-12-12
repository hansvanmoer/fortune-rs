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

use std::ops::{Add, Div, Mul, Neg, Sub};

/**
 * Vector traits structs and associated functions
 */

/**
 * Errors that can occur while performing operations on a vector
 */
#[derive(Debug, PartialEq)]
pub enum OperationError {
    
    /**
     * The supplied vectors do not have the correct dimensions
     */
    BadDim
}


/** 
 * Represents a vector
 */
pub trait Vector : Sized {

    type Value: Copy + Default;
    
    /**
     * The dimension of the vector
     */
    fn dim(&self) -> usize;

    /**
     * Gets a value from this vector
     */
    fn get(&self, index: usize) -> Self::Value;

    /**
     * Sets a value on this vector
     */
    fn set(&mut self, index: usize, value: Self::Value);

    /**
     * Assigns values to the vector
     */
    fn assign_vec(&mut self, v: &Vec<Self::Value>) -> Result<(), OperationError> {
	if self.dim() == v.len() {
	    for i in 0..self.dim() {
		self.set(i, v[i]);
	    }
	    Ok(())
	} else {
	    Err(OperationError::BadDim)
	}
    }
    
    /**
     * Adds the supplied vector to this one
     */
    fn assign_add<V>(&mut self, other: &V) -> Result<(), OperationError> where V: Vector<Value = Self::Value>, Self::Value: Add<Output = Self::Value>{
	if self.dim() != other.dim() {
	    Err(OperationError::BadDim)
	} else {
	    for i in 0..self.dim() {
		self.set(i, self.get(i) + other.get(i));
	    }
	    Ok(())
	}
    }

    /**
     * Subtracts the supplied vector from this one
     */
    fn assign_sub<V>(&mut self, other: &V) -> Result<(), OperationError>  where V: Vector<Value = Self::Value>, Self::Value: Sub<Output = Self::Value>{
	if self.dim() != other.dim() {
	    Err(OperationError::BadDim)
	} else {
	    for i in 0..self.dim() {
		self.set(i, self.get(i) - other.get(i));
	    }
	    Ok(())
	}
    }

    /**
     * Negates the vector
     */
    fn assign_neg(&mut self) where Self::Value: Neg<Output = Self::Value> {
	for i in 0..self.dim() {
	    self.set(i, - self.get(i));
	}
    }

    
    /**
     * Multiplies the supplied vector with the supplied scalar
     */
    fn assign_mul_scalar(&mut self, other: Self::Value) where Self::Value: Mul<Output = Self::Value>{
	for i in 0..self.dim() {
	    self.set(i, self.get(i) * other);
	}
    }

    
    /**
     * Divides the supplied vector through the supplied scalar
     */
    fn assign_div_scalar(&mut self, other: Self::Value) where Self::Value: Div<Output = Self::Value>{
	for i in 0..self.dim() {
	    self.set(i, self.get(i) / other);
	}
    }

    /**
     * Assigns the
     */
    fn assign_mul_matrix<M, V>(&mut self, matrix: &M, vector: &V) -> Result<(), OperationError> where
	V: Vector<Value = Self::Value>,
	M: Matrix<Value = Self::Value>,
	Self::Value: Add<Output = Self::Value> + Default + Mul<Output = Self::Value> {

	if  vector.dim() == matrix.cols() && self.dim() == matrix.rows() {
	    for row in 0..matrix.rows() {
		let mut value = Self::Value::default();
		for col in 0..matrix.cols() {
		    value = value + vector.get(col) * matrix.get(row, col);
		}
		self.set(row, value);
	    }
	    Ok(())
	} else {
	    Err(OperationError::BadDim)
	}
    }
    
    /**
     * Calculates the length squared of the vector
     */
    fn len_squared(&self) -> Self::Value  where Self::Value: Mul<Output = Self::Value> + Add<Output = Self::Value> {
	let mut result = Self::Value::default();
	for i in 0..self.dim() {
	    let val = self.get(i);
	    result = result + val * val;
	}
	result
    }
    
    /**
     * Calculates the length of the vector
     */
    fn len(&self) -> Self::Value where Self::Value: Into<f64> + From<f64> {
	let mut result = 0.0;
	for i in 0..self.dim() {
	    let val: f64 = self.get(i).into();
	    result = result + val * val;
	}
	result = result.sqrt();
	result.into()
    }

    /**
     * Calculates the scalar product
     */
    fn scalar_product<V>(&self, other: &V) -> Self::Value where V: Vector<Value = Self::Value>, Self::Value: Mul<Output = Self::Value> + Add<Output = Self::Value> {
	let mut result = Self::Value::default();
	for i in 0..self.dim() {
	    result = result + self.get(i) * other.get(i);
	}
	result
    }
}

/**
 * A variable length vector
 */
pub trait VariableVector : Vector {

    /**
     * Creates a new vector
     */
    fn new(dim: usize) -> Self;

    /**
     * Creates a new vector from the supplied values
     */
    fn from_vec(values: Vec<Self::Value>) -> Self {
	let mut v = Self::new(values.len());
	v.assign_vec(&values).unwrap();
	v
    }
    
}

/**
 * A variable length vector
 */
pub trait FixedVector : Vector {

    /**
     * Creates a new vector
     */
    fn new() -> Self;
    
}

pub trait Vector2 :  FixedVector {

    /**
     * Creates a new vector for the given coordinates
     */
    fn from_values(x: Self::Value, y: Self::Value) -> Self;

    /**
     * Gets the x coordinate
     */
    fn get_x(&self) -> Self::Value {
	self.get(0)
    }

    /**
     * Sets the x coordinate
     */
    fn set_x(&mut self, value: Self::Value) {
	self.set(0, value);
    }

    /**
     * Gets the y coordinate
     */
    fn get_y(&self) -> Self::Value {
	self.get(1)
    }

    /**
     * Sets the y coordinate
     */
    fn set_y(&mut self, value: Self::Value) {
	self.set(1, value);
    }    
    
}


pub trait Vector3 :  FixedVector {

    /**
     * Creates a new vector for the given coordinates
     */
    fn from_values(x: Self::Value, y: Self::Value, z: Self::Value) -> Self;

    /**
     * Gets the x coordinate
     */
    fn get_x(&self) -> Self::Value {
	self.get(0)
    }

    /**
     * Sets the x coordinate
     */
    fn set_x(&mut self, value: Self::Value) {
	self.set(0, value);
    }

    /**
     * Gets the y coordinate
     */
    fn get_y(&self) -> Self::Value {
	self.get(1)
    }

    /**
     * Sets the y coordinate
     */
    fn set_y(&mut self, value: Self::Value) {
	self.set(1, value);
    }

    /**
     * Gets the z coordinate
     */
    fn get_z(&self) -> Self::Value {
	self.get(2)
    }

    /**
     * Sets the z coordinate
     */
    fn set_z(&mut self, value: Self::Value) {
	self.set(2, value);
    }    

}

macro_rules! define_variable_vector_type {

    ($type_name:ident, $value_type_name:ident) => {

	/**
	 * A variable length vector type for $value_type_name
	 */
	#[derive(Debug, PartialEq)]
	pub struct $type_name {

	    /**
	     * The values
	     */
	    values: Vec<$value_type_name>,

	}

	impl VariableVector for $type_name {

	    /**
	     * Creates a new vector with all values set to default
	     */
	    fn new(dim: usize) -> Self {
		$type_name {
		    values: vec![Self::Value::default(); dim],
		}
	    }
	    
	}

	impl Vector for $type_name {

	    type Value = $value_type_name;
	    
	    fn dim(&self) -> usize {
		self.values.len()
	    }

	    fn get(&self, index: usize) -> Self::Value {
		self.values[index]
	    }

	    fn set(&mut self, index: usize, value: Self::Value) {
		self.values[index] = value;
	    }
	    
	}	
    }
}

define_variable_vector_type!(VectorVarF64, f64);

macro_rules! define_fixed_vector_type_2 {
    
    ($type_name:ident, $value_type_name:ident) => {

	define_fixed_vector_type_n!($type_name, $value_type_name, 2);
	
	impl $type_name {

	    /**
	     * Creates a vector using the supplied array
	     */
	    fn from_array(values: [$value_type_name; 2]) -> Self {
		$type_name {
		    values,
		}
	    }
	    
	}
	
	impl Vector2 for $type_name {

	    fn from_values(x: $value_type_name, y: $value_type_name) -> Self {
		$type_name {
		    values: [x, y]
		}
	    }

	}
	
    }
}

macro_rules! define_fixed_vector_type_3 {
    
    ($type_name:ident, $value_type_name:ident) => {
	
	define_fixed_vector_type_n!($type_name, $value_type_name, 3);
	
	impl $type_name {

	    /**
	     * Creates a vector using the supplied array
	     */
	    fn from_array(values: [$value_type_name; 3]) -> Self {
		$type_name {
		    values,
		}
	    }
	    
	}
	
	impl Vector3 for $type_name {
	    
	    fn from_values(x: $value_type_name, y: $value_type_name, z: $value_type_name) -> Self {
		$type_name {
		    values: [x, y, z],
		}
	    }

	}
	
    }
}

macro_rules! define_fixed_vector_type_n {
    ($type_name:ident, $value_type_name:ident, $dimension:literal) => {

	#[derive(Copy, Clone, Debug, PartialEq)]
	pub struct $type_name {
	    values: [$value_type_name; $dimension],
	}

	impl Vector for $type_name {

	    type Value = $value_type_name;

	    fn dim(&self) -> usize {
		$dimension
	    }

	    fn get(&self, index: usize) -> Self::Value {
		self.values[index]
	    }

	    fn set(&mut self, index: usize, value: Self::Value) {
		self.values[index] = value;
	    }
	    
	}
	
	impl FixedVector for $type_name {

	    fn new() -> Self {
		$type_name {
		    values: [Self::Value::default(); $dimension],
		}
	    }
	    
	}

	impl Add for $type_name {

	    type Output = $type_name;
	    
	    fn add(mut self, other: Self) -> Self {
		self.assign_add(&other).unwrap();
		self
	    }
	    
	}

	impl Sub for $type_name {

	    type Output = $type_name;
	    
	    fn sub(mut self, other: Self) -> Self {
		self.assign_sub(&other).unwrap();
		self
	    }
	    
	}

	impl Neg for $type_name {

	    type Output = $type_name;

	    fn neg(mut self) -> Self {
		self.assign_neg();
		self
	    }
	    
	}

	impl Mul<$value_type_name> for $type_name {

	    type Output = $type_name;
	    
	    fn mul(mut self, other: $value_type_name) -> Self {
		self.assign_mul_scalar(other);
		self
	    }
	    
	}

	impl Div<$value_type_name> for $type_name {

	    type Output = $type_name;
	    
	    fn div(mut self, other: $value_type_name) -> Self {
		self.assign_div_scalar(other);
		self
	    }
	    
	}
    }
}

define_fixed_vector_type_2!(Vector2F32, f32);
define_fixed_vector_type_3!(Vector3F32, f32);

define_fixed_vector_type_2!(Vector2F64, f64);
define_fixed_vector_type_3!(Vector3F64, f64);


#[cfg(test)]
mod test {

    use super::*;
    
    #[test]
    fn new_variable_vector() {
	let vector = VectorVarF64::new(2);
	assert_eq!(2, vector.dim());
	assert_eq!(0.0, vector.get(0));
	assert_eq!(0.0, vector.get(1));
    }

    #[test]
    fn vector_buf_from_vec() {
	let vector = VectorVarF64::from_vec(vec![0.0, 1.0, 2.0]);
	assert_eq!(3, vector.dim());
	assert_eq!(0.0, vector.get(0));
	assert_eq!(1.0, vector.get(1));
	assert_eq!(2.0, vector.get(2));
    }    

    #[test]
    fn add_bad_dim() {
	let mut first = VectorVarF64::new(3);
	let second = VectorVarF64::new(2);
	assert_eq!(OperationError::BadDim, first.assign_add(&second).unwrap_err());
    }

    #[test]
    fn add() {
	let mut first = VectorVarF64::from_vec(vec![1.0, 2.0]);
	let second = VectorVarF64::from_vec(vec![1.0, 3.0]);
	first.assign_add(&second).unwrap();
	assert_eq!(2, first.dim());
	assert_eq!(2.0, first.get(0));
	assert_eq!(5.0, first.get(1));
    }

    
    #[test]
    fn sub_bad_dim() {
	let mut first = VectorVarF64::new(3);
	let second = VectorVarF64::new(2);
	assert_eq!(OperationError::BadDim, first.assign_sub(&second).unwrap_err());
    }

    #[test]
    fn sub() {
	let mut first = VectorVarF64::from_vec(vec![1.0, 2.0]);
	let second = VectorVarF64::from_vec(vec![1.0, 3.0]);
	first.assign_sub(&second).unwrap();
	assert_eq!(2, first.dim());
	assert_eq!(0.0, first.get(0));
	assert_eq!(-1.0, first.get(1));
    }

    #[test]
    fn mul() {
	let mut first = VectorVarF64::from_vec(vec![1.0, 2.0]);
	let second = 2.0;
	first.assign_mul_scalar(second);
	assert_eq!(2, first.dim());
	assert_eq!(2.0, first.get(0));
	assert_eq!(4.0, first.get(1));
    }

    #[test]
    fn div() {
	let mut first = VectorVarF64::from_vec(vec![1.0, 2.0]);
	let second = 2.0;
	first.assign_div_scalar(second);
	assert_eq!(2, first.dim());
	assert_eq!(0.5, first.get(0));
	assert_eq!(1.0, first.get(1));
    }
    
    #[test]
    fn len_squared() {
	let first = VectorVarF64::from_vec(vec![4.0, 3.0]);
	assert_eq!(25.0, first.len_squared());
    }
    
    #[test]
    fn len() {
	let first = VectorVarF64::from_vec(vec![4.0, 3.0]);
	assert_eq!(5.0, first.len());
    }

    #[test]
    fn scalar_product() {
	let first = VectorVarF64::from_vec(vec![4.0, 3.0]);
	let second = VectorVarF64::from_vec(vec![2.0, 5.0]);
	
	assert_eq!(23.0, first.scalar_product(&second));
    }
}
