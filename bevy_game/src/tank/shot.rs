use bevy::prelude::shape::UVSphere;
use bevy::prelude::*;
use bevy_prototype_debug_lines::DebugLines;
use bevy_rapier3d::prelude::*;

use super::TankShotData;
use crate::game::{GameMessage, OutGameMessages};
use crate::player::{ControlFire, LocalHandles, PlayerData};
use crate::shot::{ShotData, ShotExplosionData};
//use crate::shot::Data;

pub fn update_cannon_debug_line(
    mut lines: ResMut<DebugLines>,
    query: Query<(&GlobalTransform, &TankShotData, &ControlFire)>,
) {
    for (global_transform, data, control) in query.iter() {
        //    if let Ok((global_transform, cannon_shot_data)) = query.get_single() {
        let shot_speed = data.shot_speed(control.time);
        let mut pos = global_transform.translation();
        let mut dir = global_transform.forward() * shot_speed;
        let delta_time = 0.01;
        let delta_y = -9.81 * delta_time;

        dir = Vec3::new(dir.x, dir.y + delta_y*0.5, dir.z);

        while pos.y > -10. {
            lines.line_colored(pos, pos + dir * delta_time, 0.0, Color::GREEN);

            pos += dir * delta_time;

            dir = Vec3::new(dir.x, dir.y + delta_y, dir.z);
        }
    }
}

pub fn create_player_cannon_shot(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    local_handles: Res<LocalHandles>,
    mut query: Query<(&GlobalTransform, &TankShotData, &mut ControlFire)>,
    mut output: ResMut<OutGameMessages<GameMessage>>,
//    mut shot_control: ResMut<ShotData>,
) {
    if query.is_empty() {
        return;
    }

    let (global_transform, data, mut control) = query.single_mut();

    if !control.is_shot {
        return;
    }

    control.is_shot = false;

    let shot_speed = data.shot_speed(control.time);
    let shot_pos = global_transform.translation();
    let shot_vel = global_transform.forward() * shot_speed;

    let out_data = ShotData{is_shot: true, pos: shot_pos, vel: shot_vel};

    output.data.push(GameMessage::from(out_data));

    commands
            .spawn_bundle(PbrBundle {
                mesh: meshes.add(Mesh::from(UVSphere {
                    radius: data.radius,
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
            .insert(ShotExplosionData::new(data.shot_live_max_time, data.explosion_force))
            .insert(PlayerData {handle: *local_handles.handles.first().unwrap()})
            .insert(bevy_rapier3d::prelude::RigidBody::Dynamic)
            .insert(bevy_rapier3d::prelude::Collider::ball(data.radius))
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
