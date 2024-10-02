use crate::vcell::VCellRaw;
use bevy::prelude::Vec3;
use voro_rs::prelude::{Container0, Container2, ContainerLoop, LoopAll, VoroCell, VoroCellSgl};
fn make_voronoi_container(
    size: Vec3,
    num_points: usize,
) -> voro_rs::container::ContainerStd<'static> {
    let xyz_min = [0., 0., 0.];
    let xyz_max = [size.x as f64, size.y as f64, size.z as f64];
    let rel = size / (size.x + size.y + size.z) * 2.0;

    let num_points_cbrt = (num_points as f32).cbrt().ceil();

    let grids = [
        (num_points_cbrt * rel.x) as i32 + 1,
        (num_points_cbrt * rel.y) as i32 + 1,
        (num_points_cbrt * rel.z) as i32 + 1,
    ];

    let container1 =
        voro_rs::container::ContainerStd::new(xyz_min, xyz_max, grids, [false, false, false]);
    container1
}

fn f64_3_to_vec3(slice: [f64; 3]) -> Vec3 {
    Vec3::new(slice[0] as f32, slice[1] as f32, slice[2] as f32)
}

fn f64_vv_to_vec3_vv(points: Vec<f64>) -> Vec<Vec3> {
    assert!(points.len() % 3 == 0);
    let mut vs = vec![];
    for i in 0..(points.len() / 3) {
        vs.push(Vec3::new(
            points[i * 3] as f32,
            points[i * 3 + 1] as f32,
            points[i * 3 + 2] as f32,
        ));
    }
    vs
}

pub fn impl_voro_rs(size: Vec3, points: Vec<Vec3>) -> Vec<VCellRaw> {
    let mut container1 = make_voronoi_container(size, points.len());

    for (i, point) in points.iter().enumerate() {
        container1.put(
            i as i32,
            [point.x as f64, point.y as f64, point.z as f64],
            1.0,
        );
    }
    let mut cl = LoopAll::of_container_std(&mut container1);
    cl.start();

    let mut v = vec![];

    loop {
        let cell: Option<VoroCellSgl> = container1.compute_cell(&mut cl);
        assert!(cell.is_some());
        let mut cell = cell.unwrap();
        let particle_id = cl.particle_id();
        let particle_position = f64_3_to_vec3(cl.position());
        let particle_centroid = f64_3_to_vec3(cell.centroid());
        let vs = f64_vv_to_vec3_vv(cell.vertices_local());
        let face_normals = f64_vv_to_vec3_vv(cell.normals());

        let face_orders = cell.face_orders();
        let face_vert = cell.face_vertices();
        let mut faces = vec![];
        let mut idx = 0;
        for face_ord in face_orders {
            let mut face = vec![];
            for _z in 0..(face_ord + 1) {
                if _z > 0 {
                    face.push(face_vert[idx]);
                }
                idx += 1;
            }
            faces.push(face);
        }
        assert!(idx == face_vert.len());

        let new_cell = VCellRaw {
            volume: cell.volume(),
            id: particle_id,
            vs,
            faces,
            face_normals,
            position: particle_position,
            centroid: particle_centroid,
        };
        v.push(new_cell);

        if !cl.inc() {
            break;
        }
    }
    v
}
