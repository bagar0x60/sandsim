use graphics::math;

// Cuboid with vertices in (0, 0, 0) and v1
pub type Cuboid = math::Vec3d<f32>;

pub trait Region {
    fn is_point_inside_region(&self, point: &math::Vec3d<f32>) -> bool;
    fn cuboid_hull(&self) -> Cuboid;
}

pub struct Rectangle {
    hull: Cuboid,
}

pub struct Parallelepiped {
    hull: Cuboid,
}

pub struct Circle {
    radius: f32,
}

impl Rectangle {
    pub fn new(x_size: f32, y_size: f32) -> Rectangle {
        Rectangle {hull: [x_size, y_size, 0.0]}
    }
}

impl Region for Rectangle {
    fn is_point_inside_region(&self, point: &math::Vec3d<f32>) -> bool {
        let [x, y, _] = point;
        let [x_size, y_size, _] = &self.hull;

        (0.0 <= *x && x <= x_size) && (0.0 <= *y && y <= y_size)
    }

    fn cuboid_hull(&self) -> Cuboid {
        self.hull.clone()
    }
}

impl Circle {
    pub fn new(radius: f32) -> Circle {
        Circle {radius}
    }
}

impl Region for Circle {
    fn is_point_inside_region(&self, point: &math::Vec3d<f32>) -> bool {
        let [x, y, _] = point;


        (x - self.radius).powi(2) + (y - self.radius).powi(2) <= self.radius.powi(2)
    }

    fn cuboid_hull(&self) -> Cuboid {
        [2.0*self.radius, 2.0*self.radius, 0.0]
    }
}

impl Parallelepiped {
    pub fn new(x_size: f32, y_size: f32, z_size: f32) -> Self {
        Parallelepiped {hull: [x_size, y_size, z_size]}
    }
}

impl Region for Parallelepiped {
    fn is_point_inside_region(&self, point: &math::Vec3d<f32>) -> bool {
        let [x, y, z] = point;
        let [x_size, y_size, z_size] = &self.hull;

        (0.0 <= *x && x <= x_size) && (0.0 <= *y && y <= y_size) && (0.0 <= *z && z <= z_size)
    }

    fn cuboid_hull(&self) -> Cuboid {
        self.hull.clone()
    }
}
