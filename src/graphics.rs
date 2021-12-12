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
 * Contains the structs traits and functions related to graphics
 */

/**
 * A color struct
 */
pub struct Color {
    pub red: f32,
    pub green: f32,
    pub blue: f32,
    pub alpha: f32,
}

impl Color {

    /**
     * Creates a new color representing an opaque black
     */
    pub fn black() -> Color {
	Color{red: 0.0, green: 0.0, blue: 0.0, alpha: 1.0}
    }
    
}

///
/// Errors that can occur while interacting with the graphics subsystem
///
#[derive(Debug, PartialEq)]
pub enum GraphicsError {

    ///
    /// Shader file not found
    ///

    ShaderFileNotFound,

    ///
    /// IO error while reading shader code
    ///

    ShaderReadError,

    ///
    /// Shader did not compile
    ///
    
    ShaderCompilationError(String),

    ///
    /// Program link failed
    ///
    
    ProgramLinkError(String),

    ///
    /// Invalid shader
    ///
    
    InvalidShader,
}

pub type FragmentShaderRef = usize;

pub type VertexShaderRef = usize;

pub type VertexArrayRef = usize;
pub type ProgramRef = usize;

///
/// Describes the interface to be implemented by each graphics implementation
///

pub trait Graphics {
    
    ///
    /// Creates a fragment shader for the specified name
    ///

    fn create_fragment_shader(&mut self, name: &str) -> Result<FragmentShaderRef, GraphicsError>;

    ///
    /// Creates a vertex shader for the specified name
    ///
    
    fn create_vertex_shader(&mut self, name: &str) -> Result<VertexShaderRef, GraphicsError>;

    ///
    /// Creates a vertex buffer filled with triangles.
    /// Coordinates should be specified as x, y, z
    ///
    
    fn create_triangles(&mut self, vertices: Vec<f32>, indices: Vec<u32>) -> Result<VertexArrayRef, GraphicsError>;

    ///
    /// Creates and uses a graphics pipeline program
    ///
    
    fn create_program(&mut self, vertex_shader: VertexShaderRef, fragment_shader: FragmentShaderRef) -> Result<ProgramRef, GraphicsError>;

    ///
    /// Sets the view
    ///

    fn set_view(&mut self, left: f32, right: f32, top: f32, bottom: f32);
    
    ///
    /// Sets the color used to clear the drawing area
    ///
    
    fn set_clear_color(&mut self, color: &Color);

    ///
    /// Clears the drawing area
    ///
    
    fn clear(&mut self);

    ///
    /// Draws the specified vertex array
    ///
    
    fn draw_vertex_array(&self, vertex_array: VertexArrayRef);
}
