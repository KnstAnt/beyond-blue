use std::f32::consts::PI;

use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use serde::{Deserialize, Serialize};

use crate::game::*;
use crate::network::PingList;
use crate::player::{ControlMove, PlayerData};
use crate::tank::{TankEntityes, WheelData};
use super::{TankMoveTarget, TankLastPos};
use crate::utils::*;


#[repr(C)]
#[derive(Serialize, Deserialize, Component, Debug, Default, Clone, Copy, PartialEq)]
pub struct Data {
    pub movement: Vec2,
    pub pos: Vec2,
    pub angle: f32,
    pub vel: Vec2,
}

impl Data {
    pub fn is_moved(&self) -> bool {
        return self.movement.x != 0. || 
                self.movement.y != 0. || 
                self.vel.x.abs() > VEL_EPSILON || 
                self.vel.y.abs() > VEL_EPSILON
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
    mut tank_move_target_query: Query<&mut Transform, (With<TankMoveTarget>, Without<TankLastPos>)>,
    mut tank_last_pos_query: Query<&mut Transform, (With<TankLastPos>, Without<TankMoveTarget>)>,
) {
    for (
        global_transform,
        vel,        
        state,
        /*tank_control_data, mut forces, mut impulse,*/        
        mut force,
        mut sleeping,
        entityes,
        player,
    ) in data_query.iter_mut()
    {
        let (_scale, rotation, translation) = global_transform.to_scale_rotation_translation();
        let data = state.data;

        if let Ok(mut target_transform) = tank_last_pos_query.get_single_mut() {
            target_transform.translation = Vec3::new(state.data.pos.x, translation.y, state.data.pos.y);
        }

        
        let mut movement = Vec2::new(data.movement.x, data.movement.y);

        //try to compensate the ping delay
        let (target_angle, target_pos) = if data.is_moved() {
            
            let delta_time = (time.seconds_since_startup() - state.time) as f32 + ping.get_time(player.handle)*0.7;

            let extr_angle = data.angle - data.movement.x * PI * delta_time / 1.5;

            let extr_dir   = Quat::from_axis_angle(Vec3::Y, extr_angle);

            let current_net_pos = data.pos + data.vel * delta_time;

            let extr_time = 2.0;

            let mut extr_net_pos = v2_3(current_net_pos + data.vel * extr_time);
            extr_net_pos.y = translation.y;

            let delta_pos = extr_net_pos - translation;

            

            let local_delta_pos = dir_to_local(&extr_dir, &delta_pos);

            let delta_vel = v2_3(data.vel) - vel.linvel;
            let local_delta_vel = dir_to_local(&extr_dir, &delta_vel);
           
            if delta_pos.dot(Transform::from_rotation(extr_dir).forward()).abs() > 0.8 {
                let y_coorection_value = -10.*local_delta_pos.z;//-10.*local_delta_vel.z;//  

   //             force.force = 300.*global_transform.forward()*y_coorection_value.abs().min(3.)*y_coorection_value.signum()*time.delta_seconds();

    //          movement.y = 0.5*y_coorection_value.abs().min(0.5)*y_coorection_value.signum();
            }

    //        movement.y = 0.;

     //      let extr_angle = data.angle - data.movement.x * 0.03 * delta_time;

            let angle_value = normalize(extr_angle - get_angle_y(&rotation));

            let vel_x_value = -0.4*local_delta_vel.x.abs().min(0.4)*local_delta_vel.x.signum();

            let pos_x_value = -2.0*local_delta_pos.x.abs().min(0.2)*local_delta_pos.x.signum();

            let angle = normalize(data.angle + vel_x_value + pos_x_value);

            log::info!("extr_angle:{}, angle_value:{}, vel_value:{}, pos_value:{}", 
            extr_angle, angle_value, vel_x_value, pos_x_value);
      
            movement.x -= vel_x_value + pos_x_value + angle_value;// + mov_value;

/* 
            let vel_y_value = 0.;//-0.5*local_delta_vel.y.abs().min(0.5)*local_delta_vel.y.signum();

            let pos_y_value = -10.0*local_delta_pos.y.abs().min(1.0)*local_delta_pos.y.signum();

            movement.y += pos_y_value + vel_y_value;
*/
  //          log::info!("data.angle:{} local_delta_vel.x:{} extr_net_pos.x:{} translation.x:{} local_delta_pos.x:{} data.movement.x:{} vel_mult:{}, pos_mult:{}, mov_mult:{}, angle:{} current:{}", 
    //        data.angle, local_delta_vel.x, extr_net_pos.x, translation.x, local_delta_pos.x, data.movement.x, vel_mult, pos_mult, mov_mult, angle, get_angle_y(&rotation));
            
            /*if local_dir.z.abs() > ANGLE_EPSILON {
                (local_dir.z.abs()*10. + 1.) * local_dir.z.signum() * global_transform.back()
            } else {
                Vec3::ZERO
            };
        
            let angle = if local_dir.x.abs() > ANGLE_EPSILON {
                normalize(data.angle - local_dir.x.abs().min(0.3)*local_dir.x.signum())
            } else {
                normalize(data.angle - data.movement.x*0.3*delta_time)
            };  */

    //        log::info!("n_vel:{} s_vel:{} d_vel:{} l_d_vel:{} e_n_pos:{} s_pos:{} d_pos:{} l_d_pos:{} force:{}", 
     //                   data.vel.y, vel.linvel.z, delta_vel.z, local_delta_vel.z, extr_net_pos.z, translation.z, delta_pos.z, local_delta_pos.z, force.force.z);

   //         log::info!("delta_time:{} data.vel:{} delta_vel:{} local_delta_vel:{} delta_pos:{} local_delta_pos:{} force.force:{} angle:{}", 
   //         delta_time, data.vel, delta_vel, local_delta_vel, delta_pos, local_delta_pos, force.force, angle);

            (angle, extr_net_pos)
        } else {
            //correct body pos
            let delta_pos = Vec3::new(data.pos.x - translation.x, 0., data.pos.y - translation.z);
            
            if delta_pos.length_squared() > POS_EPSILON_QRT {
                let shift_data = crate::tank::TankShift{
                    angle: get_angle_y(&global_transform.compute_transform()),
                    pos: Vec3::new(data.pos.x, translation.y, data.pos.y),
                    time: 2.,
                };

      //  !!!!!          commands.entity(entityes.body).insert(shift_data.clone());
            }

   //  !!!!!           force.force = Vec3::ZERO;

            let delta_angle = delta_angle(data.angle, get_angle_y(&rotation));
            let torque = delta_angle * 100. * time.delta_seconds();
            force.torque = rotation.mul_vec3(Vec3::Y * torque);

            (data.angle, Vec3::new(data.pos.x, translation.y, data.pos.y))
        }; 

        if let Ok(mut target_transform) = tank_move_target_query.get_single_mut() {
            target_transform.translation = target_pos;
        }

  //      let delta_angle = delta_angle(target_angle, get_angle_y(&rotation));
//        let torque = delta_angle * 100. * time.delta_seconds();
 //       force.torque = rotation.mul_vec3(Vec3::Y * torque);
  //      log::info!("update_body_position old_angle: {}  target_angle: {} delta_angle: {}  torque: {}", get_angle_y(&rotation), target_angle, delta_angle, torque);



 // !!!!       force.torque = rotation.mul_vec3(Vec3::Y * torque);

  /*       let delta_angle = delta_angle(target_angle, get_angle_y(&rotation)).min(0.4);
        if delta_angle.abs() > ANGLE_EPSILON {        
       //     let torque = (((1. + delta_angle.abs())*delta_angle.signum()).powf(2.0) - 1.) * 50.;
            let torque = delta_angle * 100. * time.delta_seconds();
            force.torque = rotation.mul_vec3(Vec3::Y * torque);
            log::info!("update_body_position old_angle: {}  target_angle: {} delta_angle: {}  torque: {}", get_angle_y(&rotation), target_angle, delta_angle, torque);

        } else {
            force.torque = Vec3::ZERO;
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
                if let Ok(mut wheel_data) = wheel_data_query.get_component_mut::<WheelData>(*wheel)
                {
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
    let new_vel = Vec2::new(vel.angvel.x, vel.angvel.z);

    let is_moved = control.movement.x != 0. || control.movement.y != 0. || 
                        vel.linvel.length_squared() >= VEL_EPSILON_QRT ||
                        vel.angvel.length_squared() >= VEL_EPSILON_QRT;

    let is_changed =  normalize(new_dir - out_data_state.old_data.angle).abs() > OUT_ANGLE_EPSILON ||
                            (new_pos - out_data_state.old_data.pos).length_squared() > POS_EPSILON_QRT ||
                            (new_vel - out_data_state.old_data.vel).length_squared() > VEL_EPSILON_QRT;

    let is_started_or_stoped = control.movement.x != out_data_state.old_data.movement.x ||
                                    control.movement.y != out_data_state.old_data.movement.y;

    if (is_changed && out_data_state.delta_time >= MIN_OUT_DELTA_TIME) || 
        (is_moved && out_data_state.delta_time >= MAX_OUT_DELTA_TIME) ||
        is_started_or_stoped {
        out_data_state.old_data.movement = control.movement;
        out_data_state.old_data.pos = new_pos;
        out_data_state.old_data.angle = new_dir;
        out_data_state.old_data.vel = Vec2::new(vel.linvel.x, vel.linvel.z);

        output.data.push(GameMessage::from(out_data_state.old_data));
        out_data_state.delta_time = 0.;
    }

    if is_started_or_stoped {
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
