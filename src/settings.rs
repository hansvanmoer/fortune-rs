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
 * Application settings and associated functions and types
 */

use log::info;
use std::path::PathBuf;

/**
 * Represents current application settings
 */
#[derive(Debug)]
pub struct Settings {

    /**
     * The path of the application's (read only) data
     */
    data_path: PathBuf,
    
}

impl Settings {

    /**
     * Creates a new settings object from the environment
     * Note: logging is not enabled in this function
     */
    pub fn new() -> Settings {
	let mut data_path = std::env::current_dir().unwrap_or_else(|_| PathBuf::new());
	data_path.push("data");
	if !data_path.is_dir() {
	    data_path.pop();
	    data_path.pop();
	    data_path.pop();
	    data_path.push("data");
	    if !data_path.is_dir() {
		data_path = PathBuf::new();
	    }
	}
	
	Settings {
	    data_path,
	}
    }

    pub fn log(&self){
	info!("settings:");
	info!("data path: {:?}", self.data_path);
    }

    /**
     * Creates a new PathBuf representing the data path
     */
    pub fn create_data_path(&self) -> PathBuf {
	self.data_path.clone()
    }
    
}
