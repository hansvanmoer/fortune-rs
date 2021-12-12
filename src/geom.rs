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
 * Utility functions for 2D geometry
 */

use crate::linear::solve_from_matrix;
use crate::matrix::{FixedMatrix, Matrix, Matrix2x3F64};
use crate::quadratic::{Solution as QuadraticSolution, solve as solve_quadratic};
use crate::vector::{Vector2, Vector2F64};

/**
 * A bounding box
 */
pub struct BoundingBox {

    /**
     * The points of the box
     */
    points: [Vector2F64; 4],

}

impl BoundingBox {

    pub fn new(x1: f64, x2: f64, y1: f64, y2: f64) -> BoundingBox {
	let (left, right) = if x1 > x2 {
	    (x2, x1)
	} else {
	    (x1, x2)
	};
	let (top, bottom) = if y1 > y2 {
	    (y2, y1)
	} else {
	    (y1, y2)
	};
	BoundingBox{
	    points: [
		Vector2F64::from_values(left, top),
		Vector2F64::from_values(right, top),
		Vector2F64::from_values(right, bottom),
		Vector2F64::from_values(left, bottom),
	    ]
	}
    }
}

/**
 * A helper struct to calculate intersections
 * The calculator provides various methods to calculate intersections but
 * reuses its buffer to avoid repeated memory allocations
 */

#[derive(Debug)]
pub struct IntersectionCalculator {

    /**
     * A 2x3 matrix
     */
    matrix: Matrix2x3F64,
    
}

impl IntersectionCalculator {

    /**
     * Creates a new intersection struct
     */
    pub fn new() -> IntersectionCalculator {
	IntersectionCalculator{
	    matrix: Matrix2x3F64::new(),
	}
    }
    
    /**
     * Intersects two lines
     */
    pub fn line_with_line(&mut self, first_point: &Vector2F64, first_dir: &Vector2F64,
			  second_point: &Vector2F64, second_dir: &Vector2F64) -> Option<Vector2F64> {
	self.set_parameter_equations(first_point, first_dir, second_point, second_dir);
	self.solve_for_parameters()
	    .map(|(k, _)| IntersectionCalculator::substitute_parameter(first_point, first_dir, k))
    }

    /**
     * Intersects two rays
     */
    pub fn ray_with_ray(&mut self,first_point: &Vector2F64, first_dir: &Vector2F64,
			  second_point: &Vector2F64, second_dir: &Vector2F64) -> Option<Vector2F64> {
	self.set_parameter_equations(first_point, first_dir, second_point, second_dir);
	self.solve_for_parameters()
	    .filter(|(k, j)| k >= &0.0 && j >= &0.0)
	    .map(|(k, _)| IntersectionCalculator::substitute_parameter(first_point, first_dir, k))
    }

    /**
     * Intersects a ray with a segment
     */
    pub fn ray_with_segment(&mut self, first_point: &Vector2F64, first_dir: &Vector2F64,
			    second_start: &Vector2F64, second_end: &Vector2F64) -> Option<Vector2F64> {
	let second_dir = Vector2F64::from_values(
	    second_end.get_x() - second_start.get_x(),
	    second_end.get_y() - second_start.get_y()
	);
	self.set_parameter_equations(first_point, first_dir, second_start, &second_dir);
	self.solve_for_parameters()
	    .filter(|(k, j)| k >= &0.0 && j >= &0.0 && j <= &1.0)
	    .map(|(k, _)| IntersectionCalculator::substitute_parameter(first_point, first_dir, k))
    }

    /**
     * Intersects a ray with a bounding box
     */
    pub fn ray_with_bounding_box(&mut self, first_point: &Vector2F64, first_dir: &Vector2F64, bounding_box: &BoundingBox) -> Option<Vector2F64> {

	self.ray_with_segment(first_point, first_dir,
			      &bounding_box.points[0], &bounding_box.points[1])
	    .or_else(|| self.ray_with_segment(first_point, first_dir,
					      &bounding_box.points[1], &bounding_box.points[2]))
	    .or_else(|| self.ray_with_segment(first_point, first_dir,
					      &bounding_box.points[2], &bounding_box.points[3]))
	    .or_else(|| self.ray_with_segment(first_point, first_dir,
					      &bounding_box.points[3], &bounding_box.points[0]))

    }

    /**
     * Returns the circle through the three specified points or None if they are colinear
     */
    pub fn circle_through_points(&mut self, first_point: &Vector2F64, second_point: &Vector2F64, third_point: &Vector2F64) -> Option<(Vector2F64, f64)> {
	let d1 = Vector2F64::from_values(
	    second_point.get_y() - first_point.get_y(),
	    first_point.get_x() - second_point.get_x()
	);
	let d2 = Vector2F64::from_values(
	    second_point.get_y() - third_point.get_y(),
	    third_point.get_x() - second_point.get_x()
	);

	if vectors_are_dependent(&d1, &d2) {
	    None
	} else {
	    let p1 = Vector2F64::from_values(
		(first_point.get_x() + second_point.get_x()) / 2.0,
		(first_point.get_y() + second_point.get_y()) / 2.0
	    );
	    let p2 = Vector2F64::from_values(
		(third_point.get_x() + second_point.get_x()) / 2.0,
		(third_point.get_y() + second_point.get_y()) / 2.0
	    );
	    let focus = self.line_with_line(&p1, &d1, &p2, &d2).expect("expected intersection");
	    let radius = distance_between_points(&focus, first_point);
	    Some((focus, radius))
	}
    }
    
    /**
     * Sets the matrix to the specified parameter equation of two lines
     */
    
    fn set_parameter_equations(&mut self, first_point: &Vector2F64, first_dir: &Vector2F64,
				second_point: &Vector2F64, second_dir: &Vector2F64) {
	self.matrix.set(0, 0, first_dir.get_x());
	self.matrix.set(0, 1, - second_dir.get_x());
	self.matrix.set(0, 2, first_point.get_x() - second_point.get_x());
	self.matrix.set(1, 0, first_dir.get_y());
	self.matrix.set(1, 1, - second_dir.get_y());
	self.matrix.set(1, 2, first_point.get_y() - second_point.get_y());
    }

    /**
     * Solves the line intersection system and returns the parameters
     */
    fn solve_for_parameters(&mut self) -> Option<(f64, f64)>{
	solve_from_matrix(&mut self.matrix)
	    .ok()
	    .map(|solution| (solution[0], solution[1]))
    }
    
    /**
     * Substitutes the parameter in the line equation point + k * dir = the resultant point on the line
     */
    fn substitute_parameter(point: &Vector2F64, dir: &Vector2F64, param: f64) -> Vector2F64 {
	Vector2F64::from_values(
	    point.get_x() + param * dir.get_x(),
	    point.get_y() + param * dir.get_y()
	)
    }

}


/**
 * Intersects two lines specified by two start points and two directions
 * Returns None if both lines are either identical or parallel
 */
pub fn intersect_line_with_line(first_point: &Vector2F64, first_dir: &Vector2F64, second_point: &Vector2F64, second_dir: &Vector2F64) -> Option<Vector2F64> {
    let mut calculator = IntersectionCalculator::new();
    calculator.line_with_line(first_point, first_dir, second_point, second_dir)
}


/**
 * Intersects two rays specified by two start points and two directions
 * Returns None if both rays are identical, parallel or do not intersect
 */
pub fn intersect_ray_with_ray(first_point: &Vector2F64, first_dir: &Vector2F64, second_point: &Vector2F64, second_dir: &Vector2F64) -> Option<Vector2F64> {
    let mut calculator = IntersectionCalculator::new();
    calculator.ray_with_ray(first_point, first_dir, second_point, second_dir)
}

/**
 * Intersects two rays specified by two start points and two directions
 * Returns None if both rays are identical, parallel or do not intersect
 */
pub fn intersect_ray_with_segment(first_point: &Vector2F64, first_dir: &Vector2F64, second_start: &Vector2F64, second_end: &Vector2F64) -> Option<Vector2F64> {
    let mut calculator = IntersectionCalculator::new();
    calculator.ray_with_segment(first_point, first_dir, second_start, second_end)
}

/**
 * Intersects a ray with a bounding box
 */
pub fn intersect_ray_with_bounding_box(first_point: &Vector2F64, first_dir: &Vector2F64, bounding_box: &BoundingBox) -> Option<Vector2F64>{
    let mut calculator = IntersectionCalculator::new();
    calculator.ray_with_bounding_box(first_point, first_dir, bounding_box)
}

/**
 * Intersection between two parabolas
 */
pub enum ParabolaIntersection {

    /**
     * No intersections
     */
    None,

    /**
     * One intersection
     */
    One(Vector2F64),

    /**
     * Two intersections
     */
    Two(Vector2F64, Vector2F64),

    /**
     * An infinite amount of intersections (i.e. intersecting parabolas are identical)
     */
    Infinite
}

/**
 * Calculates a parabola from a focus and a directrix.
 * E.g. P is point on parabola <=> dist(P, focus) = dist(P, directrix)
 */
fn calc_parabola_from_focus(focus: &Vector2F64, dir_y: f64) -> (f64, f64, f64) {
    //
    // dist²(P, focus) = dist²(P, dir)
    // (x - focus.x)² + (y - focus.y)² = (y - dir_y)²
    // y² - 2 * focus.y * y + focus.y² + x² - 2 * focus.x * x + focus.x² = y² - 2 * dir_y * y + dir_y²
    // 2 * (focus.y - dir_y) * y = x² - 2 * focus.x * x + focus.x² + focus.y²  - dir_y²
    // a = 1 / (2 * (focus.y - dir_y))
    // y = a x² - 2 * focus.x * a * x + (focus.x² + focus.y²  - dir_y²) * a

    let a = 1.0 / (2.0 * (focus.get_y() - dir_y));
    let b = - 2.0 * focus.get_x() * a;
    let c = (focus.get_x() * focus.get_x() + focus.get_y() * focus.get_y() - dir_y * dir_y) * a;
    (a, b, c)
}

/**
 * Calculates the intersection between two parabolas specified by their focii and a horizontal directrix
 */
pub fn intersect_parabolas_from_foci(first_focus: &Vector2F64, second_focus: &Vector2F64, dir_y: f64) -> ParabolaIntersection{
    let (a1, b1, c1) = calc_parabola_from_focus(first_focus, dir_y);
    let (a2, b2, c2) = calc_parabola_from_focus(second_focus, dir_y);
    let a = a2 - a1;
    let b = b2 - b1;
    let c = c2 - c1;
    if a == 0.0 && b == 0.0 && c == 0.0 {
	// degenerate case
	ParabolaIntersection::Infinite
    } else {
	match solve_quadratic(a, b, c) {
	    QuadraticSolution::None => ParabolaIntersection::None,
	    QuadraticSolution::One(x) => ParabolaIntersection::One(
		Vector2F64::from_values(x, a1 * x * x + b1 * x + c1)
	    ),
	    QuadraticSolution::Two(x1, x2) => ParabolaIntersection::Two(
		Vector2F64::from_values(x1, a1 * x1 * x1 + b1 * x1 + c1),
		Vector2F64::from_values(x2, a1 * x2 * x2 + b1 * x2 + c1)
	    ),
	}
    }
}

/**
 * Calculates the distance between two points
 */
fn distance_between_points(first: &Vector2F64, second: &Vector2F64) -> f64 {
    let dx = first.get_x() - second.get_x();
    let dy = first.get_y() - second.get_y();
    (dx * dx + dy * dy).sqrt()
}

/**
 * Tests whether vectors are dependent
 */
fn vectors_are_dependent(first: &Vector2F64, second: &Vector2F64) -> bool {
    first.get_x() * second.get_y() - second.get_x() * first.get_y() == 0.0
}

/**
 * Returns the focus and radius of a circle through the three specified points or None if they are colinear
 */
pub fn circle_through_points(first: &Vector2F64, second: &Vector2F64, third: &Vector2F64) -> Option<(Vector2F64, f64)>{
    let mut calculator = IntersectionCalculator::new();
    calculator.circle_through_points(first, second, third)
}

/**
 * Checks whether two vectors represent a clockwise turn
 */
pub fn is_clockwise(first: &Vector2F64, second: &Vector2F64) -> bool {
    if first.get_x() * second.get_y() - first.get_y() * second.get_x() < 0.0 {
	true
    } else {
	false
    }
}


#[cfg(test)]
mod test {

    use float_eq::assert_float_eq;
    
    use super::*;

    #[test]
    fn test_intersect_parallel_lines() {
	let p1 = Vector2F64::from_values(1.0, 1.0);
	let p2 = Vector2F64::from_values(-3.0, 4.0);
	let d1 = Vector2F64::from_values(1.0, 2.0);
	let d2 = Vector2F64::from_values(-2.0, -4.0);
	
	assert_eq!(intersect_line_with_line(&p1, &d1, &p2, &d2), None);
    }


    #[test]
    fn test_intersect_equal_lines() {
	let p1 = Vector2F64::from_values(1.0, 1.0);
	let d1 = Vector2F64::from_values(-3.0, 4.0);

	assert_eq!(intersect_line_with_line(&p1, &d1, &p1, &d1), None);
    }
    
    
    #[test]
    fn test_intersect_lines() {
	let p1 = Vector2F64::from_values(1.0, 1.0);
	let d1 = Vector2F64::from_values(1.0, 1.0);
	
	let p2 = Vector2F64::from_values(-3.0, 4.0);
	let d2 = Vector2F64::from_values(1.0, 0.0);

	assert_eq!(intersect_line_with_line(&p1, &d1, &p2, &d2), Some(Vector2F64::from_values(4.0, 4.0)));
    }


    #[test]
    fn test_intersect_parallel_rays() {
	let p1 = Vector2F64::from_values(1.0, 1.0);
	let p2 = Vector2F64::from_values(-3.0, 4.0);
	let d1 = Vector2F64::from_values(1.0, 2.0);
	let d2 = Vector2F64::from_values(- 2.0, - 4.0);

	assert_eq!(intersect_ray_with_ray(&p1, &d1, &p2, &d2), None);
    }


    #[test]
    fn test_intersect_equal_rays() {
	let p1 = Vector2F64::from_values(1.0, 1.0);
	let d1 = Vector2F64::from_values(-3.0, 4.0);

	assert_eq!(intersect_ray_with_ray(&p1, &d1, &p1, &d1), None);
    }
    
    
    #[test]
    fn test_intersect_rays() {
	let p1 = Vector2F64::from_values(1.0, 1.0);
	let d1 = Vector2F64::from_values(1.0, 1.0);
	
	let p2 = Vector2F64::from_values(-3.0, 4.0);
	let d2 = Vector2F64::from_values(1.0, 0.0);

	assert_eq!(intersect_ray_with_ray(&p1, &d1, &p2, &d2), Some(Vector2F64::from_values(4.0, 4.0)));
    }

    
    #[test]
    fn test_non_intersecting_rays() {
	let p1 = Vector2F64::from_values(1.0, 1.0);
	let d1 = Vector2F64::from_values(1.0, 1.0);
	
	let p2 = Vector2F64::from_values(-3.0, 4.0);
	let d2 = Vector2F64::from_values(-1.0, 0.0);

	assert_eq!(intersect_ray_with_ray(&p1, &d1, &p2, &d2), None);
    }

    #[test]
    fn test_intersect_parabolas() {
	let f1 = Vector2F64::from_values(100.0, 130.0);
	let f2 = Vector2F64::from_values(500.0, 340.0);
	let line_y = 600.0;

	match intersect_parabolas_from_foci(&f1, &f2, line_y) {
	    ParabolaIntersection::Two(first, second) => {
		let d11 = distance_between_points(&first, &f1);
		let d12 = distance_between_points(&first, &f2);
		let d13 = (first.get_y() - line_y).abs();
		let d21 = distance_between_points(&second, &f1);
		let d22 = distance_between_points(&second, &f2);
		let d23 = (second.get_y() - line_y).abs();
		
		assert_float_eq!(d11, d12, abs <= 0.000_1);	   
		assert_float_eq!(d21, d22, abs <= 0.000_1);
		assert_float_eq!(d11, d13, abs <= 0.000_1);
		assert_float_eq!(d21, d23, abs <= 0.000_1);
	    },
	    _ => {
		panic!("expected two intersections");
	    }
	}
    }

    #[test]
    fn test_circle_through_colinear_points() {
	let p1 = Vector2F64::from_values(1.0, 1.0);
	let p2 = Vector2F64::from_values(2.0, 2.0);
	let p3 = Vector2F64::from_values(44.0, 44.0);

	assert_eq!(circle_through_points(&p1, &p2, &p3), None);
    }

    
    #[test]
    fn test_circle_through_points() {
	let p1 = Vector2F64::from_values(1.0, 1.0);
	let p2 = Vector2F64::from_values(100.0, 400.0);
	let p3 = Vector2F64::from_values(400.0, -200.0);

	let (focus, radius) = circle_through_points(&p1, &p2, &p3).expect("expected circle and radius");

	let d1 = distance_between_points(&p1, &focus);
	let d2 = distance_between_points(&p2, &focus);
	let d3 = distance_between_points(&p3, &focus);
	
	assert_float_eq!(d1, radius, abs <= 0.000_1);
	assert_float_eq!(d2, radius, abs <= 0.000_1);
	assert_float_eq!(d3, radius, abs <= 0.000_1);
    }

    #[test]
    fn test_intersect_ray_with_segment() {
	let p1 = Vector2F64::from_values(1.0, 1.0);
	let d1 = Vector2F64::from_values(1.0, 1.0);
	
	let start = Vector2F64::from_values(4.0, -100.0);
	let end = Vector2F64::from_values(4.0, 50.0);

	assert_eq!(intersect_ray_with_segment(&p1, &d1, &start, &end), Some(Vector2F64::from_values(4.0, 4.0)));
    }
    
    #[test]
    fn test_intersect_ray_with_segment_edge() {
	let p1 = Vector2F64::from_values(1.0, 1.0);
	let d1 = Vector2F64::from_values(1.0, 1.0);
	
	let start = Vector2F64::from_values(4.0, -100.0);
	let end = Vector2F64::from_values(4.0, 4.0);

	assert_eq!(intersect_ray_with_segment(&p1, &d1, &start, &end), Some(Vector2F64::from_values(4.0, 4.0)));
    }

    
    #[test]
    fn test_not_intersect_ray_with_segment() {
	let p1 = Vector2F64::from_values(1.0, 1.0);
	let d1 = Vector2F64::from_values(1.0, 1.0);
	
	let start = Vector2F64::from_values(4.0, -100.0);
	let end = Vector2F64::from_values(4.0, 3.0);

	assert_eq!(intersect_ray_with_segment(&p1, &d1, &start, &end), None);
    }

    #[test]
    fn test_clockwise_vector() {
	let p1 = Vector2F64::from_values(- 1.0, 1.0);
	let p2 = Vector2F64::from_values(0.5, 0.1);

	assert_eq!(is_clockwise(&p1, &p2), true);
	assert_eq!(is_clockwise(&p2, &p1), false);
    }
    
}
