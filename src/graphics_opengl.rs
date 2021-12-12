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
 * Contains a renderer implementation using OpenGL
 * All unsafe OpenGL code should be placed here
 */

use crate::graphics::{Color, FragmentShaderRef, Graphics, GraphicsError, ProgramRef, VertexArrayRef, VertexShaderRef};
use crate::settings::Settings;

use log::debug;
use sdl2::VideoSubsystem;
use sdl2::video::{GLContext, Window};
use std::convert::TryInto;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

enum ShaderKind {
    Fragment,
    Vertex,
}

/**
 * An openGL shader
 */
struct Shader {

    /**
     * The openGL handle of the shader
     */
    handle: u32,

    /**
     * The kind of shader
     */
    kind: ShaderKind,
    
    /**
     * The shader's source code (must remain in memory for as long as the shader is used
     */
    code: String,
    
}

impl Shader {

    ///
    /// Create a shader from the supplied path
    ///
    
    pub fn from_path(path: &PathBuf, kind: ShaderKind) -> Result<Shader, GraphicsError> {
	let code = Shader::read_code(path).map_err(|_| GraphicsError::ShaderReadError)?;
	let handle = unsafe {
	    let handle = gl::CreateShader(match &kind {
		ShaderKind::Vertex => gl::VERTEX_SHADER,
		ShaderKind::Fragment => gl::FRAGMENT_SHADER,
	    });
	    gl::ShaderSource(
		handle,
		1,
		&(code.as_bytes().as_ptr().cast()),
		&(code.len().try_into().unwrap())
	    );
	    gl::CompileShader(handle);
	    let mut status = 0;
	    gl::GetShaderiv(handle, gl::COMPILE_STATUS, &mut status);
	    if status == 0 {
		let mut log_buffer: Vec<u8> = Vec::with_capacity(1024);
		let mut log_len = 0;
		gl::GetShaderInfoLog(handle, 1024, &mut log_len, log_buffer.as_mut_ptr().cast());
		log_buffer.set_len(log_len.try_into().unwrap());
		Err(GraphicsError::ShaderCompilationError(String::from_utf8_lossy(&log_buffer).into_owned()))
	    } else {
		Ok(handle)
	    }
	}?;
	Ok(Shader{
	    handle,
	    kind,
	    code,
	})
    }

    ///
    /// Read shader code
    ///
    
    fn read_code(path: &PathBuf) -> Result<String, std::io::Error> {
	let mut file = File::open(&path)?;
	let mut data = String::new();
	file.read_to_string(&mut data)?;
	Ok(data)
    }
    
}

impl Drop for Shader {

    fn drop(&mut self) {
	unsafe {
	    gl::DeleteShader(self.handle);
	}
    }
    
}

///
/// Represents a vertex array with its associated buffers
///

struct VertexArray {
    
    ///
    /// The opengl handle to the vertex array ID
    ///
    vertex_array_id: u32,

    ///
    /// The opengl handle to the vertex buffer
    ///
    vertex_buffer_id: u32,

    ///
    /// The opengl handle to the element buffer
    ///
    element_buffer_id: u32,
    
    ///
    /// The number of vertices to draw
    ///
    vertex_count: i32,
}

impl VertexArray {

    ///
    /// Creates a new VAO and associated VBO's
    ///
    
    fn new(vertices: Vec<f32>, indices: Vec<u32>) -> VertexArray{
	let (vertex_array_id, vertex_buffer_id, element_buffer_id) = unsafe{

	    let mut vertex_buffer_id = 0;
	    gl::GenBuffers(1, &mut vertex_buffer_id);

	    let mut element_buffer_id = 0;
	    gl::GenBuffers(1, &mut element_buffer_id);
	    
	    let mut vertex_array_id = 0;
	    gl::GenVertexArrays(1, &mut vertex_array_id);
	    gl::BindVertexArray(vertex_array_id);
	    
	    gl::BindBuffer(gl::ARRAY_BUFFER, vertex_buffer_id);
	    gl::BufferData(
		gl::ARRAY_BUFFER,
		(std::mem::size_of::<f32>() * vertices.len()) as isize,
		vertices.as_ptr().cast(),
		gl::STATIC_DRAW
	    );
	    
	    gl::VertexAttribPointer(
		0,
		3,
		gl::FLOAT,
		gl::FALSE,
		(std::mem::size_of::<f32>() * 3).try_into().unwrap(),
		0 as *const _,
	    );
	    gl::EnableVertexAttribArray(0);

	    gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, element_buffer_id);
	    gl::BufferData(
		gl::ELEMENT_ARRAY_BUFFER,
		(std::mem::size_of::<u32>() * indices.len()) as isize,
		indices.as_ptr().cast(),
		gl::STATIC_DRAW
	    );
	    
	    (vertex_array_id, vertex_buffer_id, element_buffer_id)
	};
	VertexArray {
	    vertex_buffer_id,
	    element_buffer_id,
	    vertex_array_id,
	    vertex_count: indices.len().try_into().unwrap(),
	}
    }

    fn draw(&self) {
	unsafe {
	    gl::BindVertexArray(self.vertex_array_id);
	    gl::DrawElements(gl::TRIANGLES, self.vertex_count, gl::UNSIGNED_INT, 0 as * const _);
	}
    }
}

///
/// An opengl program (rendering pipeline)
///

struct Program {

    ///
    /// The opengl handle for the program
    ///
    
    handle: u32,
    
}

impl Program {

    ///
    /// Create a new program
    ///
    
    fn new(vertex_shader_handle: u32, fragment_shader_handle: u32) -> Result<Program, GraphicsError> {
	let handle = unsafe {

	    let handle = gl::CreateProgram();
	    gl::AttachShader(handle, vertex_shader_handle);
	    gl::AttachShader(handle, fragment_shader_handle);
	    gl::LinkProgram(handle);

	    let mut status = 0;
	    gl::GetProgramiv(handle, gl::LINK_STATUS, &mut status);
	    if status == 0 {
		let mut log_buffer: Vec<u8> = Vec::with_capacity(1024);
		let mut log_len = 0;
		gl::GetShaderInfoLog(handle, 1024, &mut log_len, log_buffer.as_mut_ptr().cast());
		log_buffer.set_len(log_len.try_into().unwrap());
		Err(GraphicsError::ProgramLinkError(String::from_utf8_lossy(&log_buffer).into_owned()))
	    } else {
		gl::UseProgram(handle);
		Ok(handle)
	    }
	}?;
	Ok(Program {
	    handle,
	})
    }
    
    
}

///
/// This struct creates the OpenGL renderer and ensures that the openGL context is initialized
/// OpenGL calls can be made as long as this struct lives
///

pub struct OpenGLGraphics {

    ///
    /// OpenGL context
    ///

    context: GLContext,
    
    ///
    /// Shader directory
    ///

    shader_dir: PathBuf,

    ///
    /// All loaded openGL vertex shaders
    ///

    vertex_shaders: Vec<Shader>,

    ///
    /// All loaded openGL fragment shaders
    ///

    fragment_shaders: Vec<Shader>,

    ///
    /// All loaded vertex arrays
    ///

    vertex_arrays: Vec<VertexArray>,

    ///
    /// All loaded programs
    ///

    programs: Vec<Program>,
}


impl OpenGLGraphics {
    
    ///
    /// Initializes the factory and the underlying OpenGL context
    ///
    pub fn new(settings: &Settings, sdl_video: &VideoSubsystem, sdl_window: &Window) -> OpenGLGraphics {

	let mut shader_dir = settings.create_data_path();
	shader_dir.push("opengl");
	shader_dir.push("shaders");
	
	let context = sdl_window.gl_create_context().expect("could not load OpenGL context");
	gl::load_with(|s| sdl_video.gl_get_proc_address(s) as * const std::os::raw::c_void);
	unsafe {
	    gl::PolygonMode(gl::FRONT_AND_BACK, gl::LINE);
	}
	OpenGLGraphics {
	    context,
	    shader_dir,
	    vertex_shaders: Vec::new(),
	    fragment_shaders: Vec::new(),
	    vertex_arrays: Vec::new(),
	    programs: Vec::new(),
	}
    }
    
}

impl Graphics for OpenGLGraphics {

    ///
    /// Creates a fragment shader and returns a reference to it
    ///
    
    fn create_fragment_shader(&mut self, name: &str) -> Result<FragmentShaderRef, GraphicsError> {

	let mut path = self.shader_dir.clone();
	path.push(name);
	
	debug!("reading fragment shader from path {:?}", path);
	let id = self.fragment_shaders.len();
	self.fragment_shaders.push(Shader::from_path(&path, ShaderKind::Fragment)?);
	Ok(id)
    }

    ///
    /// Creates a vertex shader and returns a reference to it
    ///
    
    fn create_vertex_shader(&mut self, name: &str) -> Result<VertexShaderRef, GraphicsError> {

	let mut path = self.shader_dir.clone();
	path.push(name);
	
	debug!("reading vertex shader from file {:?}", path);
	let id = self.vertex_shaders.len();
	self.vertex_shaders.push(Shader::from_path(&path, ShaderKind::Vertex)?);
	Ok(id)
    }

    ///
    /// Creates a vertex buffer filled with triangle data and returns a reference to it
    ///
    fn create_triangles(&mut self, vertices: Vec<f32>, indices: Vec<u32>) -> Result<VertexArrayRef, GraphicsError> {
	let id = self.vertex_arrays.len();
	self.vertex_arrays.push(VertexArray::new(vertices, indices));
	Ok(id)
    }

    ///
    /// Creates a program
    ///
    
    fn create_program(&mut self, vertex_shader: VertexShaderRef, fragment_shader: FragmentShaderRef) -> Result<ProgramRef, GraphicsError> {
	if vertex_shader < self.vertex_shaders.len() {
	    if fragment_shader < self.fragment_shaders.len() {
		let id = self.programs.len();
		self.programs.push(Program::new(
		    self.vertex_shaders[vertex_shader].handle,
		    self.fragment_shaders[fragment_shader].handle
		)?);
		Ok(id)
	    } else {
		Err(GraphicsError::InvalidShader)
	    }
	} else {
	    Err(GraphicsError::InvalidShader)
	}
    }

    fn set_view(&mut self, left: f32, right: f32, top: f32, bottom: f32) {
	unsafe {
	    
	}
    }

    fn set_clear_color(&mut self, color: &Color) {
	unsafe {
	    gl::ClearColor(color.red, color.green, color.blue, color.alpha);
	}
    }

    fn clear(&mut self) {
	unsafe {
	    gl::Clear(gl::COLOR_BUFFER_BIT);
	}
    }

    fn draw_vertex_array(&self, id: VertexArrayRef) {
	self.vertex_arrays[id].draw();
    }
    
}
