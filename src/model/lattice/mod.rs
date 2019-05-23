mod regular_lattice;
mod uniform_lattice;
mod tilings;
mod utils;


pub use self::regular_lattice::{SquareLattice, TriangleLattice, HexagonLattice, CubeLattice};
pub use self::uniform_lattice::{SemiRegularLattice, KUniformLattice, TetrahedralOctahedral};

use graphics::math;
use model::SandPileModel;
use super::region::Cuboid;
use super::sand_graph::{SandGraph, NodeIndex};
use super::embedding::{ EmbeddingToR3, Figure };
use std::collections::linked_list::LinkedList;
use vecmath;
use graphics::math::Vec3d;


pub trait Lattice {
    fn get_lattice(&self, cuboid_hull: &Cuboid) -> SandPileModel;
}


















