use model::SandPileModel;
use model::region::Cuboid;
use super::utils::{Constructor, continue_tiling_by_translation};

pub(super) fn tiling_1(cuboid_hull: &Cuboid) -> SandPileModel {
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


pub(super) fn tiling_2(cuboid_hull: &Cuboid) -> SandPileModel {
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


pub(super) fn tiling_3(cuboid_hull: &Cuboid) -> SandPileModel {
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

pub(super) fn tiling_4(cuboid_hull: &Cuboid) -> SandPileModel {
    // 2-uniform
    // [4^4; 3^3.4^2]_1
    // cmm, 2*22

    let mut constructor = Constructor::new(1.0, 4);

    constructor.add(0, 1, 4);
    constructor.add(0, 3, 3);
    constructor.add(1, 2, 3);
    constructor.add(2, 2, 3);
    constructor.add(0, 0, 4);

    let v1 = constructor.get_vector(3, 4);
    let v2 = constructor.get_vector(0, 5);

    continue_tiling_by_translation(&mut constructor.tiling, (v1, v2), cuboid_hull);
    constructor.tiling.build()
}

pub(super) fn tiling_5(cuboid_hull: &Cuboid) -> SandPileModel {
    // 2-uniform
    // [3^3.4^2; 3^2.4.3.4]_2
    // pgg, 22×

    let mut constructor = Constructor::new(1.0, 4);

    constructor.add(0, 0, 4);
    constructor.add(0, 1, 3);
    constructor.add(1, 3, 3);
    constructor.add(1, 2, 3);
    constructor.add(1, 1, 3);
    constructor.add(0, 3, 3);
    constructor.add(0, 2, 3);

    constructor.add(4, 1, 4);
    constructor.add(8, 3, 4);
    constructor.add(9, 3, 3);
    constructor.add(9, 1, 3);
    constructor.add(9, 2, 3);
    constructor.add(8, 2, 3);
    constructor.add(8, 1, 3);


    let v1 = constructor.get_vector(6, 12);
    let v2 = constructor.get_vector(3, 14);

    continue_tiling_by_translation(&mut constructor.tiling, (v1, v2), cuboid_hull);
    constructor.tiling.build()
}

pub(super) fn tiling_6(cuboid_hull: &Cuboid) -> SandPileModel {
    // 4-uniform
    // [3.3.4.12; 3.4.3.12; 3.4.6.4; 4.6.12]
    // cmm

    let mut constructor = Constructor::new(1.0, 12);

    for i in 0..=1 {
        constructor.add(0,  i*6, 4);
        for j in 1..=3 {
            constructor.add(0, i*6 + j, 3);
        }
        constructor.add(0, i*6 + 4, 4);
        constructor.add(0, i*6 + 5, 6);
    }
    constructor.add(4, 1, 4);


    let v1 = constructor.get_vector(6, 12);
    let v2 = constructor.get_vector(11, 13);

    continue_tiling_by_translation(&mut constructor.tiling, (v1, v2), cuboid_hull);
    constructor.tiling.build()
}

pub(super) fn tiling_7(cuboid_hull: &Cuboid) -> SandPileModel {
    // 6-uniform
    // [3^6; 3^4.6; 3^3.4^2; 3^2.4.3.4; 3^2.6^2; 3.4^2.6]
    // p31m

    let mut constructor = Constructor::new(1.0, 6);

    for i in 0..=2 {
        constructor.add(0,  2*i, 6);
    }
    for i in 0..=2 {
        constructor.add(0,  2*i + 1, 3);
    }
    for i in 0..=2 {
        let idx = constructor.add(4 + i,  1, 3);
        for j in 0..=3 {
            constructor.add(idx + j, 2 , 3);
        }
    }
    for i in 1..=3 {
        let idx = constructor.add(i,  4, 3);
        constructor.add(idx,  2, 3);
        let idx = constructor.add(i,  2, 3);
        constructor.add(idx,  1, 3);
    }
    let idx = constructor.add(1,  3, 4);
    assert_eq!(idx, 34);
    constructor.add(idx,  3, 4);
    constructor.add(idx,  1, 4);
    let mut idx = constructor.add(35,  2, 3);

    for i in 0..5 {
        for j in 0..3 {
            constructor.add(idx + j,  2, 4);
        }
        idx = constructor.add(idx + 3,  2, 3);
    }

    constructor.add(47,  1, 6);
    idx = constructor.add(55,  1, 6);
    assert_eq!(idx, 59);

    let v1 = constructor.get_vector(1, 58);
    let v2 = constructor.get_vector(2, 59);

    continue_tiling_by_translation(&mut constructor.tiling, (v1, v2), cuboid_hull);
    constructor.tiling.build()
}