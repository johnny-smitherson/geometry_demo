use crate::log_time;
use crate::vcell::*;
use bevy::prelude::Vec3;
use meshless_voronoi::{Voronoi, VoronoiIntegrator};
use std::convert::TryInto;

pub fn impl_meshless_voro(size: Vec3, points: Vec<Vec3>) -> Vec<VCellRaw> {
    log::info!("voronoi demo");

    let anchor = Vec3::splat(0.);
    let points_d = points.iter().map(|x| x.as_dvec3()).collect::<Vec<_>>();

    let _voronoi_integrator = log_time!(
        "VoronoiIntegrator::build",
        VoronoiIntegrator::build(
            &points_d,
            None,
            anchor.as_dvec3(),
            size.as_dvec3(),
            3.try_into().unwrap(),
            false,
        )
        .with_faces()
    );

    let _voronoi2: Voronoi = log_time!(
        "Voronoi::from(&_voronoi_integrator)",
        (&_voronoi_integrator).into()
    );

    log_time!("extract face vertex", {
        _voronoi_integrator
            .cells_iter()
            .zip(_voronoi2.cells().iter())
            .enumerate()
            .map(|(cell_id, (convex_cell, v_cell))| {
                assert!(cell_id == convex_cell.idx);

                assert!(convex_cell.loc == v_cell.loc());
                // let convex_cell = convex_cell.clone().with_faces();

                let vs = convex_cell
                    .vertices
                    .iter()
                    .map(|v| (v.loc - convex_cell.loc).as_vec3())
                    .collect();
                let face_count = v_cell.face_count();

                let v_faces = (0..face_count)
                    .map(|face_idx| {
                        convex_cell
                            .face_vertices(face_idx)
                            .iter()
                            .map(|f| *f as i32)
                            .collect()
                    })
                    .collect();
                let face_norm = v_cell
                    .faces(&_voronoi2)
                    .map(|f| f.normal().as_vec3())
                    .collect();

                // for vx in cell.get_face_vertices(face_idx)

                let cell_position = v_cell.loc().as_vec3();
                VCellRaw {
                    id: cell_id as i32,
                    vs,
                    faces: v_faces,
                    volume: v_cell.volume().abs(),
                    position: cell_position,
                    centroid: v_cell.centroid().as_vec3() - cell_position,
                    face_normals: face_norm,
                }
            })
            .collect()
    })
}
