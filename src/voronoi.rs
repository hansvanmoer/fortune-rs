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
 * Calculates the voronoi diagram
 */

use crate::geom::{intersect_parabolas_from_foci, is_clockwise, BoundingBox, IntersectionCalculator, ParabolaIntersection};
use crate::vector::{Vector2, Vector2F64};

use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::fmt::{Display, Error as FormatError, Formatter};

#[derive(Debug, PartialEq)]
pub struct Vertex {
    id: usize,
    x: f64,
    y: f64,
}

#[derive(Debug, PartialEq)]
pub struct HalfEdge {
    id: usize,
    face_id: usize,
    start_id: usize,
    twin_id: Option<usize>,
    prev_id: usize,
    next_id: usize,
}

struct HalfEdgeBuilder {
    id: usize,
    face_id: usize,
    start_id: Option<usize>,
    twin_id: Option<usize>,
    prev_id: Option<usize>,
    next_id: Option<usize>,
}

impl HalfEdgeBuilder {

    fn into_half_edge(self) -> HalfEdge {
	HalfEdge {
	    id: self.id,
	    face_id: self.face_id,
	    start_id: self.start_id.expect("all half edges should have a start vertex"),
	    twin_id: self.twin_id,
	    prev_id: self.prev_id.expect("all half edges should have a previous half edge"),
	    next_id: self.next_id.expect("all half edges should have a next half edge"),
	}
    }
    
}

#[derive(Debug, PartialEq)]
pub struct Face {
    id: usize,
    x: f64,
    y: f64,
    start_id: usize,
}

struct FaceBuilder {
    id: usize,
    x: f64,
    y: f64,
    half_edge_id: Option<usize>,
    start_id: Option<usize>,
    end_id: Option<usize>,
}

impl FaceBuilder {

    fn into_face(self) -> Face {
	Face {
	    id: self.id,
	    x: self.x,
	    y: self.y,
	    start_id: self.half_edge_id.expect("face should have at least one half edge"),
	}
    }

    fn has_complete_bounds(&self) -> bool {
	self.start_id.is_none() && self.end_id.is_none()
    } 
    
}

#[derive(Debug, PartialEq)]
pub struct Diagram {
    width: f64,
    height: f64,
    vertices: Vec<Vertex>,
    half_edges: Vec<HalfEdge>,
    faces: Vec<Face>,
}

impl Diagram {

    fn fmt_face(&self, face_id: usize, f: &mut Formatter<'_>) -> Result<(), FormatError> {
	let face = &self.faces[face_id];
	write!(f, "Face {}\nsite: ({}, {})\nbounds:\n", face_id, face.x, face.y)?;
	let start_id = face.start_id;
	self.fmt_half_edge(start_id, f)?;
	let mut cur_id = start_id;
	loop {
	    let next_id = self.half_edges[cur_id].next_id;
	    if next_id == start_id {
		break;
	    } else {
		cur_id = next_id;
		self.fmt_half_edge(cur_id, f)?;
	    }
	}
	write!(f, "\n")?;
	Ok(())
    }

    fn fmt_half_edge(&self, half_edge_id: usize, f: &mut Formatter<'_>) -> Result<(), FormatError> {
	let half_edge = &self.half_edges[half_edge_id];
	write!(f, "Half edge {}\n", half_edge_id)?;
	self.fmt_vertex(half_edge.start_id, "start", f)?;
	self.fmt_vertex(self.half_edges[half_edge.next_id].start_id, "end", f)?;
	match &half_edge.twin_id {
	    Some(twin_id) => write!(f, "twin: {}", *twin_id)?,
	    None => write!(f, "twin: none")?
	}
	Ok(())
    }

    fn fmt_vertex(&self, vertex_id: usize, label: &str, f: &mut Formatter<'_>) -> Result<(), FormatError> {
	let vertex = &self.vertices[vertex_id];
	write!(f, "{}: ({}, {})\n", label, vertex.x, vertex.y)?;
	Ok(())
    }

    pub fn create_triangles(&self) -> (Vec<f32>, Vec<u32>) {
	let scale = self.width / 2.0;
	let mut vertices = Vec::with_capacity((self.vertices.len() + self.faces.len()) * 3);
	
	for f in self.faces.iter() {
	    vertices.push((f.x / scale - 1.0) as f32);
	    vertices.push((f.y / scale - 1.0) as f32);
	    vertices.push(0.0f32);
	}
	
	for v in self.vertices.iter() {
	    vertices.push((v.x / scale - 1.0) as f32);
	    vertices.push((v.y / scale - 1.0) as f32);
	    vertices.push(0.0f32);
	}

	let offset = self.faces.len();
	let mut indices = Vec::new();
	
	for id in 0..self.faces.len() {
	    let f = &self.faces[id];
	    let mut cur_id = f.start_id;
	    loop {
		let next_id = self.half_edges[cur_id].next_id;
		if next_id == f.start_id {
		    break;
		} else {
		    indices.push(id as u32);
		    indices.push((offset + self.half_edges[cur_id].start_id) as u32);
		    indices.push((offset + self.half_edges[next_id].start_id) as u32);
		    cur_id = next_id;
		}
	    }
	    indices.push(id as u32);
	    indices.push((offset + self.half_edges[cur_id].start_id) as u32);
	    indices.push((offset + self.half_edges[f.start_id].start_id) as u32);
	}
	(vertices, indices)
    }
    
}

impl Display for Diagram {

    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), FormatError> {
	for id in 0..self.faces.len() {
	    self.fmt_face(id, f)?;
	}
	Ok(())
    }
    
}

enum EventKind {
    AddArc {
	face_id: usize,
    },
    RemoveArc {
	arc_id: usize,
    },
}

struct Event {
    id: usize,
    priority: f64,
    kind: EventKind,
}

impl PartialEq for Event {

    fn eq(&self, other: &Self) -> bool {
	self.id.eq(&other.id)
    }
    
}

impl Eq for Event {}

impl Ord for Event {

    fn cmp(&self, other: &Self) -> Ordering {
	match self.priority.partial_cmp(&other.priority) {
	    Some(Ordering::Equal) | None => self.id.cmp(&other.id),
	    Some(Ordering::Less) => Ordering::Less,
	    Some(Ordering::Greater) => Ordering::Greater,
	}
    }
    
}

impl PartialOrd for Event {

    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
	Some(self.cmp(other))
    }
    
}


#[derive(PartialEq)]
enum NodeId {
    Edge(usize),
    Arc(usize),
}


struct Edge {
    id: usize,
    half_edge_id: usize,

    parent_id: Option<usize>,
    left_child: NodeId,
    right_child: NodeId,
}

struct Arc {
    id: usize,
    face_id: usize,
    remove_event_id: Option<usize>,

    parent_id: Option<usize>,
}

struct Builder {
    width: f64,
    height: f64,
    vertices: Vec<Vertex>,
    half_edges: Vec<HalfEdgeBuilder>,
    faces: Vec<FaceBuilder>,
    events: BinaryHeap<Event>,
    edges: Vec<Edge>,
    arcs: Vec<Arc>,
    root: Option<NodeId>,
    calculator: IntersectionCalculator,
}

impl Builder {

    fn new(width: f64, height: f64) -> Builder {
	Builder{
	    width,
	    height,
	    vertices: vec![],
	    half_edges: vec![],
	    faces: vec![],
	    events: BinaryHeap::new(),
	    edges: vec![],
	    arcs: vec![],
	    root: None,
	    calculator: IntersectionCalculator::new(),
	}
    }

    fn add_site(&mut self, x: f64, y: f64) {
	let id = self.faces.len();
	self.faces.push(FaceBuilder {
	    id,
	    x,
	    y,
	    half_edge_id: None,
	    start_id: None,
	    end_id: None,
	});
    }

    fn create_vertex(&mut self, x: f64, y: f64) -> usize {
	let id = self.vertices.len();
	self.vertices.push(Vertex{
	    id,
	    x,
	    y,
	});
	id
    }

    fn create_half_edge(&mut self, face_id: usize, start_id: Option<usize>) -> usize {
	let id = self.half_edges.len();
	self.half_edges.push(HalfEdgeBuilder {
	    id,
	    face_id,
	    start_id,
	    twin_id: None,
	    prev_id: None,
	    next_id: None,
	});
	id
    }

    fn create_half_edge_pair(&mut self, face_id: usize, twin_face_id: usize) -> (usize, usize) {
	let id = self.create_half_edge(face_id, None);
	let twin_id = self.create_half_edge(twin_face_id, None);
	self.half_edges[id].twin_id = Some(twin_id);
	self.half_edges[twin_id].twin_id = Some(id);
	(id, twin_id)
    }

    fn connect_half_edges(&mut self, first_id: usize, second_id: usize) {
	let first = &self.half_edges[first_id];
	let second = &self.half_edges[second_id];
	if first.face_id != second.face_id {
	    panic!("half edges have different face");
	}
	if first.next_id.is_some() {
	    panic!("first half edge is already connected");
	}
	if second.prev_id.is_some() {
	    panic!("second half edge is already connected");
	}
	self.half_edges[first_id].next_id = Some(second_id);
	self.half_edges[second_id].prev_id = Some(first_id);
    }
    
    fn create_arc(&mut self, face_id: usize) -> usize {
	let id = self.arcs.len();
	self.arcs.push(Arc {
	    id,
	    face_id,
	    remove_event_id: None,
	    parent_id: None,
	});
	id
    }

    fn create_edge(&mut self, half_edge_id: usize, left_child: NodeId, right_child: NodeId) -> usize {
	let id = self.edges.len();
	self.edges.push(Edge{
	    id,
	    half_edge_id,
	    parent_id: None,
	    left_child,
	    right_child,
	});
	id
    }

    fn find_prev_arc_id(&self, edge_id: usize) -> usize {
	let mut node_id = &self.edges[edge_id].left_child;
	loop {
	    match node_id {
		NodeId::Arc(arc_id) => break *arc_id,
		NodeId::Edge(child_edge_id) => {
		    node_id = &self.edges[*child_edge_id].right_child;
		}
	    }
	}
    }

    fn find_next_arc_id(&self, edge_id: usize) -> usize {
	let mut node_id = &self.edges[edge_id].right_child;
	loop {
	    match node_id {
		NodeId::Arc(arc_id) => break *arc_id,
		NodeId::Edge(child_edge_id) => {
		    node_id = &self.edges[*child_edge_id].left_child;
		}
	    }
	}
    }

    fn find_first_edge_id(&self) -> Option<usize> {
	match &self.root {
	    Some(root) => {
		let mut result = None;
		let mut node = root;
		while let NodeId::Edge(edge_id) = node {
		    result = Some(*edge_id);
		    node = &self.edges[*edge_id].left_child;
		}
		result
	    },
	    None => None,
	}
    }

    fn find_prev_edge_id(&self, arc_id: usize) -> Option<usize> {
	match &self.arcs[arc_id].parent_id {
	    Some(parent_id) => {
		if self.edges[*parent_id].right_child == NodeId::Arc(arc_id) {
		    Some(*parent_id)
		} else {
		    let mut edge_id = parent_id;
		    loop {
			match &self.edges[*edge_id].parent_id {
			    Some(parent_id) => {
				if self.edges[*parent_id].right_child == NodeId::Edge(*edge_id) {
				    break Some(*parent_id);
				}
				edge_id = parent_id;
			    },
			    None => {
				break None;
			    }
			}
		    }
		}
	    },
	    None => {
		None
	    }
	}
    }
    
    fn find_next_edge_id(&self, arc_id: usize) -> Option<usize> {
	match &self.arcs[arc_id].parent_id {
	    Some(parent_id) => {
		if self.edges[*parent_id].left_child == NodeId::Arc(arc_id) {
		    Some(*parent_id)
		} else {
		    let mut edge_id = parent_id;
		    loop {
			match &self.edges[*edge_id].parent_id {
			    Some(parent_id) => {
				if self.edges[*parent_id].left_child == NodeId::Edge(*edge_id) {
				    break Some(*parent_id);
				}
				edge_id = parent_id;
			    },
			    None => {
				break None;
			    }
			}
		    }
		}
	    },
	    None => {
		None
	    }
	}
    }
     
    fn find_split_arc(&self, start_node_id: &NodeId, new_face_id: usize) -> usize {
	let mut node_id = start_node_id;
	let face = &self.faces[new_face_id];
	loop {
	    match node_id {
		NodeId::Arc(arc_id) => break *arc_id,
		NodeId::Edge(edge_id) => {
		    let left_arc_id = self.find_prev_arc_id(*edge_id);
		    let right_arc_id = self.find_next_arc_id(*edge_id);
		    let left_face = &self.faces[self.arcs[left_arc_id].face_id];
		    let right_face = &self.faces[self.arcs[right_arc_id].face_id];
		    match intersect_parabolas_from_foci(
			&Vector2F64::from_values(left_face.x, left_face.y),
			&Vector2F64::from_values(right_face.x, right_face.y),
			face.y
		    ){
			ParabolaIntersection::Two(i1, i2) => {
			    //note i1.x < i2.x
			    let ix = if left_face.y < right_face.y {
				i1.get_x()
			    } else {
				i2.get_x()
			    };
			    node_id = if face.x < ix {
				&self.edges[*edge_id].left_child
			    } else {
				&self.edges[*edge_id].right_child
			    };
			},
			_ => panic!("expected two intersections")
		    }
		}
	    }
	}
    }
    
    fn add_arc(&mut self, face_id: usize) {
	match &self.root {
	    None => {
		self.root = Some(NodeId::Arc(self.create_arc(face_id)));
	    },
	    Some(node_id) => {
		let split_arc_id = self.find_split_arc(node_id, face_id);
		let split_face_id = self.arcs[split_arc_id].face_id;

		let (half_edge_id, twin_id) = self.create_half_edge_pair(face_id, split_face_id);
		self.faces[face_id].half_edge_id.get_or_insert(half_edge_id);
		self.faces[split_face_id].half_edge_id.get_or_insert(twin_id);
		
		let parent_id = self.arcs[split_arc_id].parent_id;
		let new_arc_id = self.create_arc(face_id);
		let cloned_arc_id = self.create_arc(self.arcs[split_arc_id].face_id);
		let right_edge_id = self.create_edge(half_edge_id,
						     NodeId::Arc(new_arc_id),
						     NodeId::Arc(cloned_arc_id)
		);
		self.arcs[new_arc_id].parent_id = Some(right_edge_id);
		self.arcs[cloned_arc_id].parent_id = Some(right_edge_id);
		let left_edge_id = self.create_edge(twin_id,
						    NodeId::Arc(split_arc_id),
						    NodeId::Edge(right_edge_id)
		);
		self.arcs[split_arc_id].parent_id = Some(left_edge_id);
		self.edges[right_edge_id].parent_id = Some(left_edge_id);
		self.edges[left_edge_id].parent_id = parent_id;
		if let None = parent_id {
		    self.root = Some(NodeId::Edge(left_edge_id));
		}
		let scan_line_y = self.faces[face_id].y;
		self.update_remove_event(split_arc_id, scan_line_y);
		self.update_remove_event(cloned_arc_id, scan_line_y);
	    }
	}
    }

    fn remove_arc(&mut self, arc_id: usize) {
	let left_edge_id = self.find_prev_edge_id(arc_id).unwrap();
	let right_edge_id = self.find_next_edge_id(arc_id).unwrap();
	let left_arc_id = self.find_prev_arc_id(left_edge_id);
	let left_face_id = self.arcs[left_arc_id].face_id;
	let right_arc_id = self.find_next_arc_id(right_edge_id);
	let right_face_id = self.arcs[right_arc_id].face_id;
	let focus = self.get_arc_focus(arc_id);
	let left_focus = self.get_arc_focus(left_arc_id);
	let right_focus = self.get_arc_focus(right_arc_id);
	let (center, _) = self.calculator.circle_through_points(
	    &left_focus, &focus, &right_focus).unwrap();

	let intersection_id = self.create_vertex(center.get_x(), center.get_y());
	let left_in_id = self.edges[left_edge_id].half_edge_id;
	let left_out_id = self.half_edges[left_in_id].twin_id.unwrap();
	let right_out_id = self.edges[right_edge_id].half_edge_id;
	let right_in_id = self.half_edges[right_out_id].twin_id.unwrap();
	let (down_out_id, down_in_id) = self.create_half_edge_pair(left_face_id, right_face_id);

	self.half_edges[down_out_id].start_id = Some(intersection_id);
	self.connect_half_edges(left_in_id, down_out_id);

	self.half_edges[left_out_id].start_id = Some(intersection_id);
	self.connect_half_edges(right_in_id, left_out_id);

	self.half_edges[right_out_id].start_id = Some(intersection_id);
	self.connect_half_edges(down_in_id, right_out_id);

	
	if self.edges[left_edge_id].right_child == NodeId::Arc(arc_id) {
	    // arc is the right child of its parent
	    let grand_parent_id = self.edges[left_edge_id].parent_id.unwrap();
	    self.replace_child(grand_parent_id, &NodeId::Edge(left_edge_id), NodeId::Arc(left_arc_id));
	    self.edges[right_edge_id].half_edge_id = down_out_id;
	} else {
	    // arc is the left child of its parent
	    let grand_parent_id = self.edges[right_edge_id].parent_id.unwrap();
	    self.replace_child(grand_parent_id, &NodeId::Edge(right_edge_id), NodeId::Arc(right_arc_id));
	    self.edges[left_edge_id].half_edge_id = down_out_id;
	}
    }

    fn replace_child(&mut self, parent_edge_id: usize, old_child: &NodeId, new_child: NodeId) {
	match &new_child {
	    NodeId::Arc(arc_id) => {
		self.arcs[*arc_id].parent_id = Some(parent_edge_id);
	    },
	    NodeId::Edge(edge_id) => {
		self.edges[*edge_id].parent_id = Some(parent_edge_id);
	    }
	}
	if &self.edges[parent_edge_id].left_child == old_child {
	    self.edges[parent_edge_id].left_child = new_child;
	} else if &self.edges[parent_edge_id].right_child == old_child {
	    self.edges[parent_edge_id].right_child = new_child;
	} else {
	    panic!("node to be replaces is not a child of the parent");
	}
    }

    fn check_remove_arc(&mut self, arc_id: usize, event_id: usize) {
	if self.arcs[arc_id].remove_event_id == Some(event_id) {
	    self.remove_arc(arc_id);
	}
    }
    
    fn get_arc_focus(&self, arc_id: usize) -> Vector2F64 {
	let face = &self.faces[self.arcs[arc_id].face_id];
	Vector2F64::from_values(face.x, face.y)
    }
    
    fn update_remove_event(&mut self, arc_id: usize, scan_line_y: f64) {
	self.arcs[arc_id].remove_event_id = None;
	if let Some(left_edge_id) = self.find_prev_edge_id(arc_id) {
	    if let Some(right_edge_id) = self.find_next_edge_id(arc_id) {
		let left_arc_id = self.find_prev_arc_id(left_edge_id);
		let right_arc_id = self.find_next_arc_id(right_edge_id);
		let focus = self.get_arc_focus(arc_id);
		let left_focus = self.get_arc_focus(left_arc_id);
		let right_focus = self.get_arc_focus(right_arc_id);
		if let Some((center, radius)) = self.calculator.circle_through_points(
		    &left_focus,
		    &focus,
		    &right_focus
		){
		    let priority = center.get_y() + radius;
		    if priority >= scan_line_y && center.get_y() <= self.height && center.get_y() >= focus.get_y(){
			let id = self.events.len();
			self.events.push(Event{
			    id,
			    priority,
			    kind: EventKind::RemoveArc {
				arc_id
			    }
			});
			self.arcs[arc_id].remove_event_id = Some(arc_id);
		    }
		}
	    }
	}
    }
    
    fn create_events(&mut self) {
	for face in self.faces.iter() {
	    let id = self.events.len();
	    self.events.push(Event{
		id,
		priority: face.x,
		kind: EventKind::AddArc{
		    face_id: face.id,
		},
	    });
	}
    }

    fn handle_events(&mut self) {
	while let Some(event) = self.events.pop() {
	    match event.kind {
		EventKind::AddArc{face_id} => {
		    self.add_arc(face_id);
		},
		EventKind::RemoveArc{arc_id} => {
		    self.check_remove_arc(arc_id, event.id);
		}
	    }
	}
    }

    fn complete_edges(&mut self) {
	let bounds = BoundingBox::new(0.0, self.width, 0.0, self.height);
	
	let mut next = self.find_first_edge_id();
	while let Some(edge_id) = next {
	    self.complete_edge(&bounds, edge_id);
	    next = self.find_next_edge_id(self.find_next_arc_id(edge_id));
	}
    }

    fn complete_edge(&mut self, bounds: &BoundingBox, edge_id: usize) {
	let half_edge_id = self.edges[edge_id].half_edge_id;
	let face_id = self.half_edges[half_edge_id].face_id;
	let twin_id = self.half_edges[half_edge_id].twin_id.expect("half edge should have a twin");
	let twin_face_id = self.half_edges[twin_id].face_id;
	if let None = self.half_edges[half_edge_id].start_id {
	    self.half_edges[half_edge_id].start_id = Some(self.calc_half_edge_start(bounds, face_id, twin_face_id));
	}
	self.faces[face_id].end_id = Some(half_edge_id);
	self.faces[twin_face_id].start_id = Some(twin_id);
    }
    
    fn calc_half_edge_start(&mut self, bounds: &BoundingBox, face_id: usize, twin_face_id: usize) -> usize {
	let face = &self.faces[face_id];
	let twin_face = &self.faces[twin_face_id];
	let pos = Vector2F64::from_values((face.x + twin_face.x) / 2.0, (face.y + twin_face.y) / 2.0);
	let sub = Vector2F64::from_values(twin_face.x - face.x, twin_face.y - face.y);
	let mut dir = Vector2F64::from_values(face.y - twin_face.y, twin_face.x - face.x);
	if !is_clockwise(&sub, &dir) {
	    dir = Vector2F64::from_values(-dir.get_x(), -dir.get_y());
	}
	match self.calculator.ray_with_bounding_box(&pos, &dir, bounds) {
	    Some(v) => {
		self.create_vertex(v.get_x(), v.get_y())
	    },
	    None => panic!("ray should intersect with bounds")
	}
    }

    fn bound(&mut self) {
	match self.root {
	    None => {},
	    Some(NodeId::Arc(arc_id)) => {
		let face_id = self.arcs[arc_id].face_id;
		let top_left_id = self.create_vertex(0.0, 0.0);
		let top_right_id = self.create_vertex(self.width, 0.0);
		let bottom_right_id = self.create_vertex(self.width, self.height);
		let bottom_left_id = self.create_vertex(0.0, self.height);
		let top_id = self.create_half_edge(face_id, Some(top_left_id));
		let right_id = self.create_half_edge(face_id, Some(top_right_id));
		let bottom_id = self.create_half_edge(face_id, Some(bottom_right_id));
		let left_id = self.create_half_edge(face_id, Some(bottom_left_id));
		self.connect_half_edges(top_id, right_id);
		self.connect_half_edges(right_id, bottom_id);
		self.connect_half_edges(bottom_id, left_id);
		self.connect_half_edges(left_id, top_id);
		self.faces[0].half_edge_id = Some(top_id);
	    },
	    Some(NodeId::Edge(_)) => {
		for id in 0..self.faces.len() {
		    if !self.faces[id].has_complete_bounds() {
			self.complete_face_bounds(id);
		    }
		}
	    },
	}
    }

    fn complete_face_bounds(&mut self, face_id: usize) {
	let mut cur_half_edge_id = self.faces[face_id].start_id.expect("face bounds must have start half edge");
	let mut cur_vertex_id = self.half_edges[self.half_edges[cur_half_edge_id].twin_id.unwrap()]
	    .start_id.unwrap();
	let mut cur_x = self.vertices[cur_vertex_id].x;
	let mut cur_y = self.vertices[cur_vertex_id].y;
	let end_half_edge_id = self.faces[face_id].end_id.expect("face bounds must have end half_edge");
	let end_vertex_id = self.half_edges[cur_half_edge_id].start_id.unwrap();
	let end_x = self.vertices[end_vertex_id].x;
	let end_y = self.vertices[end_vertex_id].y;
	loop {
	    let (next_x, next_y) = if cur_y == 0.0 && cur_x != self.width {
		if end_y == 0.0 && end_x >= cur_x {
		    break;
		}
		(self.width, 0.0)
	    } else if cur_x == self.width && cur_y != self.height{
		if end_x == self.width && end_y >= cur_y {
		    break;
		}
		(self.width, self.height) 
	    } else if cur_y == self.height && cur_x != 0.0 {
		if end_y == self.height && end_x <= cur_x {
		    break;
		}
		(0.0, self.height)
	    } else if cur_x == 0.0 && cur_y != 0.0 {
		if end_x == 0.0 && end_y <= cur_y {
		    break;
		}
		(0.0, 0.0)
	    } else {
		panic!("end vertex is not on bounding box: ({:?}, {:?})", cur_x, cur_y);
	    };
	    let next_half_edge_id = self.create_half_edge(face_id, Some(cur_vertex_id));
	    self.connect_half_edges(cur_half_edge_id, next_half_edge_id);
	    
	    cur_vertex_id = self.create_vertex(next_x, next_y);
	    cur_half_edge_id = next_half_edge_id;
	    cur_x = next_x;
	    cur_y = next_y;
	}
	let next_half_edge_id = self.create_half_edge(face_id, Some(cur_vertex_id));
	self.connect_half_edges(cur_half_edge_id, next_half_edge_id);
	self.connect_half_edges(next_half_edge_id, end_half_edge_id);
    }

    fn build(&mut self) -> Diagram {
	self.create_events();
	self.handle_events();
	self.complete_edges();
	self.bound();
	let diagram = Diagram {
	    width: self.width,
	    height: self.height,
	    vertices: std::mem::take(&mut self.vertices),
	    half_edges: self.half_edges.drain(..).map(|he| he.into_half_edge()).collect(),
	    faces: self.faces.drain(..).map(|f| f.into_face()).collect(),
	};
	self.events.clear();
	self.edges.clear();
	self.arcs.clear();
	self.root = None;
	diagram
    }

}

pub fn generate() -> Diagram {
    let mut builder = Builder::new(1000.0, 1000.0);
    builder.add_site(100.0, 100.0);
    builder.add_site(900.0, 900.0); 
    builder.build()
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn test_generate_empty() {
	let mut builder = Builder::new(1000.0, 1000.0);
	let diagram = builder.build();
	assert_eq!(Diagram {
	    width: 1000.0,
	    height: 1000.0,
	    vertices: vec![],
	    half_edges: vec![],
	    faces: vec![],
	}, diagram);
    }

        #[test]
    fn test_generate_unary() {
	let mut builder = Builder::new(1000.0, 1000.0);
	builder.add_site(500.0, 500.0);
	let diagram = builder.build();
	assert_eq!(Diagram {
	    width: 1000.0,
	    height: 1000.0,
	    vertices: vec![
		Vertex{id: 0, x: 0.0, y: 0.0},
		Vertex{id: 1, x: 1000.0, y: 0.0},
		Vertex{id: 2, x: 1000.0, y: 1000.0},
		Vertex{id: 3, x: 0.0, y: 1000.0},
	    ],
	    half_edges: vec![
		HalfEdge{
		    id: 0, face_id: 0, start_id: 0, prev_id: 3, next_id: 1, twin_id: None
		},
		HalfEdge{
		    id: 1, face_id: 0, start_id: 1, prev_id: 0, next_id: 2, twin_id: None
		},
		HalfEdge{
		    id: 2, face_id: 0, start_id: 2, prev_id: 1, next_id: 3, twin_id: None
		},
		HalfEdge{
		    id: 3, face_id: 0, start_id: 3, prev_id: 2, next_id: 0, twin_id: None
		},
	    ],
	    faces: vec![
		Face {
		    id: 0,
		    x: 500.0,
		    y: 500.0,
		    start_id: 0,
		},
	    ],
	}, diagram);
    }

    #[test]
    fn test_generate_binary() {
	let mut builder = Builder::new(1000.0, 1000.0);
	builder.add_site(100.0, 100.0);
	builder.add_site(900.0, 900.0);
	let diagram = builder.build();
	assert_eq!(Diagram {
	    width: 1000.0,
	    height: 1000.0,
	    vertices: vec![
		Vertex{id: 0, x: 0.0, y: 1000.0},
		Vertex{id: 1, x: 1000.0, y: 0.0},
		Vertex{id: 2, x: 0.0, y: 0.0},
		Vertex{id: 3, x: 1000.0, y: 1000.0},
	    ],
	    half_edges: vec![
		HalfEdge {
		    id: 0, face_id: 0, start_id: 1, twin_id: Some(1), prev_id: 3, next_id: 2
		},
		HalfEdge {
		    id: 1, face_id: 1, start_id: 0, twin_id: Some(0), prev_id: 5, next_id: 4
		},
		HalfEdge {
		    id: 2, face_id: 0, start_id: 0, twin_id: None, prev_id: 0, next_id: 3
		},
		HalfEdge {
		    id: 3, face_id: 0, start_id: 2, twin_id: None, prev_id: 2, next_id: 0
		},
		HalfEdge {
		    id: 4, face_id: 1, start_id: 1, twin_id: None, prev_id: 1, next_id: 5
		},
		HalfEdge {
		    id: 5, face_id: 1, start_id: 3, twin_id: None, prev_id: 4, next_id: 1
		}
	    ],
	    faces: vec![
		Face {
		    id: 0,
		    x: 100.0,
		    y: 100.0,
		    start_id: 0,
		},
		Face {
		    id: 1,
		    x: 900.0,
		    y: 900.0,
		    start_id: 1,
		},
	    ],
	}, diagram);
    }

    #[test]
     fn create_triangles() {
	 let mut builder = Builder::new(1000.0, 1000.0);
	 builder.add_site(100.0, 100.0);
	 builder.add_site(900.0, 900.0);
	 let (vertices, indices) = builder.build().create_triangles();
	 let expected_vertices: Vec<f32> = vec![
	     -0.8, -0.8, 0.0, 0.8, 0.8, 0.0, -1.0, 1.0, 0.0, 1.0, -1.0, 0.0, -1.0, -1.0, 0.0, 1.0, 1.0, 0.0
	 ];
	 assert_eq!(expected_vertices, vertices);

	 let expected_indices: Vec<u32> = vec![
	     0, 3, 2, 0, 2, 4, 0, 4, 3, 1, 2, 3, 1, 3, 5, 1, 5, 2
	 ];
	 assert_eq!(expected_indices, indices);
     }
}
