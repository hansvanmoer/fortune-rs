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
 * Utility functions and structs for quadratic aritmetic
 */

/**
 * The possible solutions of a quadratic equation
 */

#[derive(Debug, PartialEq)]
pub enum Solution {

    /**
     * There are no roots
     */
    None,

    /**
     * There is one (double) root
     */
    One(f64),

    /**
     * There are two distinct roots
     */
    
    Two(f64, f64),
}

/**
 * Solves the quadratic equation
 */
pub fn solve(quadratic: f64, linear: f64, constant: f64) -> Solution {
    let discr = (linear * linear) - 4.0 * quadratic * constant;
    if discr > 0.0 {
	let discr_sqrt = discr.sqrt();
	let div = 2.0 * quadratic;
	let x1 =  (- linear - discr_sqrt) / div;
	let x2 =  (- linear + discr_sqrt) / div;
	if x1 < x2 {
	    Solution::Two(x1, x2)
	} else {
	    Solution::Two(x2, x1)
	}
    } else if discr < 0.0 {
	Solution::None
    } else {
	Solution::One(- linear / (2.0 * quadratic))
    }
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn test_quadratic_two_roots() {
	// 2 (x - 1) (x - 2) = 2 (x² - 3x + 2) = 2x² - 6x + 4 = 0
	assert_eq!(solve(2.0, -6.0, 4.0), Solution::Two(1.0, 2.0));
    }

    #[test]
    fn test_quadratic_one_root() {
	// 2 (x - 1) (x - 1) = 2 (x² - 2x + 1) = 2x² - 4x + 2 = 0
	assert_eq!(solve(2.0, -4.0, 2.0), Solution::One(1.0));
    }

    #[test]
    fn test_quadratic_no_roots() {
	// x² + 1 = 0
	assert_eq!(solve(1.0, 0.0, 1.0), Solution::None);
    }
    
}


