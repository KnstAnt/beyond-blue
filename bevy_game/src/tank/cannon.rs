use std::f32::consts::PI;

use bevy::prelude::Component;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::utils::*;
use crate::game::*;
use crate::network::PingList;
//use crate::network::PingList;
use crate::player::{ControlCannon, PlayerData};

#[repr(C)]
#[derive(Serialize, Deserialize, Component, Debug, Default, Clone, Copy, PartialEq)]
pub struct Data {
    pub speed: f32,
    pub angle: f32,
}

pub fn update_cannon_rotation_from_net(
    time: Res<Time>,
    ping: Res<PingList>,
    mut query: Query<(&mut Transform, &MesState<Data>, &PlayerData)>,
) {
    for (mut transform, state, player) in query.iter_mut() {
        let data = state.data;
        let old_angle = transform.rotation.to_euler(EulerRot::XYZ).0;

        let mut new_angle = if data.speed != 0. {
            let delta_time = (time.seconds_since_startup() - state.time) as f32 + ping.get_time(player.handle)*0.5;
            normalize_angle(data.angle + data.speed * delta_time)
        } else {
            data.angle
        };


/*        let mut new_angle = calc_angle(
            data.angle,
            old_angle,
            data.speed,
            ping.get_time(player.handle),
        );
*/
        if new_angle < -0.7 {
            new_angle = -0.7;
        }

        if new_angle > 0.7 {
            new_angle = 0.7;
        }

        //         dbg![cross, dot, dot3, move_angle.angle_between(Game_transform.forward())];

        if new_angle != old_angle {
            transform.rotation = Quat::from_axis_angle(Vec3::X, new_angle);
        }
    }
}

pub fn update_player_cannon_rotation(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &ControlCannon)>,
    mut out_data_state: ResMut<OutMessageState<Data>>,
    mut output: ResMut<OutGameMessages<GameMessage>>,
) {
    if query.is_empty() {
        return;
    }
    
    out_data_state.delta_time += time.delta_seconds();

    let (mut transform, control) = query.single_mut();

    let is_moved = control.speed.abs() > ANGLE_SPEED_EPSILON;
    let is_changed = (control.speed - out_data_state.old_data.speed).abs() > OUT_ANGLE_EPSILON;
    let is_started_or_stoped = is_changed && (control.speed.abs() < ANGLE_SPEED_EPSILON || out_data_state.old_data.speed.abs() < ANGLE_SPEED_EPSILON);

    let mut rotation = 0.;

    if is_moved {
        rotation = control.speed;
    } 

    let rot_speed = 0.3 * PI * rotation;
    let old_angle = transform.rotation.to_euler(EulerRot::XYZ).0;
    let new_angle = normalize_angle(old_angle + rot_speed * time.delta_seconds()).max(-0.7).min(0.7);

    transform.rotation = Quat::from_axis_angle(Vec3::X, new_angle);

    if (is_changed && out_data_state.delta_time >= MIN_OUT_DELTA_TIME) || 
        (is_moved && out_data_state.delta_time >= MAX_OUT_DELTA_TIME) ||
        is_started_or_stoped {
        out_data_state.old_data.speed = rot_speed;
        out_data_state.old_data.angle = new_angle;

        output.data.push(GameMessage::from(out_data_state.old_data));
        out_data_state.delta_time = 0.;
    }
}

/* 
pub fn update_cannon_debug_line(
    mut lines: ResMut<DebugLines>,
    query: Query<(&GlobalTransform, &TankShotData, &ActionShot)>,
) {
    for (global_transform, shot_data, shot_action) in query.iter() {
        //    if let Ok((global_transform, cannon_shot_data)) = query.get_single() {
        let shot_speed = shot_data.shot_speed_delta * shot_action.time + shot_data.shot_speed_min;

        let mut pos = global_transform.translation();
        let mut angle = global_transform.back() * shot_speed;
        let delta_time = 0.05;
        let delta_y = -9.81 * delta_time;

        while pos.y > -10. {
            lines.line_colored(pos, pos + angle * delta_time, 0.0, Color::GREEN);

            pos += angle * delta_time;

            angle = Vec3::new(angle.x, angle.y + delta_y, angle.z);
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
        &mut ControlFire,
        &PlayerData,
    )>,
    mut shot_control: ResMut<ActionShot>,
    ping: Res<PingList>,
) {
    let mut shot_pos;
    let mut shot_vel;

    for (global_transform, shot_data, mut control, player) in query.iter_mut() {
        if *local_handles.handles.first().unwrap() == player.handle {
            if !control.is_shot {
                if shot_control.is_shot {
                    shot_control.is_shot = false;
                }
                continue;
            }

            let shot_speed =
                shot_data.shot_speed_min + shot_data.shot_speed_delta * control.time;
            //           dbg![shot_speed, global_transform];

            shot_pos = global_transform.translation();
            shot_vel = global_transform.back() * shot_speed;

            shot_control.is_shot = true;
            shot_control.pos = shot_pos;
            shot_control.vel = shot_vel;
        } else {
            if !control.is_shot {
                continue;
            }

            //TODO add compensation of ping: delta pos:shot_action.vel*ping.get_time(player.handle)
            //apply gravity to velosity
            shot_pos = control.pos + control.vel * ping.get_time(player.handle);
            shot_vel = control.vel - Vec3::Y * 9.8 * ping.get_time(player.handle);
            //         cannon_shot_data.is_shot = false;
        }

        control.is_shot = false;

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
*/