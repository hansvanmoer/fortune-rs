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

/**
 * Utility functions and structs for 2D affine transforms
 */

use crate::matrix::{FixedMatrix, Matrix, Matrix3x3F32};
use crate::vector::{FixedVector, Vector, Vector3, Vector3F32};

/**
 * Represents a 2D transform
 */
pub struct Transform {
    /**
     * The linear tranformation matrix 
     */
    matrix: Matrix3x3F32,
}

impl Transform {
    
    pub fn transform_values(&self, x: f32, y: f32) -> (f32, f32) {
	let mut result = Vector3F32::new();
	result.assign_mul_matrix(&self.matrix, &Vector3F32::from_values(x, y, 1.0));
	(result.get(0), result.get(1))
    }

}

/**
 * A builder for transforms
 */
struct TransformBuilder {
    /**
     * A stack of extended affine transformation matrices
     */
    stack: Vec<Matrix3x3F32>,

}

impl TransformBuilder {
    
    const IDENTITY: [f32; 9] = [
	1.0, 0.0, 0.0,
	0.0, 1.0, 0.0,
	0.0, 0.0, 1.0,
    ];
    
    /**
     * Creates a new transform factory
     */
    pub fn new() -> TransformBuilder {
	TransformBuilder {
	    stack: Vec::new(),
	}
    }
    
    pub fn push_scale(&mut self, sx: f32, sy: f32) -> usize {
	self.push_matrix(Matrix3x3F32::from_values(
	    sx, 0.0, 0.0,
	    0.0, sy, 0.0,
	    0.0, 0.0, 1.0,
	))
    }

    pub fn push_rotate(&mut self, angle: f32) -> usize {
	let cos = angle.cos();
	let sin = angle.sin();
	self.push_matrix(Matrix3x3F32::from_values(
	    cos, sin, 0.0,
	    -sin, cos, 0.0,
	    0.0, 0.0, 1.0,
	))
    }

    pub fn push_translate(&mut self, dx: f32, dy: f32) -> usize {
	self.push_matrix(Matrix3x3F32::from_values(
	    1.0, 0.0, dx,
	    0.0, 1.0, dy,
	    0.0, 0.0, 1.0,
	))
    }

    fn push_matrix(&mut self, matrix: Matrix3x3F32) -> usize {
	let index = self.stack.len();
	self.stack.push(matrix);
	index
    }

    fn clear(&mut self) {
	self.stack.clear();
    }
    
    pub fn build(&self) -> Transform {
	let mut result = Matrix3x3F32::from_array(Self::IDENTITY.clone());
	let mut buffer = Matrix3x3F32::new();
	for m in self.stack.iter() {
	    buffer.assign_mul_matrices(&*m, &result).unwrap();
	    std::mem::swap(&mut buffer, &mut result);
	}
	Transform {
	    matrix: result,
	}
    }
    
}

#[cfg(test)]
mod test {

    use super::*;

    use crate::vector::Vector2F32;
    use float_eq::assert_float_eq;
    
    #[test]
    fn identity() {
	let mut builder = TransformBuilder::new();
	let t = builder.build();
	assert_eq!((2.0, 3.0), t.transform_values(2.0, 3.0));
    }

    #[test]
    fn scale() {
	let mut builder = TransformBuilder::new();
	builder.push_scale(3.0, 4.0);
	let t = builder.build();
	assert_eq!((6.0, 12.0), t.transform_values(2.0, 3.0));
    }
    
    #[test]
    fn translation() {
	let mut builder = TransformBuilder::new();
	builder.push_translate(5.0, 7.0);
	let t = builder.build();
	assert_eq!((7.0, 10.0), t.transform_values(2.0, 3.0));
    }

    #[test]
    fn rotation() {
	let mut builder = TransformBuilder::new();
	builder.push_rotate(std::f32::consts::PI / 2.0);
	let t = builder.build();

	let (x, y) =  t.transform_values(0.0, 1.0);
	assert_float_eq!(x, 1.0, abs <= 0.00_1);
	assert_float_eq!(y, 0.0, abs <= 0.00_1);

	let (x, y) =  t.transform_values(1.0, 0.0);
	assert_float_eq!(x, 0.0, abs <= 0.00_1);
	assert_float_eq!(y, -1.0, abs <= 0.00_1);
	
	let (x, y) =  t.transform_values(0.0, -1.0);
	assert_float_eq!(x, - 1.0, abs <= 0.00_1);
	assert_float_eq!(y, 0.0, abs <= 0.00_1);

	let (x, y) =  t.transform_values(- 1.0, 0.0);
	assert_float_eq!(x, 0.0, abs <= 0.00_1);
	assert_float_eq!(y, 1.0, abs <= 0.00_1);
    }

    #[test]
    fn build_transform() {
	let mut builder = TransformBuilder::new();
	builder.push_translate(1.0, 2.0);
	builder.push_scale(3.0, 7.0);
	builder.push_rotate(std::f32::consts::PI / 2.0);
	let t = builder.build();

	let (x, y) =  t.transform_values(5.0, 2.0);

	assert_float_eq!(28.0, x, abs <= 0.00_1);
	assert_float_eq!(-18.0, y, abs <= 0.00_1);
    }

    #[test]
    fn clear_transform() {
	let mut builder = TransformBuilder::new();
	builder.push_translate(1.0, 2.0);
	builder.clear();
	
	let t = builder.build();
	
	assert_eq!((1.0, 2.0), t.transform_values(1.0, 2.0));
    }
    
}
