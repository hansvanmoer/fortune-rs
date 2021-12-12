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
 * Main application subroutines
 */

mod geom;
mod graphics;
mod graphics_opengl;
mod linear;
mod quadratic;
mod matrix;
mod minor_matrix_view;
mod row_matrix_view;
mod settings;
mod sub_matrix_view;
mod transposed_matrix_view;
mod transform;
mod vector;
mod voronoi;


use crate::graphics::{Color, Graphics};
use crate::graphics_opengl::OpenGLGraphics;
use crate::settings::Settings;

use log::info;
use sdl2::event::Event;

/**
 * Application entry point
 */
fn main() {

    let settings = Settings::new();
    
    env_logger::init();

    info!("application started...");
    settings.log();

    info!("generating voronoi diagram");
    let diagram = voronoi::generate();
    
    let sdl_context = sdl2::init().expect("could not initialize SDL context");

    let sdl_video = sdl_context.video().expect("could not initialize SDL video subsystem");

    let sdl_window = sdl_video.window("Test", 1500, 1500)
	.position_centered()
	.opengl()
	.build()
	.expect("could not intiialize SDL window");

    let mut graphics = OpenGLGraphics::new(&settings, &sdl_video, &sdl_window);

    let vertex_shader = graphics.create_vertex_shader("vertex.shader").expect("unable to create vertex shader");
    let fragment_shader = graphics.create_fragment_shader("fragment.shader").expect("unable to create fragment shader");
    let program = graphics.create_program(vertex_shader, fragment_shader).expect("unable to create graphics pipeline");
    
    let (vertices, indices) = diagram.create_triangles();
    println!("{:?}", vertices);
    let vertex_buffer = graphics.create_triangles(vertices, indices).expect("unable to create vertex array");

    let mut sdl_event_pump = sdl_context.event_pump()
	.expect("could not initialize SDL events");

    'run_loop: loop {
	for event in sdl_event_pump.poll_iter() {
	    match event {
		Event::Quit{..} => break 'run_loop,
		_ => {},
	    }
	}
	
	graphics.set_clear_color(&Color::black());
	graphics.clear();
	graphics.draw_vertex_array(vertex_buffer);
	
	sdl_window.gl_swap_window();
	
    }

}

