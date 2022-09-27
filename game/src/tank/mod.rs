use bevy::prelude::{
    shape::{Cube, UVSphere},
    *,
};
use serde::{Deserialize, Serialize};
use std::{collections::{LinkedList, HashMap}, f32::consts::PI};
use bevy_rapier3d::prelude::*;

use bevy_prototype_debug_lines::DebugLines;
use iyes_loopless::prelude::*;

use crate::{
    loading::ModelAssets,
    player::{LocalHandles, PlayerData, PlayerHandle},
    AppState, shot::TankShotOutData, menu::is_play_online, network::PingList,
};

use crate::shot::Data as ShotData;

pub mod body_tank_physics;
use body_tank_physics::*;

//structs for exchange with network
#[repr(C)]
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct TankBodyOutData {
    pub movement: Vec2,
    pub pos: Vec2,
    pub dir: f32,
}

/* 
#[repr(C)]
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct TankTurretOutData {
    pub speed: f32,
    pub dir: f32,
}
*/
#[repr(C)]
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct Rotation {
    pub speed: f32,
    pub dir: f32,
}

#[repr(C)]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum TurretMessage {
    StartRotate(Rotation),
    ContinueRotate(Rotation),
    StopRotate(Rotation),
}
impl Default for TurretMessage {
        fn default() -> Self {
        TurretMessage::StopRotate(Rotation::default())
    }
}

#[repr(C)]
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct TankCannonOutData {
    pub speed: f32,
    pub dir: f32,
}

pub struct InBody {
    pub data: HashMap<PlayerHandle, TankBodyOutData>,
}
pub struct InTurret {
    pub data: HashMap<PlayerHandle, TurretMessage>,
}
pub struct InCannon {
    pub data: HashMap<PlayerHandle, TankCannonOutData>,
}

//componentsin entities for control
#[derive(Component, Debug, Default)]
pub struct TankControlBody {
    pub movement: Vec2,
    pub pos: Vec2,
    pub dir: f32,
}
#[derive(Component, Debug, Default)]
pub struct TankControlTurret {
    pub dir: f32,
    pub speed: f32,
}

#[derive(Component, Debug, Default)]
pub struct TankControlCannon {
    pub dir: f32,
    pub speed: f32,
}

#[derive(Component, Debug, Default, PartialEq)]
pub struct TankControlActionShot {
    pub time: f32,
    pub is_shot: bool,
    pub pos: Vec3,
    pub vel: Vec3,
}

#[derive(Component, Debug)]
pub struct TankShotData {
    pub shot_speed_min: f32,
    pub shot_speed_delta: f32,
    pub shot_live_max_time: f32,
    pub explosion_force: f32,
}

impl TankShotData {
    fn init() -> Self {
        Self {
            shot_speed_min: 10.,
            shot_speed_delta: 5.,
            shot_live_max_time: 30.,
            explosion_force: 20.,
        }
    }
}

#[derive(Component, Debug)]
pub struct TankEntityes {
    pub body: Entity,
    pub turret: Entity,
    pub cannon: Entity,
    pub fire_point: Entity,
    pub wheels: LinkedList<Entity>,
}

#[derive(Default, Debug)]
pub struct NewTank {
    pub handle: usize,
    pub pos: Vec3,
    pub angle: f32,
}

#[derive(Debug)]
pub struct NewTanksData {
    pub vector: Vec<NewTank>,
}

impl Default for NewTanksData {
    fn default() -> Self {
        Self { vector: vec![] }
    }
}

pub struct TankPlugin;

impl Plugin for TankPlugin {
    fn build(&self, app: &mut App) {
        let before_system_set = SystemSet::on_update(AppState::Playing)
            .with_system(obr_in_body.run_if(is_play_online))
            .with_system(obr_in_turret.run_if(is_play_online))
            .with_system(obr_in_cannon.run_if(is_play_online))
            .with_system(
                update_body_position
                    //                  .label(InputLabel::ApplyInput)
                    //                  .after(InputLabel::PrepInput)
                    .before(update_body_moving),
            )
            .with_system(
                update_body_moving
                    //                  .label(InputLabel::ApplyInput)
                    //                  .after(InputLabel::PrepInput)
                    .before(update_turret_rotation),
            )
            .with_system(
                update_turret_rotation
                    .after(update_body_moving)
                    .before(update_cannon_rotation),
            )
            .with_system(
                update_cannon_rotation
                    .after(update_turret_rotation)
                    .before(update_cannon_debug_line)
                    .before(update_cannon_shot),
            )
            .with_system(update_cannon_debug_line.after(update_cannon_rotation))
            .with_system(update_cannon_shot.after(update_cannon_rotation));

        //  let after_system_set = SystemSet::on_update(AppState::Playing)
        //    .with_system(print_after_system)
        //      .with_system(handle_explosion_events);

        //   let update_system_set = SystemSet::on_update(AppState::Playing)
        //    .with_system(print_update_system)
        //     .with_system(display_events)
        //      .with_system(remove_shots)
        //       .with_system(apply_explosion)
        //      .with_system(process_explosion_event)
        //        .with_system(accelerate_system)
        //     ;

        app.init_resource::<NewTanksData>()
            .add_system_set(SystemSet::on_enter(AppState::Loading).with_system(setup))
            .add_system_set_to_stage(CoreStage::PreUpdate, before_system_set)
            .add_system(obr_spawn_tanks.run_if(is_create_tanks))
            .add_system_set(ConditionSet::new().run_if(is_create_tanks).into());
    }
}

fn is_create_tanks(data: Res<NewTanksData>) -> bool {
    //   println!("tank is_create_tanks");
    !data.vector.is_empty()
}

fn setup(
    mut commands: Commands,
    //   asset_server: Res<AssetServer>,
) {
    println!("Tank setup");

    //   let _scenes: Vec<HandleUntyped> = asset_server.load_folder("Tank_1/PARTS").unwrap();
}

pub fn obr_spawn_tanks(
    mut data: ResMut<NewTanksData>,
    mut commands: Commands,
    model_assets: Res<ModelAssets>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for data in &data.vector {
/*      create_debug_tank(
            &mut commands,
            data.handle,
            data.pos,
            data.angle,
            &mut meshes,
            &mut materials,
        );
*/
        create_tank(
            &mut commands,
            data.handle,
            data.pos,
            data.angle,
            &model_assets,
        );
    }

    data.vector.clear();
}

fn create_debug_tank(
    mut commands: &mut Commands,
    player_handle: usize,
    pos: Vec3,
    angle: f32,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) {
    let body_size = Vec3::new(1., 0.45, 1.6);
    let config = VehicleConfig::new(body_size);

    let body = commands
        .spawn_bundle(SpatialBundle {
            transform: Transform::from_translation(pos)
                .with_rotation(Quat::from_axis_angle(Vec3::Y, angle)),
            ..Default::default()
        })
        .id();

    let (body, wheels) = create_body(
        body,
        &mut commands,
        pos,
        angle,
        config,
        CollisionGroups::new(0b0010, 0b1111),
        SolverGroups::new(0b0010, 0b1111),
    );

    /*    commands
            .entity(body)
            .insert(LockedAxes::ROTATION_LOCKED | LockedAxes::TRANSLATION_LOCKED );
    */
    let turret_base = commands
        .spawn_bundle(SpatialBundle {
            transform: Transform::from_translation(Vec3::new(0.0, 0.10, 0.0)),
            ..Default::default()
        })
        .id();

    commands.entity(body).add_child(turret_base);

    let turret = commands
        .spawn_bundle(SpatialBundle {
            transform: Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
            ..Default::default()
        })
        .insert(TankControlTurret::default())
        .id();

    commands.entity(turret_base).add_child(turret);

    let cannon_base = commands
        .spawn_bundle(SpatialBundle {
            transform: Transform::from_translation(Vec3::new(0.0, 0.50, 0.45)),
            ..Default::default()
        })
        .id();

    commands.entity(turret).add_child(cannon_base);

    let cannon = commands
        .spawn_bundle(SpatialBundle {
            transform: Transform::from_translation(Vec3::new(0., 0., 0.)),
            ..Default::default()
        })
        .insert(TankControlCannon::default())
        .id();

    commands.entity(cannon_base).add_child(cannon);

    let fire_point = commands
        .spawn_bundle(SpatialBundle {
            transform: Transform::from_translation(Vec3::new(0., 0., 0.5)),
            ..Default::default()
        })
        .insert(TankShotData::init())
        .insert(TankControlActionShot::default())
        .with_children(|parent| {
            parent.spawn_bundle(PbrBundle {
                mesh: meshes.add(Mesh::from(Cube::new(0.1))),
                material: materials.add(Color::RED.into()),
                transform: Transform::from_translation(Vec3::new(0., 0., 0.5)),
                //        rotation:
                ..Default::default()
            });
        })
        .id();

    commands.entity(cannon).add_child(fire_point);

    commands.entity(body).insert(PlayerData {
        handle: player_handle,
    });
    commands.entity(turret).insert(PlayerData {
        handle: player_handle,
    });
    commands.entity(cannon).insert(PlayerData {
        handle: player_handle,
    });
    commands.entity(fire_point).insert(PlayerData {
        handle: player_handle,
    });
    wheels.iter().for_each(|wheel| {
        commands.entity(*wheel).insert(PlayerData {
            handle: player_handle,
        });
    });

    let data = TankEntityes {
        body,
        turret,
        cannon,
        fire_point,
        wheels,
    };

    commands.entity(body).insert(data);
}

fn create_tank(
    mut commands: &mut Commands,
    //    asset_server: &Res<AssetServer>,
    player_handle: usize,
    pos: Vec3,
    angle: f32,
    model_assets: &Res<ModelAssets>,
    //    material: &Handle<bevy::prelude::StandardMaterial>,
) {
    let body_size = Vec3::new(1., 0.45, 1.6);
    let config = VehicleConfig::new(body_size);

    let body = commands
        .spawn_bundle(SceneBundle {
            scene: model_assets.tank_body.clone(),
            transform: Transform::from_translation(pos)
                .with_rotation(Quat::from_axis_angle(Vec3::Y, angle)),
            ..Default::default()
        })
        .id();

    let (body, wheels) = create_body(
        body,
        &mut commands,
        pos,
        angle,
        config,
        CollisionGroups::new(0b0010, 0b1111),
        SolverGroups::new(0b0010, 0b1111),
    );

    let turret = commands
        .spawn_bundle(SceneBundle {
            scene: model_assets.tank_turret.clone(),
            transform: Transform::from_translation(Vec3::new(0.0, 0.10, 0.0)),
            //               visibility: visibility.clone(),
            ..Default::default()
        })
        .insert(TankControlTurret::default())
        .id();

    commands.entity(body).add_child(turret);

    let cannon = commands
        .spawn_bundle(SceneBundle {
            scene: model_assets.tank_cannon.clone(),
            transform: Transform::from_translation(Vec3::new(0.0, 0.50, 0.45)),
            ..Default::default()
        })
        .insert(TankControlCannon::default())
        .id();

    commands.entity(turret).add_child(cannon);

    let fire_point = commands
        .spawn_bundle(TransformBundle {
            local: Transform::from_translation(Vec3::new(0., 0., 0.5)),
            global: GlobalTransform::identity(),
        })
        .insert(TankShotData::init())
        .insert(TankControlActionShot::default())
        .id();
    /*        .with_children(|parent| {
        parent.spawn_bundle(PbrBundle {
            mesh: meshes.add(Mesh::from(Cube::new(0.1))),
            material: materials.add(Color::RED.into()),
            transform: Transform::from_translation(Vec3::new(0., 0., 0.5)),
            //        rotation:
            ..Default::default()
        })
    })
    */

    /*        // fire point
            .with_children(|from| {
                from.spawn_bundle(TransformBundle {
                    local: Transform::from_translation(Vec3::new(0., 0., 0.5)),
                    //        rotation:
                    ..Default::default()
                })
                .insert(TankControlCannonShot::default());
            }).id();
    */
    commands.entity(cannon).add_child(fire_point);

    commands.entity(body).insert(PlayerData {
        handle: player_handle,
    });
    commands.entity(turret).insert(PlayerData {
        handle: player_handle,
    });
    commands.entity(cannon).insert(PlayerData {
        handle: player_handle,
    });
    commands.entity(fire_point).insert(PlayerData {
        handle: player_handle,
    });
    wheels.iter().for_each(|wheel| {
        commands.entity(*wheel).insert(PlayerData {
            handle: player_handle,
        });
    });

    let data = TankEntityes {
        body,
        turret,
        cannon,
        fire_point,
        wheels,
    };

    commands.entity(body).insert(data);
}

pub fn update_body_position(
    local_handles: Res<LocalHandles>,
    time: Res<Time>,
    mut data_query: Query<(
        &GlobalTransform,
        ChangeTrackers<TankControlBody>,
        &TankControlBody,
        &mut ExternalImpulse,
        &mut Sleeping,
        &TankEntityes,
        &PlayerData,
    )>,
    //        &mut ExternalForce,
    mut out_data: ResMut<TankBodyOutData>,
    mut wheel_data_query: Query<&mut WheelData>,
) {
    for (
        global_transform,
        tank_control_body_tracker,
        tank_control_body,
        /*tank_control_data, mut forces,*/ mut impulse,
        mut sleeping,
        tank_entityes,
        player,
    ) in data_query.iter_mut()
    {
        let (_scale, rotation, translation) = global_transform.to_scale_rotation_translation();

        if *local_handles.handles.first().unwrap() == player.handle {
            let new_pos = Vec2::new(translation.x, translation.z);
            let new_dir = rotation.to_euler(EulerRot::YXZ).0;

            if !tank_control_body
                .movement
                .abs_diff_eq(out_data.movement, 0.01)
                || !new_pos.abs_diff_eq(out_data.pos, 0.01)
                || (out_data.dir - new_dir).abs() >= 1.
            {
                out_data.movement = tank_control_body.movement;
                out_data.pos = new_pos;
                out_data.dir = new_dir;
            }
        } else {
            //correct body pos
            let delta_pos = Vec3::new(
                tank_control_body.pos.x - translation.x,
                0.,
                tank_control_body.pos.y - translation.z,
            );

            //       log::info!("tank mod update_body_position translation.pos {} input.pos{} delta_pos{}",
            //           transform.translation, tank_control_body.pos, delta_pos);

            impulse.impulse = delta_pos*delta_pos.length_squared()*100.*time.delta_seconds();

  /*           impulse.impulse = if delta_pos.length_squared() > 1. {
                delta_pos.normalize_or_zero()
            } else {
                delta_pos
            } * 10.;
*/
            let current_body_dir = rotation.to_euler(EulerRot::YXZ).0;
            let torque = calc_delta_dir(tank_control_body.dir, current_body_dir, 30. * PI / 180.)*10000.*time.delta_seconds();

            //       log::info!("tank mod update_body_position current_dir: {}; from_net.dir: {}; torque: {}",
            //       current_body_dir, tank_control_body.dir, torque);

            impulse.torque_impulse = rotation.mul_vec3(Vec3::Y * torque);
        }

        if tank_control_body_tracker.is_changed() {
            let wheel_data_movement = if tank_control_body.movement.length_squared() > 0.001 {
                sleeping.linear_threshold = -1.;
                sleeping.angular_threshold = -1.;
                sleeping.sleeping = false;
                Some(tank_control_body.movement.clone())
            } else {
                sleeping.linear_threshold = 1.;
                sleeping.angular_threshold = 10.;
                sleeping.sleeping = true;
                //          sleeping.default();
                None
            };

            for wheel in &tank_entityes.wheels {
                if let Ok(mut wheel_data) = wheel_data_query.get_component_mut::<WheelData>(*wheel)
                {
                    wheel_data.movement = wheel_data_movement.clone();

                    //           println!("player prep_wheel_input, ok");
                }
            }
        }
    }
}

pub fn calc_delta_dir(new_dir: f32, old_dir: f32, max_delta: f32) -> f32 {
    let mut delta = new_dir - old_dir;

    let res = if delta.abs() > max_delta {
        if delta.abs() > std::f32::consts::PI {
            delta = -delta;
        }

        max_delta*delta.signum()

    } else {
        delta
    };

    res
}

pub fn calc_dir(dir: f32, old_dir: f32, rot_speed: f32, delta_time: f32) -> f32 {
    let delta = calc_delta_dir(dir, old_dir, rot_speed * delta_time); 
    let new_dir = dir + delta;//TODO implement ping time

  //  log::info!("Tank calc_dir dir:{:?} old_dir:{:?} rot_speed:{:?} delta_time:{:?} delta:{:?} new_dir:{:?}",
  //      dir, old_dir, rot_speed, delta_time, delta, new_dir );

    normalize(new_dir)
}

pub fn normalize(mut dir: f32) -> f32 {
    if dir.abs() > std::f32::consts::PI {
        dir -= std::f32::consts::TAU;
    }

    if dir.abs() < -std::f32::consts::PI {
        dir += std::f32::consts::TAU;
    }

    dir
}


pub fn update_turret_rotation(
    time: Res<Time>,
    ping: Res<PingList>,
    local_handles: Res<LocalHandles>,
    mut query: Query<(&mut Transform, &TankControlTurret, &PlayerData)>,
    mut out_data: ResMut<TurretMessage>,
) {
    for (mut transform, tank_control_turret, player) in query.iter_mut() {

        let rot_speed = 0.5 * PI * tank_control_turret.speed;

        let old_dir = transform.rotation.to_euler(EulerRot::YXZ).0;

        let new_dir = if *local_handles.handles.first().unwrap() != player.handle {
            calc_dir(
                tank_control_turret.dir,
                old_dir,
                rot_speed,
                ping.get_time(player.handle),
            ) //*1.*time.delta_seconds();
        } else {
            let tmp_dir = normalize(old_dir + rot_speed * time.delta_seconds());

            if (out_data.speed - rot_speed).abs() * 180. / PI >= 1.
                || (out_data.dir - tmp_dir).abs() * 180. / PI >= 1.
            {
                out_data.speed = rot_speed;
                out_data.dir = tmp_dir;
  //              log::info!( "Tank turret out speed:{:?} dir:{:?}", out_data.speed, out_data.dir);
            }

            tmp_dir
        };

        if new_dir != old_dir {
            transform.rotation = Quat::from_axis_angle(Vec3::Y, new_dir);
        }
    }
}

pub fn update_cannon_rotation(
    time: Res<Time>,
    ping: Res<PingList>,
    local_handles: Res<LocalHandles>,
    mut query: Query<(&mut Transform, &TankControlCannon, &PlayerData)>,
    mut out_data: ResMut<TankCannonOutData>,
) {
    for (mut transform, tank_control_cannon, player) in query.iter_mut() {

        let rot_speed = 0.1 * PI * tank_control_cannon.speed;

        let old_dir = transform.rotation.to_euler(EulerRot::XYZ).0;

        let mut new_dir = if *local_handles.handles.first().unwrap() != player.handle {
            calc_dir(
                tank_control_cannon.dir,
                old_dir,
                rot_speed,
                ping.get_time(player.handle),
            ) 
        } else {
            let tmp_dir = normalize(old_dir + rot_speed * time.delta_seconds());

            if (out_data.speed - rot_speed).abs() * 180. / PI >= 1.
                || (out_data.dir - old_dir).abs() * 180. / PI >= 1.
            {
                out_data.speed = rot_speed;
                out_data.dir = tmp_dir;
 //               log::info!( "Tank cannon out speed:{:?} dir:{:?}", out_data.speed, out_data.dir );
            }

            tmp_dir
        };

        if new_dir < -0.7 {
            new_dir = -0.7;
        }

        if new_dir > 0.7 {
            new_dir = 0.7;
        }

        //         dbg![cross, dot, dot3, move_dir.angle_between(Game_transform.forward())];

        if new_dir != old_dir {
            transform.rotation = Quat::from_axis_angle(Vec3::X, new_dir);
        }
    }
}

pub fn update_cannon_debug_line(
    mut lines: ResMut<DebugLines>,
    query: Query<(&GlobalTransform, &TankShotData, &TankControlActionShot)>,
) {
    for (global_transform, shot_data, shot_action) in query.iter() {
        //    if let Ok((global_transform, cannon_shot_data)) = query.get_single() {
        let shot_speed = shot_data.shot_speed_delta * shot_action.time + shot_data.shot_speed_min;

        let mut pos = global_transform.translation();
        let mut dir = global_transform.back() * shot_speed;
        let delta_time = 0.05;
        let delta_y = -9.81 * delta_time;

        while pos.y > -10. {
            lines.line_colored(pos, pos + dir * delta_time, 0.0, Color::GREEN);

            pos += dir * delta_time;

            dir = Vec3::new(dir.x, dir.y + delta_y, dir.z);
        }
    }
}

pub fn update_cannon_shot(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    local_handles: Res<LocalHandles>,
    mut query: Query<(
        &GlobalTransform,
        &TankShotData,
        &mut TankControlActionShot,
        &PlayerData,
    )>,
    mut shot_control: ResMut<TankShotOutData>,
    ping: Res<PingList>,
) {
    let mut shot_pos;
    let mut shot_vel;

    for (global_transform, shot_data, mut shot_action, player) in query.iter_mut() {
        if *local_handles.handles.first().unwrap() == player.handle {
            if !shot_action.is_shot {
                if shot_control.is_shot {
                    shot_control.is_shot = false;
                }
                continue;
            }

            let shot_speed =
                shot_data.shot_speed_min + shot_data.shot_speed_delta * shot_action.time;
            //           dbg![shot_speed, global_transform];

            shot_pos = global_transform.translation();
            shot_vel = global_transform.back() * shot_speed;

            shot_control.is_shot = true;
            shot_control.pos = shot_pos;
            shot_control.vel = shot_vel;
        } else {
            if !shot_action.is_shot {
                continue;
            }

            //TODO add compensation of ping: delta pos:shot_action.vel*ping.get_time(player.handle)
            //apply gravity to velosity
            shot_pos = shot_action.pos + shot_action.vel*ping.get_time(player.handle);
            shot_vel = shot_action.vel - Vec3::Y*9.8*ping.get_time(player.handle);
            //         cannon_shot_data.is_shot = false;
        }

        shot_action.is_shot = false;

        commands
            .spawn_bundle(PbrBundle {
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

                transform: Transform::from_translation(shot_pos),

                ..default()
            })
            .insert(ShotData::new(shot_data.shot_live_max_time, shot_data.explosion_force))
            .insert(player.clone())
            .insert(bevy_rapier3d::prelude::RigidBody::Dynamic)
            .insert(bevy_rapier3d::prelude::Collider::ball(0.02))
            //                .insert_bundle(collider)
            .insert(bevy_rapier3d::prelude::ActiveEvents::COLLISION_EVENTS)
            .insert(Restitution::coefficient(0.01))
            .insert(Friction::coefficient(1.0))
            .insert(ColliderMassProperties::Density(5.))
            .insert(Ccd::enabled())
            .insert(Velocity {
                linvel: shot_vel,
                angvel: Vec3::ZERO,
            })
            .insert(CollisionGroups::new(0b0100, 0b0011))
            .insert(SolverGroups::new(0b0100, 0b0011))
            .insert(bevy_rapier3d::prelude::ActiveHooks::FILTER_CONTACT_PAIRS)
//          .insert(CustomFilterTag::GroupShot)
            ;
    }
}


fn obr_in_body(
    mut input: ResMut<InBody>, 
    mut query: Query<(&mut TankControlBody, &PlayerData)>
) {
    for (mut body, player) in query.iter_mut() {
        if let Some(data) = input.data.get(&player.handle) {
            body.movement.x = data.movement.x;
            body.movement.y = data.movement.y;

            body.pos.x = data.pos.x;
            body.pos.y = data.pos.y;

            body.dir = data.dir;
        }
    }

    input.data.clear();
}

fn obr_in_turret(
    mut input: ResMut<InTurret>,
    mut query: Query<(&mut TankControlTurret, &PlayerData)>,
) {
    for (mut turret, player) in query.iter_mut() {
        if let Some(data) = input.data.get(&player.handle) {
            turret.speed = data.speed;
            turret.dir = data.dir;
            log::info!("game obr_in_turret in speed:{:?} dir:{:?}", turret.speed, turret.dir);
        }
    }

    input.data.clear();
}

fn obr_in_cannon(
    mut input: ResMut<InCannon>,
    mut query: Query<(&mut TankControlCannon, &PlayerData)>,
) {
    for (mut cannon, player) in query.iter_mut() {
        if let Some(data) = input.data.get(&player.handle) {
            cannon.speed = data.speed;
            cannon.dir = data.dir;
            log::info!("game obr_in_cannon in speed:{:?} dir:{:?}", cannon.speed, cannon.dir);
        }
    }

    input.data.clear();
}

