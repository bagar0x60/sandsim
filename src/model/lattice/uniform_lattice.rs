use super::utils::{UniformTiling, FigureGeometricInfo, FULL_CIRCLE, Constructor, continue_tiling_by_translation};
use super::Lattice;
use super::tilings::*;

use model::region::Cuboid;
use model::sand_graph::{SandGraph, NodeIndex};
use model::SandPileModel;

#[derive(Debug)]
pub struct SemiRegularLattice {
    tiling_code: Vec<usize>,
}

pub struct KUniformLattice {

}

impl SemiRegularLattice {
    pub fn new(tiling_code: Vec<usize>) -> Self {
        let angle: usize = tiling_code.iter().map(|n| FULL_CIRCLE / 2 - FULL_CIRCLE / n).sum();
        assert_eq!(angle % FULL_CIRCLE, 0,
                   "Incorrect tiling code {:#?}. Figures don't sum up to {} degrees", tiling_code, FULL_CIRCLE);

        SemiRegularLattice { tiling_code }
    }
}

impl Lattice for SemiRegularLattice {
    fn get_lattice(&self, cuboid_hull: &Cuboid) -> SandPileModel {
        let side_size = 1.0_f32;
        let [x_size, y_size, _] = *cuboid_hull;

        let mut tiling_builder = UniformTiling::new(1.0);

        // add figures from self.tiling_code around [0.0, 0.0, 0.0]
        let mut rotate = 0;
        for sides_count in &self.tiling_code {
            let figure =
                FigureGeometricInfo::new([x_size / 2.0, y_size / 2.0, 0.0], rotate, side_size, *sides_count);

            rotate = (rotate + figure.alpha) % FULL_CIRCLE;
            tiling_builder.add_figure(figure);
        }

        // add all other figures
        loop {
            let mut new_figures: Vec<FigureGeometricInfo> = Vec::new();

            for (pos, node_figures) in &mut tiling_builder.vertices_info.data {
                let [x, y, _] = *pos;
                if 0.0 <= x && x < x_size && 0.0 <= y && y < y_size {
                    let new_figures_partial_info = node_figures.new_figures(&self.tiling_code);

                    if new_figures_partial_info.len() > 0 {
                        new_figures = new_figures_partial_info
                            .iter()
                            .map(|(angle, sides_count)| FigureGeometricInfo::new(*pos, *angle, side_size, *sides_count))
                            .collect();
                        break
                    }
                }
            }

            if new_figures.len() > 0 {
                for figure in new_figures {
                    tiling_builder.add_figure(figure);
                }
            } else {
                break
            }
        }


        tiling_builder.build()
    }
}


impl KUniformLattice {
    pub fn new() -> Self {
        KUniformLattice {}
    }
}



impl Lattice for KUniformLattice {
    fn get_lattice(&self, cuboid_hull: &Cuboid) -> SandPileModel {
        tiling_8(cuboid_hull)
    }
}
