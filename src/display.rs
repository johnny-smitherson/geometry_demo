use bevy::color::palettes::basic::SILVER;
use bevy::color::palettes::css::{RED, WHITE, YELLOW};
use bevy::core::FrameCount;
use bevy::pbr::wireframe::{WireframeConfig, WireframePlugin};
use bevy::pbr::CascadeShadowConfigBuilder;
use bevy::prelude::Image;
use bevy::prelude::*;
use bevy::render::view::screenshot::ScreenshotManager;
use bevy::window::{PrimaryWindow, WindowResolution};

// use bevy_atmosphere::prelude::*;

use crate::conf::{CONTAINER_HEIGHT, CONTAINER_WIDTH, NPOINTS};
// use crate::debug_texture::create_cube_mesh;
use crate::log_time;
use crate::util::{perturbed_grid, random_points};
use crate::vcell::VCellRaw;

#[derive(Component)]
struct ContainerBoxMesh;

#[derive(Resource)]
struct Decompositions {
    points_meshless: Vec<VCellRaw>,
    points_voro_rs: Vec<VCellRaw>,
    mask: Vec<bool>,
}

impl Decompositions {
    fn new() -> Self {
        Self {
            points_meshless: vec![],
            points_voro_rs: vec![],
            mask: vec![],
        }
    }
}

#[derive(Resource)]
struct DecompositionsPosition(Vec3, Vec3);

#[derive(Component)]
struct MainCamera;

pub fn display_app_root(take_screenshots_and_exit: bool) {
    let mut app = App::new();
    app.add_plugins((
        DefaultPlugins
            .set(ImagePlugin::default_nearest())
            .set(WindowPlugin {
                primary_window: Some(Window {
                    resolution: WindowResolution::new(1024., 1024.).with_scale_factor_override(1.0),
                    ..default()
                }),
                ..default()
            }),
        #[cfg(not(target_arch = "wasm32"))]
        WireframePlugin,
        // AtmospherePlugin,
    ))
    .insert_resource(WireframeConfig {
        global: true,
        default_color: WHITE.into(),
    })
    .insert_resource(Decompositions::new())
    .insert_resource(DecompositionsPosition(Vec3::ZERO, Vec3::ZERO))
    .add_systems(Startup, (setup, voronoi_demo))
    .add_systems(Update, draw_decomposition_gizmos)
    .add_systems(PreUpdate, spin_camera);

    if take_screenshots_and_exit {
        app.add_systems(PostUpdate, _take_screenshots_and_exit);
    }
    app.run();
}

const NFRAMES: u32 = 240;
const NSKIP: u32 = 20;

fn _take_screenshots_and_exit(
    time1: Res<FrameCount>,
    mut app_exit_events: ResMut<Events<bevy::app::AppExit>>,
    mut screenshot_manager: ResMut<ScreenshotManager>,
    main_window: Query<Entity, With<PrimaryWindow>>,
) {
    if time1.0 <= NSKIP {
        return;
    }

    let current_frame = time1.0 - NSKIP;

    if current_frame < NFRAMES {
        let path = format!("./screenshot/{:05}.png", current_frame);
        screenshot_manager
            .save_screenshot_to_disk(main_window.single(), path)
            .unwrap();
    }

    if current_frame > NFRAMES + NSKIP {
        app_exit_events.send(bevy::app::AppExit::Success);
    }
}

fn spin_camera(mut camera_tr: Query<&mut Transform, With<MainCamera>>, time1: Res<FrameCount>) {
    let phi = (time1.0 as f64) / NFRAMES as f64 * core::f64::consts::PI * 2.0
        + core::f64::consts::FRAC_PI_4;
    let xx = phi.sin() as f32;
    let zz = phi.cos() as f32;

    // let dist_scale = (((NFRAMES + current_frame * 5) as f32 / NFRAMES as f32)).cbrt();
    let v2 = Vec3::new(500.8 * xx, 533.8, 500.8 * zz); // / dist_scale;
    let transform = Transform::from_translation(v2)
        .looking_at(Vec3::Y * CONTAINER_HEIGHT as f32 / 2.0, Vec3::Y);

    *camera_tr.single_mut() = transform;
}

fn draw_decomposition_gizmos(
    mut gizmos: Gizmos,
    d: Res<Decompositions>,
    d_tr: Res<DecompositionsPosition>,
) {
    let d = d.as_ref();
    draw_gizmos(&d.points_meshless, &d.mask, d_tr.0, &mut gizmos, YELLOW);
    draw_gizmos(&d.points_voro_rs, &d.mask, d_tr.1, &mut gizmos, RED);
}

fn draw_gizmos(
    cells: &Vec<VCellRaw>,
    mask: &[bool],
    offset: Vec3,
    gizmos: &mut Gizmos,
    color: impl Into<Color> + Clone,
) {
    for cell in cells {
        if !mask[cell.id as usize] {
            continue;
        }
        let mut edge_hash = std::collections::HashMap::<(i32, i32), ()>::new();
        let offset = cell.position + offset;
        gizmos.sphere(offset, Quat::IDENTITY, 0.1, color.clone());
        for face_ids in cell.faces.iter() {
            let mut face_ids_shift = face_ids.clone();
            face_ids_shift.rotate_right(1);
            for (i1, i2) in face_ids.iter().zip(face_ids_shift.iter()) {
                if let std::collections::hash_map::Entry::Vacant(e) = edge_hash.entry((*i1, *i2)) {
                    e.insert(());

                    let p1 = offset + cell.vs[*i1 as usize];
                    let p2 = offset + cell.vs[*i2 as usize];
                    gizmos.line(p1, p2, color.clone());
                }
            }
        }
    }
}

fn voronoi_demo(mut commands: Commands) {
    info!("hello world 1");
    warn!("hello world 2");
    error!("hello world 3");
    let size = Vec3::new(
        CONTAINER_WIDTH as f32,
        CONTAINER_HEIGHT as f32,
        CONTAINER_WIDTH as f32,
    );
    let points = random_points(size, NPOINTS);
    eprintln!(
        "{:?} {:?} {:?} {:?}",
        points[0], points[1], points[2], points[3]
    );
    let (points, mask) = perturbed_grid(size, NPOINTS, 0.25);
    eprintln!(
        "{:?} {:?} {:?} {:?}",
        points[0], points[1], points[2], points[3]
    );

    let _p1 = points.clone();
    let mut _c_meshless = log_time!(
        "MESHLESS VORO",
        crate::impl_meshless_voro::impl_meshless_voro(size, _p1)
    );
    let mut _c_voro_rs = log_time!("VORO RS", crate::impl_voro_rs::impl_voro_rs(size, points));

    _c_meshless.sort_by_key(|z| z.id);
    _c_voro_rs.sort_by_key(|z| z.id);

    assert!(_c_meshless.len() == _c_voro_rs.len());
    for (cell1, cell2) in _c_meshless.iter().zip(_c_voro_rs.iter()) {
        // CHECK IDS & POSITIONS
        assert!(cell1.id == cell2.id);
        assert!((cell1.volume - cell2.volume).abs() < 1e-3);
        assert!(cell1.position.distance(cell2.position) < 1e-5);
        assert!(cell1.centroid.distance(cell2.centroid) < 1e-3);
    }
    commands.insert_resource(Decompositions {
        points_meshless: _c_meshless,
        points_voro_rs: _c_voro_rs,
        mask,
    });
}

fn setup(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut images: ResMut<Assets<Image>>,
) {
    let custom_texture_handle: Handle<Image> = images.add(crate::debug_texture::uv_debug_texture());

    let box_size = Vec3::new(
        CONTAINER_WIDTH as f32,
        CONTAINER_HEIGHT as f32,
        CONTAINER_WIDTH as f32,
    );
    let box1 = meshes.add(Cuboid::from_corners(Vec3::ZERO, box_size));

    let t1 = Transform::from_xyz(0., CONTAINER_HEIGHT as f32 / 2., 0.0);
    let t2 = Transform::from_xyz(
        -CONTAINER_WIDTH as f32,
        CONTAINER_HEIGHT as f32 / 2.,
        -CONTAINER_WIDTH as f32 * 1.5,
    );

    commands.insert_resource(DecompositionsPosition(
        // move coords to have box center in 0,0 (same as bevy box mesh)
        t1.translation - box_size / 2.0,
        t2.translation - box_size / 2.0,
    ));

    // Render the mesh with the custom texture using a PbrBundle, add the marker.
    commands.spawn((
        PbrBundle {
            mesh: box1.clone(),
            material: materials.add(StandardMaterial {
                base_color_texture: Some(custom_texture_handle.clone()),
                ..default()
            }),
            transform: t1,
            ..default()
        },
        ContainerBoxMesh,
    ));
    // Render the mesh with the custom texture using a PbrBundle, add the marker.
    commands.spawn((
        PbrBundle {
            mesh: box1,
            material: materials.add(StandardMaterial {
                base_color_texture: Some(custom_texture_handle),
                ..default()
            }),
            transform: t2,
            ..default()
        },
        ContainerBoxMesh,
    ));

    // // ground plane
    commands.spawn(PbrBundle {
        mesh: meshes.add(
            Plane3d::default()
                .mesh()
                .size(5550.0, 5550.0)
                .subdivisions(7),
        ),
        material: materials.add(Color::from(SILVER)),
        ..default()
    });

    // Camera in 3D space.
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(-355.8, 533.8, 355.8)
                .looking_at(Vec3::Y * CONTAINER_HEIGHT as f32 / 2.0, Vec3::Y),
            ..default()
        },
        MainCamera,
        //   AtmosphereCamera::default(),
    ));

    // directional 'sun' light
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: light_consts::lux::FULL_DAYLIGHT,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform {
            translation: Vec3::new(0.0, 2.0, 0.0),
            rotation: Quat::from_rotation_x(std::f32::consts::PI / 4.)
                * Quat::from_rotation_z(std::f32::consts::PI / 4.),
            ..default()
        },
        // The default cascade config is designed to handle large scenes.
        // As this example has a much smaller world, we can tighten the shadow
        // bounds for better visual quality.
        cascade_shadow_config: CascadeShadowConfigBuilder {
            first_cascade_far_bound: 550.0,
            maximum_distance: 1500.0,
            ..default()
        }
        .into(),
        ..default()
    });

    // // Text to describe the controls.
    // commands.spawn(
    //     TextBundle::from_section(
    //         "Controls:\n Space: Change UVs\n X/Y/Z: Rotate\n R: Reset orientation",
    //         TextStyle::default(),
    //     )
    //     .with_style(Style {
    //         position_type: PositionType::Absolute,
    //         top: Val::Px(12.0),
    //         left: Val::Px(12.0),
    //         ..default()
    //     }),
    // );
}

