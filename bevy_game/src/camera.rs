use crate::AppState;
use crate::game::{COLLISION_TERRAIN, COLLISION_MISSILE};
use bevy::input::mouse::MouseWheel;
//use bevy::prelude::shape::UVSphere;
//use bevy::render::primitives::Sphere;
use bevy::ecs::component::Component;
use bevy::prelude::*;
//use bevy_prototype_debug_lines::DebugLines;
use bevy_rapier3d::plugin::RapierContext;
use bevy_rapier3d::prelude::InteractionGroups;
//use heron::rapier_plugin::PhysicsWorld;

use std::f32::consts::FRAC_PI_2;
use std::{marker::PhantomData, ops::Mul};

#[derive(Component)]
pub struct CameraTarget;

pub struct CameraPlugin<T: 'static + Send + Sync>(pub PhantomData<T>);


#[derive(Component, Debug)]
pub struct MyCamera;

#[derive(Debug, Resource)]
pub struct CameraState {
    pub forward: Vec3,
    pub right: Vec3,
    pub dist: f32,           // meters
    pub pitch: f32,          // rad
    pub yaw: f32,            // rad
    pub rotate_speed_x: f32, // rad
    pub rotate_speed_y: f32, // rad
    pub screen_dist_x: f32,  // pixels
    pub screen_dist_y: f32,  // pixels
    pub cursor_prev: Vec2,
    pub cursor_latest: Vec2,
    pub global_position: Vec3,
    pub mouse_ray: Vec3,
    pub mouse_hit_position: Option<Vec3>,    
    pub center_screen_ray: Vec3,
    pub center_screen_hit_position: Option<Vec3>,    
    mleft_press_position: Option<Vec3>,
}

impl Default for CameraState {
    fn default() -> Self {
        Self {
            forward: Vec3::new(0., 0., -1.),
            right: Vec3::new(1., 0., 0.),
            dist: 30.0,
            pitch: -1.0,//-FRAC_PI_2,
            yaw: 0.,
            rotate_speed_x: 1.,
            rotate_speed_y: 0.3,
            screen_dist_x: 100.,
            screen_dist_y: 100.,
            cursor_prev: Vec2::new(101., 101.),
            cursor_latest: Vec2::new(101., 101.),
            global_position: Vec3::new(0., 0., 0.),
            mouse_ray: Vec3::new(0., -1., 0.),
            mouse_hit_position: None,
            center_screen_ray: Vec3::new(0., -1., 0.),
            center_screen_hit_position: None, 

            mleft_press_position: None,
        }
    }
}

impl<T: 'static + Send + Sync> Plugin for CameraPlugin<T> {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<CameraState>()
            .add_system_set(
                SystemSet::on_enter(AppState::Playing)
                    .with_system(setup::<T>)
            )
            .add_system_set(
                SystemSet::on_update(AppState::Playing)
                    .with_system(follow_target)
                    .with_system(update_dir_by_mouse)
                    .with_system(update_dist_by_mouse                    
                        .before(update_ray_with_cursor))

                    .with_system(update_ray_with_cursor
                        .after(update_dir_by_mouse)
                        .before(obr_mouse))

                    .with_system(obr_mouse
                        .after(update_ray_with_cursor)
                        .before(move_camera_target_by_mouse))
                        
                    .with_system(move_camera_target_by_mouse
                        .after(obr_mouse))
            );
    }
}

impl<T: 'static + Send + Sync> Default for CameraPlugin<T> {
    fn default() -> Self {
        CameraPlugin(PhantomData::<T>)
    }
}

fn setup<T: 'static + Send + Sync>(
    mut commands: Commands,
//    mut meshes: ResMut<Assets<Mesh>>,
//    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands
        .spawn_bundle(Camera3dBundle {
            transform: Transform::from_xyz(0., 50.0, 0.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..Default::default()
        })
        .insert(MyCamera);

    commands
        .spawn_bundle((Transform::IDENTITY, GlobalTransform::IDENTITY))
        /* .spawn_bundle(PbrBundle {
            mesh: meshes.add(Mesh::from(UVSphere {
                radius: 0.1,
                sectors: 8,
                stacks: 8,
            })),

            material: materials.add(StandardMaterial {
                base_color: Color::RED,
                emissive: Color::rgba_linear(100.0, 0.0, 0.0, 0.0),
                ..default()
            }),

            ..default()
        })*/
        .insert(CameraTarget);
}

fn follow_target(
    time: Res<Time>,
    windows: Res<Windows>,
    mut camera_state: ResMut<CameraState>,
    mut camera_query: Query<(&MyCamera, &mut Transform), Without<CameraTarget>>,
    mut target_query: Query<(&CameraTarget, &mut Transform), Without<MyCamera>>,
) {
    let window = windows.get_primary().unwrap();

    let dx = if camera_state.cursor_latest.x < camera_state.screen_dist_x {
        camera_state.screen_dist_x - camera_state.cursor_latest.x
    } else if camera_state.cursor_latest.x > window.width() - camera_state.screen_dist_x {
        window.width() - camera_state.cursor_latest.x - camera_state.screen_dist_x
    } else {
        camera_state.screen_dist_x + 1.
    } / camera_state.screen_dist_x;

    if dx.abs() < 1.0 {
        camera_state.yaw += dx * camera_state.rotate_speed_x * time.delta_seconds();
    }

    let dy = if camera_state.cursor_latest.y < camera_state.screen_dist_y {
        camera_state.screen_dist_y - camera_state.cursor_latest.y
    } else if camera_state.cursor_latest.y > window.height() - camera_state.screen_dist_y {
        window.height() - camera_state.cursor_latest.y - camera_state.screen_dist_y
    } else {
        camera_state.screen_dist_y + 1.
    } / camera_state.screen_dist_y;

    if dy.abs() < 1.0 {
        camera_state.pitch -= dy * camera_state.rotate_speed_y * time.delta_seconds(); 
    }

    camera_state.pitch = camera_state.pitch.clamp(-1.5, -0.20);

    if let Ok((_camera, mut cam_transform)) = camera_query.get_single_mut() {
        if let Ok((_path, target_transform)) = target_query.get_single_mut() {
            cam_transform.rotation = Quat::from_axis_angle(Vec3::Y, camera_state.yaw)
                * Quat::from_axis_angle(Vec3::X, camera_state.pitch);

            camera_state.forward =
                Vec3::new(cam_transform.forward().x, 0., cam_transform.forward().z).normalize();

            camera_state.right =
                Vec3::new(cam_transform.right().x, 0., cam_transform.right().z).normalize();

            // * time.delta_seconds();
            cam_transform.translation =
                target_transform.translation + cam_transform.back().mul(camera_state.dist);
        }
    }
}

fn update_dir_by_mouse(
    mut cursor: EventReader<CursorMoved>,
    mut camera_state: ResMut<CameraState>,
    camera_query: Query<&Transform, With<MyCamera>>,
) {
    if cursor.len() < 1 {
        return;
    }

    if let Ok(cam_transform) = camera_query.get_single() {
        camera_state.forward =
            Vec3::new(cam_transform.forward().x, 0., cam_transform.forward().z).normalize();

        camera_state.right =
            Vec3::new(cam_transform.right().x, 0., cam_transform.right().z).normalize();
    }

    camera_state.cursor_prev = camera_state.cursor_latest;
    camera_state.cursor_latest = cursor.iter().last().unwrap().position;
}

fn update_dist_by_mouse(
    mut camera_state: ResMut<CameraState>,
    mut scroll_evr: EventReader<MouseWheel>,
) {
    for ev in scroll_evr.iter() {
        camera_state.dist -= camera_state.dist * 0.05 * ev.y;
    }
}

fn update_ray_with_cursor(
    //    mut lines: ResMut<DebugLines>,
    //    mut cursor: EventReader<CursorMoved>,
    windows: Res<Windows>,
    mut camera_state: ResMut<CameraState>,
    camera_query: Query<(&Camera, &GlobalTransform), With<MyCamera>>,
) {
    /*  let cursor_pos = if let Some(cursor_latest) = cursor.iter().last() {
        cursor_latest.position
    } else {
        return;
    };*/

    let window = windows.primary();

    if let Ok((camera, camera_transform)) = camera_query.get_single() {
        /*        if window.id() != camera. {
                    panic!("Generating Ray from Camera with wrong Window");
                }
        */
        camera_state.global_position = camera_transform.translation();

        camera_state.mouse_ray = screen_to_world_dir(
            &camera_state.cursor_latest,
            &Vec2::from([window.width() as f32, window.height() as f32]),
            camera,
            camera_transform,
        );

        camera_state.center_screen_ray = screen_to_world_dir(
            &Vec2::from([window.width()/2. as f32, window.height()/2. as f32]),
            &Vec2::from([window.width() as f32, window.height() as f32]),
            camera,
            camera_transform,
        );


    /*
        let world_coord = camera_transform.translation + dir * 30.; // distance from the camera to put the world coord.

    //       dbg![cursor_pos, screen_size, normal, world_coord];

            lines.line_colored(
                camera_transform.translation,
                world_coord,
                10.0,
                Color::RED,
            );

            lines.line_colored(
                Vec3::new(world_coord.x, 100.0, world_coord.z),
                world_coord,
                10.0,
                Color::BLACK,
            );
    */
    } else {
        return;
    };
}

pub fn screen_to_world_dir(
    mouse_position: &Vec2,
    screen_size: &Vec2,
    camera: &Camera,
    camera_transform: &GlobalTransform,
) -> Vec3 {
    let view = camera_transform.compute_matrix();

    let projection = camera.projection_matrix();
    let cursor_ndc = Vec2::new(
        2.0 * (mouse_position.x / screen_size.x) - 1.0,
        2.0 * (mouse_position.y / screen_size.y) - 1.0,
    );

    let ndc_to_world: Mat4 = view * projection.inverse();
    let world_to_ndc = projection * view;

    let projection = projection.to_cols_array_2d();
    let camera_near = (2.0 * projection[3][2]) / (2.0 * projection[2][2] - 2.0);

    let ndc_near = world_to_ndc.transform_point3(-Vec3::Z * camera_near).z;
    let cursor_pos_near = ndc_to_world.transform_point3(cursor_ndc.extend(ndc_near));

    let dir = cursor_pos_near - camera_transform.translation();

    //    dbg![mouse_position, cursor_ndc, camera_transform.translation, camera_near, cursor_pos_near, dir];

    return dir.normalize();
}

fn move_camera_target_by_mouse(
    //    mut lines: ResMut<DebugLines>,
    mut camera_state: ResMut<CameraState>,
    rapier_context: Res<RapierContext>,
    //   bodies: Query<&RigidBody>,
    //    mouse_event: Res<Input<MouseButton>>,
    mut target_query: Query<&mut Transform, (With<CameraTarget>, Without<MyCamera>)>,
    mut camera_query: Query<(&MyCamera, &mut Transform), Without<CameraTarget>>,
) {

//    log::info!("move_camera_target_by_mouse start");

    /*
        lines.line_colored(
            Vec3::new(ray.x, 100.0, ray.z),
            ray,
            1.0,
            Color::BLACK,
        );
    */

    let filter = bevy_rapier3d::prelude::QueryFilter::from(
        InteractionGroups::new(
            unsafe { bevy_rapier3d::rapier::geometry::Group::from_bits_unchecked(COLLISION_MISSILE) },
            unsafe { bevy_rapier3d::rapier::geometry::Group::from_bits_unchecked(COLLISION_TERRAIN) },
        )
    );

    // Then cast the ray.
    let result = rapier_context.cast_ray(
        camera_state.global_position,
        camera_state.mouse_ray,
        f32::MAX,
        true,
        filter,/* {
            flags: 0,//bevy_rapier3d::rapier::prelude::QueryFilterFlags::EXCLUDE_SENSORS,
            groups: Some(InteractionGroups::new(COLLISION_TERRAIN, COLLISION_TERRAIN)),
            exclude_collider: None,
            exclude_rigid_body: None,
            predicate: None,
        }*/
    );

    camera_state.mouse_hit_position = if let Some((_entity, toi)) = result {        
 //       log::info!("move_camera_target_by_mouse mouse_hit_position ok");
        Some(camera_state.global_position + camera_state.mouse_ray * toi)
    } else {
//        log::info!("move_camera_target_by_mouse mouse_hit_position error");
        None
    };

 //   flags: bevy_rapier3d::rapier::prelude::QueryFilterFlags::EXCLUDE_SENSORS,
 //   groups: Some(InteractionGroups::new(COLLISION_TERRAIN, COLLISION_TERRAIN)),

 //           flags: bevy_rapier3d::rapier::prelude::QueryFilterFlags::ONLY_FIXED,
  //          groups: Some(InteractionGroups::new(COLLISION_TERRAIN, EXCLUDE_TERRAIN)),

    let result = rapier_context.cast_ray(
        camera_state.global_position,
        camera_state.center_screen_ray,
        f32::MAX,
        true,
        filter,
    );

    camera_state.center_screen_hit_position = if let Some((_entity, toi)) = result {
 //       log::info!("move_camera_target_by_mouse center_screen_hit_position ok");
        Some(camera_state.global_position + camera_state.center_screen_ray * toi)
    } else {
//        log::info!("move_camera_target_by_mouse center_screen_hit_position error");
        None
    };

    // drag camera pos with mouse
    if let Some(mleft_press_position) = camera_state.mleft_press_position {
        if let Some(ray_hit_position) = camera_state.mouse_hit_position {
            if let Ok(mut target_transform) = target_query.get_single_mut() {
                if ray_hit_position.abs_diff_eq(mleft_press_position, 0.1) {
                    return;
                }

                let new_pos = Vec3::new(
                    target_transform.translation.x - ray_hit_position.x + mleft_press_position.x,
                    target_transform.translation.y,
                    target_transform.translation.z - ray_hit_position.z + mleft_press_position.z,
                );

                let new_dir = (new_pos - camera_state.global_position).normalize();

                let new_result = rapier_context.cast_ray(
                    camera_state.global_position,
                    new_dir,
                    f32::MAX,
                    true,
                    filter,
                );

                if let Some((_entity, _toi)) = new_result {
                    if let Ok((_camera, mut cam_transform)) = camera_query.get_single_mut() {
                        let delta_pos = camera_state.global_position + new_dir * _toi
                            - target_transform.translation;
                        target_transform.translation += delta_pos;
                        cam_transform.translation += delta_pos;
                    }
                }
            }
        }
    }
}

fn obr_mouse(
    mut camera_state: ResMut<CameraState>, 
    mouse_event: Res<Input<MouseButton>>
) {
    if mouse_event.just_pressed(MouseButton::Right) {
        camera_state.mleft_press_position = camera_state.mouse_hit_position.clone();
    } else if mouse_event.just_released(MouseButton::Right) {
        camera_state.mleft_press_position = None;
    }
}
