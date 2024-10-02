use bevy::prelude::Vec3;

#[macro_export]
macro_rules! log_time {
    ($msg:expr, $x:expr) => {{
        let t0 = web_time::Instant::now();
        let _result = $x;
        let t1 = web_time::Instant::now();
        let dt = t1 - t0;
        ::log::info!("{}: {:?}", $msg, dt);
        _result
    }};
}

/// random points in cuboid of given size in interval [0, size]
pub fn random_points(size: Vec3, count: u32) -> Vec<Vec3> {
    let mut v = vec![];

    for _i in 0..count {
        let p = Vec3::new(
            rand::random::<f32>() * size.x,
            rand::random::<f32>() * size.y,
            rand::random::<f32>() * size.z,
        );
        v.push(p)
    }
    v
}

pub fn perturbed_grid(size: Vec3, count: u32, pert: f32) -> (Vec<Vec3>, Vec<bool>) {
    use rand::{distributions::Uniform, prelude::*};

    let anchor = Vec3::ZERO;
    let mut generators = vec![];
    let mut rng = thread_rng();
    let distr = Uniform::new(-0.5, 0.5);
    let rel = size / (size.x * size.y * size.z).cbrt();

    let nx = (rel.x * (count as f32).cbrt()).round() as usize;
    let ny = (rel.y * (count as f32).cbrt()).round() as usize;
    let nz = (rel.z * (count as f32).cbrt()).round() as usize;
    let count2 = nx * ny * nz;
    eprintln!("{nx} {ny} {nz} old count: {count}, new_count: {count2}");
    let count_v = Vec3::new(nx as f32, ny as f32, nz as f32);

    const TH: usize = 2;
    let mut mask = vec![];

    for i in 0..nx {
        for j in 0..ny {
            for k in 0..nz {
                if (i >= TH && i < nx - TH) && (j >= TH && j < ny - TH) && (k >= TH && k < nz - TH)
                {
                    continue;
                }
                let is_at_edge =
                    i == 0 || i == nx - 1 || j == 0 || j == ny - 1 || k == 0 || k == nz - 1;

                let pos = Vec3 {
                    x: i as f32 + 0.5 + pert * rng.sample(distr),
                    y: j as f32 + 0.5 + pert * rng.sample(distr),
                    z: k as f32 + 0.5 + pert * rng.sample(distr),
                } * size
                    / count_v
                    + anchor;
                mask.push(is_at_edge);
                generators.push(pos.clamp(anchor, anchor + size));
            }
        }
    }
    eprintln!(
        "final result: {}/{}",
        mask.iter().filter(|b| **b).collect::<Vec<_>>().len(),
        generators.len()
    );

    // for n in 0..count.pow(3) {
    //     let i = n / count.pow(2);
    //     let j = (n % count.pow(2)) / count;
    //     let k = n % count;
    // }

    (generators, mask)
}
