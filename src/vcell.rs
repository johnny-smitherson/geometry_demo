use bevy::math::Vec3;

#[derive(Debug)]
pub struct VCellRaw {
    pub id: i32,
    pub vs: Vec<Vec3>,
    pub faces: Vec<Vec<i32>>,
    pub volume: f64,
    pub position: Vec3,
    pub centroid: Vec3,
    #[allow(unused)]
    pub face_normals: Vec<Vec3>,
}
