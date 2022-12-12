use std::f32::consts::PI;

use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use serde::{Deserialize, Serialize};

//use super::{TankLastPos, TankMoveTarget};
use crate::game::*;
use crate::network::PingList;
use crate::player::{ControlMove, PlayerData};
use crate::tank::{TankEntityes, WheelData};
use crate::utils::*;


const START_DELAY: f32 = 0.5;
const STOP_DELAY: f32 = 0.3;
const WHEEL_SPEED_MAX: f32 = 10.;
//pub const LINEAR_SPEED_MAX: f32 = 1.;
//pub const ANGULAR_SPEED_MAX: f32 = PI / 2.0;


#[repr(C)]
#[derive(Serialize, Deserialize, Component, Debug, Default, Clone, Copy, PartialEq)]
pub struct Data {
    pub movement: Vec2,
    pub delta_time_linear: u16,
    pub delta_time_angular: u16,
    pub pos: Vec2,
    pub angle: f32,
    pub linvel: Vec2,
    pub angvel: f32,
}

impl Data {
    pub fn is_moved(&self) -> bool {
        return self.movement.length_squared() > VEL_EPSILON_QRT 
               || self.linvel.length_squared() > VEL_EPSILON_QRT 
               || self.angvel.abs() > ANGLE_SPEED_EPSILON
    }

    pub fn set_delta_time_linear(&mut self, delta: f32) {
        self.delta_time_linear = (delta.min(10.) * 1000.) as u16;
    }
    pub fn set_delta_time_angular(&mut self, delta: f32) {
        self.delta_time_angular = (delta.min(10.) * 1000.) as u16;
    }

    pub fn get_delta_time_linear(&self) -> f32 {
        (self.delta_time_linear as f32) / 1000. 
    }
    pub fn get_delta_time_angular(&self) -> f32 {
        (self.delta_time_angular as f32) / 1000. 
    }
}

pub fn update_body_position_from_net(
    mut commands: Commands,
    time: Res<Time>,
    ping: Res<PingList>,
    mut data_query: Query<(
        &GlobalTransform,
        &mut Velocity,
        //       ChangeTrackers<State<Message<Data>>>,
        &MesState<Data>,
        //      &mut ExternalImpulse,
        &mut ExternalForce,
        &mut Sleeping,
        &TankEntityes,
        &PlayerData,
    )>,
    mut wheel_data_query: Query<&mut WheelData>,
//    mut tank_move_target_query: Query<&mut Transform, (With<TankMoveTarget>, Without<TankLastPos>)>,
//    mut tank_last_pos_query: Query<&mut Transform, (With<TankLastPos>, Without<TankMoveTarget>)>,
) {
    for (
        global_transform,
        _vel,
        state,
        /*tank_control_data, mut forces, mut impulse,*/
        mut _force,
        mut sleeping,
        entityes,
        player,
    ) in data_query.iter_mut()
    {
        let (_scale, rotation, translation) = global_transform.to_scale_rotation_translation();
        let data = state.data;

  /*       if let Ok(mut target_transform) = tank_last_pos_query.get_single_mut() {
            target_transform.translation =
                Vec3::new(state.data.pos.x, translation.y, state.data.pos.y);
        }
*/
        let mut movement = Vec2::ZERO;

        // if stop only correct pos and angle
        if data.movement.length_squared() < VEL_EPSILON_QRT {
            let mut net_pos = v2_3(data.pos + data.linvel * 0.15);
            net_pos.y = translation.y;
            let delta_pos = net_pos - translation;

            let net_angle = data.angle + data.angvel * 0.15;
            let delta_angle = delta_angle(net_angle, get_angle_y(&rotation));

            if delta_pos.length_squared() > POS_EPSILON_QRT 
                || delta_angle.abs() > ANGLE_EPSILON {

                let shift_data = crate::tank::TankShift {
                    rotation: delta_angle,
                    velosity: delta_pos,
                    time: 1.,
                };
    
                commands.entity(entityes.body).insert(shift_data.clone());
            } else {
                commands
                    .entity(entityes.body)
                    .remove::<crate::tank::TankShift>();
            }
        } else {
            //try to compensate the ping delay 
            let state_delta_time_data = (time.seconds_since_startup() - state.time) as f32;// + ping.get_time(player.handle) * 0.7;
            let ping_time = ping.get_time(player.handle);
            let delta_time_linear = data.get_delta_time_linear() + state_delta_time_data + ping_time;
            let delta_time_angular = data.get_delta_time_angular() + state_delta_time_data + ping_time;//).min(START_DELAY);
            let max_time_delay = (START_DELAY - ping_time).max(0.01);

            let move_y = data.movement.y
            * delta_time_linear.min(max_time_delay)
            * WHEEL_SPEED_MAX
            / max_time_delay;

            let move_x = data.movement.x
                * delta_time_angular.min(max_time_delay)
                * WHEEL_SPEED_MAX
                / max_time_delay;

            movement = Vec2::new(move_x, move_y);


            if (data.movement.y.abs() > VEL_EPSILON && data.get_delta_time_linear() >= START_DELAY) 
            || (data.movement.x.abs() > VEL_EPSILON && data.get_delta_time_angular() >= START_DELAY) {
                let mut net_pos = v2_3(data.pos + data.linvel * ping_time);
                net_pos.y = translation.y;
                let delta_pos = net_pos - translation;

                let net_angle = data.angle + data.angvel * ping_time;
                let delta_angle = delta_angle(net_angle, get_angle_y(&rotation));

                if delta_pos.length_squared() > POS_EPSILON_QRT 
                    || delta_angle.abs() > ANGLE_EPSILON {

                    let shift_data = crate::tank::TankShift {
                        rotation: delta_angle,
                        velosity: delta_pos,
                        time: 1.,
                    };
        
                    commands.entity(entityes.body).insert(shift_data.clone());
                } else {
                    commands
                        .entity(entityes.body)
                        .remove::<crate::tank::TankShift>();
                }

                /*if let Ok(mut target_transform) = tank_move_target_query.get_single_mut() {
                    target_transform.translation =
                        Vec3::new(current_net_pos.x, translation.y, current_net_pos.y);
                }*/
            }

        }


/*
        if delta_time_linear > START_DELAY {
            let delta_pos_delay = 
        }

        if delta_time_angular > START_DELAY {

        }     

        let linear_speed1 = -data.movement.x * ANGULAR_SPEED_MAX;
        let linear_speed2 = -data.movement.x * ANGULAR_SPEED_MAX;

        let rotation_speed1 = -data.movement.x * ANGULAR_SPEED_MAX;
        let rotation_speed2 = -data.movement.x * ANGULAR_SPEED_MAX;

        let extr_angle = normalize_angle(data.angle + rotation_speed * delta_time);

        let extr_dir = Quat::from_axis_angle(Vec3::Y, extr_angle);

        let current_net_pos = data.pos + data.vel * delta_time;

        let extr_time = 2.0;

        let mut extr_net_pos = v2_3(current_net_pos + data.vel * extr_time);
        extr_net_pos.y = translation.y;

        let delta_pos = extr_net_pos - translation;

        let local_delta_pos = dir_to_local(&extr_dir, &delta_pos);

        let delta_vel = v2_3(data.vel) - vel.linvel;
        let local_delta_vel = dir_to_local(&extr_dir, &delta_vel);

        if delta_pos
            .dot(Transform::from_rotation(extr_dir).forward())
            .abs()
            > 0.8
        {
            let y_coorection_value = 1. * local_delta_pos.z;

            force.force = 15.
                * global_transform.back()
                * y_coorection_value.abs().min(2.)
                * y_coorection_value.signum()
                * time.delta_seconds();

            movement.y += 0.5 * y_coorection_value.abs().min(0.5) * y_coorection_value.signum();
        }

        //        movement.y = 0.;

        //      let extr_angle = data.angle - data.movement.x * 0.03 * delta_time;

        let delta_angle = delta_angle(extr_angle, get_angle_y(&rotation));

        let vel_x_value = -0.4 * local_delta_vel.x.abs().min(0.4) * local_delta_vel.x.signum();

        let pos_x_value = -2.0 * local_delta_pos.x.abs().min(0.2) * local_delta_pos.x.signum();

        let target_angle = normalize_angle(extr_angle + vel_x_value + pos_x_value);

        log::info!(
            "extr_angle:{}, angle_value:{}, vel_value:{}, pos_value:{}, target_angle:{}",
            extr_angle,
            delta_angle,
            vel_x_value,
            pos_x_value,
            target_angle
        );

        movement.x -= vel_x_value + pos_x_value + delta_angle; // + mov_value;

        if local_delta_pos.x.abs() > POS_EPSILON_QRT {
            let correction_pos = if movement.y != 0. {
                Vec3::new(local_delta_pos.x, 0., local_delta_pos.y / 3.)
            } else {
                Vec3::new(local_delta_pos.x, 0., local_delta_pos.y)
            };

            let correction_pos = dir_to_global(&extr_dir, &correction_pos);

            let correction_rot = if movement.x != 0. {
                delta_angle / 3.
            } else {
                delta_angle
            };

            let shift_data = crate::tank::TankShift {
                rotation: correction_rot / extr_time,
                velosity: correction_pos / extr_time,
                time: 1.,
            };

            commands.entity(entityes.body).insert(shift_data.clone());
        } else {
            commands
                .entity(entityes.body)
                .remove::<crate::tank::TankShift>();
        }

        if let Ok(mut target_transform) = tank_move_target_query.get_single_mut() {
            target_transform.translation =
                Vec3::new(current_net_pos.x, translation.y, current_net_pos.y);
        }
*/
       /*
                net_vel 0 current_vel -1  local 1    mult  1

        data.vel -2.4935524 current_vel -1.7 delta_vel: 0.7641309 local_delta_vel:0.88171524
         vel_mult:0.25

         delta_pos: 1.4875035
         extr_net_pos.x:-5.74324 translation.x:-7.2307434 local_delta_pos.x:3.3352304
         pos_mult:0.09,

         data.movement.x:0
         mov_mult:-0

        data.angle:0.0059203263  calculated_angle:0.34592032 current:0.35462776


        data.angle:-0.48985457

        extr_net_pos.x:11.34357 translation.x:9.625186 local_delta_pos.x:1.7202746 data.movement.x:0 vel_mult:0.25, pos_mult:-0.09, mov_mult:-0, angle:-0.32985458 current:0.0005708073
         data.vel: 0.50456065  delta_vel:0.5120836  local_delta_vel:0.5126514
         delta_pos: 1.7183838 local_delta_pos:1.7202746  angle:-0.32985458

         old_angle: 0.0005708073  target_angle: -0.32985458 delta_angle: -0.33042538  torque: -27.583492
        */
        //        log::info!("update_body_position delta_pos: {}; tmp_impulse: {}; current_dir: {}; from_net.dir: {}; torque: {}", delta_pos, tmp_impulse, current_body_dir, data.angle, torque);

        let wheel_data_movement = if movement.length_squared() > 0.1 {
            sleeping.linear_threshold = 2.;
            sleeping.angular_threshold = 10.;
            sleeping.sleeping = false;

            Some(movement)
        } else {
            sleeping.linear_threshold = 2.;
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
        //       }
    }
}

//apply player control
pub fn update_player_body_control(
    //    local_handles: Res<LocalHandles>,
    time: Res<Time>,
    mut query: Query<(
        &GlobalTransform,
        &Velocity,
        //       ChangeTrackers<PlayerControlMove>,
        &ControlMove,
        //        &mut ExternalImpulse,
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
        vel,
        //     tracker,
        control,
        /*tank_control_data, mut forces, mut impulse,*/
        mut sleeping,
        entityes,
    ) = query.single_mut();

    let (_scale, rotation, translation) = global_transform.to_scale_rotation_translation();
    let new_pos = Vec2::new(translation.x, translation.z);
    let new_dir = get_angle_y(&rotation);
    let new_linvel = Vec2::new(vel.linvel.x, vel.linvel.z);
    let new_angvel = vel.angvel.y;

    let delta_time_linear = (time.seconds_since_startup() - control.time_linear) as f32;
    let delta_time_angular = (time.seconds_since_startup() - control.time_angular) as f32;

    let move_y = control.movement.y
        * delta_time_linear.min(START_DELAY)
        * WHEEL_SPEED_MAX
        / START_DELAY;

    let move_x = control.movement.x
        * delta_time_angular.min(START_DELAY)
        * WHEEL_SPEED_MAX
        / START_DELAY;

    let wheel_data_movement = if move_y.abs() > VEL_EPSILON || move_x.abs() > VEL_EPSILON {
        Some(Vec2::new(move_x, move_y))
    } else {
        None
    };

    let is_moved = wheel_data_movement.is_some()
        || vel.linvel.length_squared() >= VEL_EPSILON_QRT
        || vel.angvel.length_squared() >= VEL_EPSILON_QRT;

    let is_changed = normalize_angle(new_dir - out_data_state.old_data.angle).abs() > OUT_ANGLE_EPSILON
        || (new_pos - out_data_state.old_data.pos).length_squared() > POS_EPSILON_QRT
        || (new_linvel - out_data_state.old_data.linvel).length_squared() > VEL_EPSILON_QRT
        || (new_angvel - out_data_state.old_data.angvel).abs() > ANGLE_SPEED_EPSILON;

    let is_started_or_stoped = control.movement.x != out_data_state.old_data.movement.x
        || control.movement.y != out_data_state.old_data.movement.y;

    if (is_changed && out_data_state.delta_time >= MIN_OUT_DELTA_TIME)
        || (is_moved && out_data_state.delta_time >= MAX_OUT_DELTA_TIME)
        || is_started_or_stoped
    {
        out_data_state.old_data.movement = control.movement;
        out_data_state.old_data.set_delta_time_linear(delta_time_linear);
        out_data_state.old_data.set_delta_time_angular(delta_time_angular);

        out_data_state.old_data.pos = new_pos;
        out_data_state.old_data.angle = new_dir;
        out_data_state.old_data.linvel = Vec2::new(vel.linvel.x, vel.linvel.z);
        out_data_state.old_data.angvel = vel.angvel.y;

        output.data.push(GameMessage::from(out_data_state.old_data));
        out_data_state.delta_time = 0.;
    }

    if is_started_or_stoped {
        if control.movement.length_squared() > 0.001 {
            sleeping.linear_threshold = -1.;
            sleeping.angular_threshold = -1.;
            sleeping.sleeping = false;
        } else {
            sleeping.linear_threshold = 1.;
            sleeping.angular_threshold = 10.;
            sleeping.sleeping = true;
        }
    }
    //  dbg![control.time_linear, control.time_angular, move_x, move_y];

    for wheel in &entityes.wheels {
        if let Ok(mut wheel_data) = wheel_data_query.get_component_mut::<WheelData>(*wheel) {
            if wheel_data.movement != wheel_data_movement {
                if wheel_data_movement.is_some() {
                    wheel_data.movement = wheel_data_movement.clone();
                } else if let Some(mut movement) = wheel_data.movement {
                    if movement.length_squared() > VEL_EPSILON_QRT {
                        movement = movement * (1. - time.delta_seconds()/STOP_DELAY);                        
                        wheel_data.movement = Some(movement);
                    } else {
                        wheel_data.movement = wheel_data_movement;
                    }
                }
                //           println!("player prep_wheel_input, ok");
            }
        }
    }
}
