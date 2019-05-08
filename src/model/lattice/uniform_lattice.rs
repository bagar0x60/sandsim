use super::utils::{UniformTiling, FigureGeometricInfo, FULL_CIRCLE, Constructor, continue_tiling_by_translation};
use super::Lattice;

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

            for (pos, node_figures) in &mut tiling_builder.vertices_info {
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

fn tiling_1(cuboid_hull: &Cuboid) -> SandPileModel {
    // 1-uniform
    // 3.3.3.3.6
    // p6, 632

    let mut constructor = Constructor::new(1.0, 6);

    for i in 0..=5 {
        constructor.add(0, i, 3);
    }

    for i in 1..=6 {
        constructor.add(i, 1, 3);
        constructor.add(i, 2, 3);
    }

    let v1 = constructor.get_vector(1, 13);
    let v2 = constructor.get_vector(3, 17);

    continue_tiling_by_translation(&mut constructor.tiling, (v1, v2), cuboid_hull);
    constructor.tiling.build()
}


fn tiling_2(cuboid_hull: &Cuboid) -> SandPileModel {
    // 2-uniform
    // [3^6; 3^2.4.3.4]
    // p6m, *632

    let mut constructor = Constructor::new(1.0, 3);

    for i in 0..5 {
        constructor.add(i, 2, 3);
    }
    for i in 0..6 {
        constructor.add(i, 1, 4);
    }
    for i in 6..12 {
        constructor.add(i, 3, 3);
    }

    let v1 = constructor.get_vector(7, 10);
    let v2 = constructor.get_vector(6, 9);

    continue_tiling_by_translation(&mut constructor.tiling, (v1, v2), cuboid_hull);
    constructor.tiling.build()
}


fn tiling_3(cuboid_hull: &Cuboid) -> SandPileModel {
    // 2-uniform
    // [3^6; 3^3.4^2]
    // pmm, *2222

    let mut constructor = Constructor::new(1.0, 4);

    constructor.add(0, 1, 3);
    constructor.add(0, 3, 3);
    constructor.add(2, 2, 3);
    constructor.add(1, 2, 3);
    constructor.add(1, 1, 3);
    constructor.add(5, 2, 3);

    let v1 = constructor.get_vector(3, 6);
    let v2 = constructor.get_vector(4, 5);

    continue_tiling_by_translation(&mut constructor.tiling, (v1, v2), cuboid_hull);
    constructor.tiling.build()
}



impl Lattice for KUniformLattice {
    fn get_lattice(&self, cuboid_hull: &Cuboid) -> SandPileModel {
        tiling_1(cuboid_hull)
    }
}
