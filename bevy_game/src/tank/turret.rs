use bevy::prelude::Component;
use serde::{Deserialize, Serialize};

use bevy::prelude::*;
use std::f32::consts::PI;

use super::utils::*;
use crate::game::{GameMessage, MesState, OutGameMessages, OutMessageState, MAX_OUT_DELTA_TIME, MIN_OUT_DELTA_TIME, OUT_ANGLE_EPSILON, SPEED_EPSILON};
use crate::network::PingList;
//use crate::network::PingList;
use crate::player::{ControlTurret, PlayerData};

#[repr(C)]
#[derive(Serialize, Deserialize, Component, Debug, Default, Clone, Copy, PartialEq)]
pub struct Data {
    pub speed: f32,
    pub dir: f32,
}

pub fn update_turret_rotation_from_net(
    time: Res<Time>,
    ping: Res<PingList>,
    mut query: Query<(&mut Transform, &MesState<Data>, &PlayerData)>,
) {
    for (mut transform, state, player) in query.iter_mut() {
        let data = state.data;
        let old_dir = transform.rotation.to_euler(EulerRot::YXZ).0;

        let target_dir = if data.speed != 0. {
            let delta_time = (time.seconds_since_startup() - state.time) as f32 + ping.get_time(player.handle)*0.5;
            normalize(data.dir + data.speed * delta_time)
        } else {
            data.dir
        }; 

        let new_dir =  calc_dir(
                target_dir,
                old_dir,
                data.speed,
                time.delta_seconds(),
            ) 
            ;


        if new_dir != old_dir {
            transform.rotation = Quat::from_axis_angle(Vec3::Y, new_dir);
        }
    }
}

pub fn update_player_turret_rotation(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &ControlTurret)>,
    mut out_data_state: ResMut<OutMessageState<Data>>,
    mut output: ResMut<OutGameMessages<GameMessage>>,
) {
    if query.is_empty() {
        return;
    }

    out_data_state.delta_time += time.delta_seconds();

    let (mut transform, control) = query.single_mut();

    let is_moved = control.speed.abs() > SPEED_EPSILON;
    let is_changed = (control.speed - out_data_state.old_data.speed).abs() > OUT_ANGLE_EPSILON;
    let is_started_or_stoped = is_changed && (control.speed.abs() < SPEED_EPSILON || out_data_state.old_data.speed.abs() < SPEED_EPSILON);

    let mut rotation = 0.;

    if is_moved {
        rotation = control.speed;
    } 

    let rot_speed = 0.5 * PI * rotation;
    let old_dir = transform.rotation.to_euler(EulerRot::YXZ).0;
    let new_dir = normalize(old_dir + rot_speed * time.delta_seconds());
    transform.rotation = Quat::from_axis_angle(Vec3::Y, new_dir);

    if (is_changed && out_data_state.delta_time >= MIN_OUT_DELTA_TIME) || 
        (is_moved && out_data_state.delta_time >= MAX_OUT_DELTA_TIME) || 
        is_started_or_stoped {
        out_data_state.old_data.speed = rot_speed;
        out_data_state.old_data.dir = new_dir;

        output.data.push(GameMessage::from(out_data_state.old_data));
        out_data_state.delta_time = 0.;
    }
}