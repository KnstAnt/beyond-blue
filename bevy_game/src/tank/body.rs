use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use serde::{Deserialize, Serialize};
use std::f32::consts::PI;

use crate::game::{GameMessage, MesState, OutGameMessages, OutMessageState, MAX_OUT_DELTA_TIME};
use crate::player::{ControlMove as PlayerControlMove, LocalHandles, PlayerData};
use crate::tank::{TankEntityes, WheelData};

use super::utils::*;

#[repr(C)]
#[derive(Serialize, Deserialize, Component, Debug, Default, Clone, Copy, PartialEq)]
pub struct Data {
    pub movement: Vec2,
    pub pos: Vec2,
    pub angle: f32,
}

pub fn update_body_position_from_net(
    time: Res<Time>,
    mut data_query: Query<(
        &GlobalTransform,
        //       ChangeTrackers<State<Message<Data>>>,
        &MesState<Data>,
        &mut ExternalImpulse,
        &mut Sleeping,
        &TankEntityes,
    )>,
    mut wheel_data_query: Query<&mut WheelData>,
) {
    for (
        global_transform,
        state,
        /*tank_control_data, mut forces,*/ mut impulse,
        mut sleeping,
        entityes,
    ) in data_query.iter_mut()
    {
        let (_scale, rotation, translation) = global_transform.to_scale_rotation_translation();
        let data = state.data;

        //correct body pos
        let delta_pos = Vec3::new(data.pos.x - translation.x, 0., data.pos.y - translation.z);

        //       log::info!("tank mod update_body_position translation.pos {} input.pos{} delta_pos{}",
        //           transform.translation, tank_control_body.pos, delta_pos);

        impulse.impulse = delta_pos * delta_pos.length_squared() * 100. * time.delta_seconds();

        /*           impulse.impulse = if delta_pos.length_squared() > 1. {
                        delta_pos.normalize_or_zero()
                    } else {
                        delta_pos
                    } * 10.;
        */
        let current_body_dir = rotation.to_euler(EulerRot::YXZ).0;
        let torque = calc_delta_dir(data.angle, current_body_dir, 30. * PI / 180.)
            * 10000.
            * time.delta_seconds();

        //       log::info!("tank mod update_body_position current_dir: {}; from_net.dir: {}; torque: {}",
        //       current_body_dir, tank_control_body.dir, torque);

        impulse.torque_impulse = rotation.mul_vec3(Vec3::Y * torque);

        if data.movement.x != 0. || data.movement.y != 0. {
            let wheel_data_movement = if data.movement.length_squared() > 0.001 {
                sleeping.linear_threshold = -1.;
                sleeping.angular_threshold = -1.;
                sleeping.sleeping = false;
                Some(data.movement.clone())
            } else {
                sleeping.linear_threshold = 1.;
                sleeping.angular_threshold = 10.;
                sleeping.sleeping = true;
                //          sleeping.default();
                None
            };

            for wheel in &entityes.wheels {
                if let Ok(mut wheel_data) = wheel_data_query.get_component_mut::<WheelData>(*wheel)
                {
                    wheel_data.movement = wheel_data_movement.clone();

                    //           println!("player prep_wheel_input, ok");
                }
            }
        }
    }
}

//apply player control
pub fn update_player_body_control(
//    local_handles: Res<LocalHandles>,
    time: Res<Time>,
    mut query: Query<(
        &GlobalTransform,
        //       ChangeTrackers<PlayerControlMove>,
        &PlayerControlMove,
        &mut ExternalImpulse,
        &mut Sleeping,
        &TankEntityes,
    )>,
    //        &mut ExternalForce,
    mut out_data_state: ResMut<OutMessageState<Data>>,
    mut output: ResMut<OutGameMessages<GameMessage>>,
    mut wheel_data_query: Query<&mut WheelData>,
) {
    if query.is_empty() {
        return;
    }
    
    out_data_state.delta_time += time.delta_seconds();

    let (
        global_transform,
        //     tracker,
        control,
        /*tank_control_data, mut forces,*/ mut impulse,
        mut sleeping,
        entityes,
    ) = query.single_mut();

    let (_scale, rotation, translation) = global_transform.to_scale_rotation_translation();
    let new_pos = Vec2::new(translation.x, translation.z);
    let new_dir = rotation.to_euler(EulerRot::YXZ).0;

    let is_moved = control.movement.x != 0. || control.movement.y != 0.;
    let is_changed = control.movement.x != out_data_state.old_data.movement.x
                        || control.movement.y != out_data_state.old_data.movement.y;

    if is_changed || (is_moved && out_data_state.delta_time >= MAX_OUT_DELTA_TIME) {
        out_data_state.old_data.movement = control.movement;
        out_data_state.old_data.pos = new_pos;
        out_data_state.old_data.angle = new_dir;

        output.data.push(GameMessage::from(out_data_state.old_data));
        out_data_state.delta_time = 0.;
    }

    if is_changed {
        let wheel_data_movement = if control.movement.length_squared() > 0.001 {
            sleeping.linear_threshold = -1.;
            sleeping.angular_threshold = -1.;
            sleeping.sleeping = false;
            Some(control.movement.clone())
        } else {
            sleeping.linear_threshold = 1.;
            sleeping.angular_threshold = 10.;
            sleeping.sleeping = true;
            //          sleeping.default();
            None
        };

        for wheel in &entityes.wheels {
            if let Ok(mut wheel_data) = wheel_data_query.get_component_mut::<WheelData>(*wheel) {
                wheel_data.movement = wheel_data_movement.clone();

                //           println!("player prep_wheel_input, ok");
            }
        }
    }
}
