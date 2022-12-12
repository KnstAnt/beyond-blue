use std::f32::consts::FRAC_PI_2;

use bevy::prelude::*;
use bevy_rapier3d::prelude::Velocity;

use crate::game::*;
use crate::utils::*;
use crate::AppState;

pub struct TestPlugin;

impl Plugin for TestPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(SystemSet::on_enter(AppState::Test).with_system(start_tests))
            .add_system_set(SystemSet::on_update(AppState::Test).with_system(process_tests));
    }
}

fn start_tests() {    
  //  calc_forse();
  test_utils_dir();
}

fn process_tests(mut app_state: ResMut<State<AppState>>) {
    app_state.replace(AppState::Menu).unwrap();
}

fn test_utils_dir() {
    let data = crate::tank::TankBodyData {
        movement: Vec2::new(0., 0.),
        delta_time_linear: 0,
        delta_time_angular: 0,
        pos: Vec2::new(1., 0.5),
        angle: 0.,
        linvel: Vec2::new(0., -1.),
        angvel: 0.,
    };

    let vel = Velocity {
        linvel: Vec3::new(1., 0., -0.5),
        angvel: Vec3::ZERO,
    };

    let transform0 = Transform::from_rotation(set_angle_y(0.)).with_translation(Vec3::new(0., 0., 0.0));
    let transformNZ = Transform::from_rotation(set_angle_y(0.)).with_translation(Vec3::new(0., 0., -10.0));
    let transformX = Transform::from_rotation(set_angle_y(0.)).with_translation(Vec3::new(10., 0., 0.0));

    let point_NZ = Vec3::new(0., 0., -10.0);
    let point_Z = Vec3::new(0., 0., 10.0);
    let point_NX = Vec3::new(-10., 0., 0.0);
    let point_X = Vec3::new(10., 0., 0.0);

    let res01 = dir_to_local(&transform0, &point_NZ);
    let res02 = dir_to_local(&transform0, &point_Z);
    let res03 = dir_to_local(&transform0, &point_NX);
    let res04 = dir_to_local(&transform0, &point_X);

    let resNZ1 = dir_to_local(&transformNZ, &point_NZ);
    let resNZ2 = dir_to_local(&transformNZ, &point_Z);
    let resNZ3 = dir_to_local(&transformNZ, &point_NX);
    let resNZ4 = dir_to_local(&transformNZ, &point_X);

    let resX1 = dir_to_local(&transformX, &point_NZ);
    let resX2 = dir_to_local(&transformX, &point_Z);
    let resX3 = dir_to_local(&transformX, &point_NX);
    let resX4 = dir_to_local(&transformX, &point_X);


    let transform0a = Transform::from_rotation(set_angle_y(FRAC_PI_2)).with_translation(Vec3::new(0., 0., 0.0));
    let transformNZa = Transform::from_rotation(set_angle_y(FRAC_PI_2)).with_translation(Vec3::new(0., 0., -10.0));
    let transformXa = Transform::from_rotation(set_angle_y(FRAC_PI_2)).with_translation(Vec3::new(10., 0., 0.0));


    let res01a = dir_to_local(&transform0a, &point_NZ);
    let res02a = dir_to_local(&transform0a, &point_Z);
    let res03a = dir_to_local(&transform0a, &point_NX);
    let res04a = dir_to_local(&transform0a, &point_X);

    let resNZ1a = dir_to_local(&transformNZa, &point_NZ);
    let resNZ2a = dir_to_local(&transformNZa, &point_Z);
    let resNZ3a = dir_to_local(&transformNZa, &point_NX);
    let resNZ4a = dir_to_local(&transformNZa, &point_X);

    let resX1a = dir_to_local(&transformXa, &point_NZ);
    let resX2a = dir_to_local(&transformXa, &point_Z);
    let resX3a = dir_to_local(&transformXa, &point_NX);
    let resX4a = dir_to_local(&transformXa, &point_X);


    dbg!(res01, res02, res03, res04, resNZ1, resNZ2, resNZ3, resNZ4, resX1, resX2, resX3, resX4, 
        res01a, res02a, res03a, res04a, resNZ1a, resNZ2a, resNZ3a, resNZ4a, resX1a, resX2a, resX3a, resX4a,);

}


/*
fn calc_angle() {
    let data_angle = 0.;

    let data_movement_x = 0.;

    let local_delta_pos = Vec3::new(1.0, 0.0, 0.0);

    let local_delta_vel = Vec3::new(1.0, 0.0, 0.0);

    let force_coorection_value = local_delta_vel.z * local_delta_pos.z;

    let angle = normalize(
        data_angle + 2.0 * local_delta_vel.x.abs().min(0.5) * local_delta_vel.x.signum()
            - 1.0 * local_delta_pos.x.abs().min(0.5) * local_delta_pos.x.signum()
            - data_movement_x * 0.2 * delta_time,
    );

    log::info!(
        "data_angle:{} local_delta_vel.x:{} local_delta_pos.x:{} data.movement.x:{} angle:{}",
        data_angle,
        local_delta_vel.x,
        local_delta_pos.x,
        data_movement_x,
        angle
    );

    /*  let data = crate::tank::TankBodyData {
            movement: Vec2::new(0., 0.),
            pos: Vec2::new(1., 0.5),
            angle: 0.,
            vel: Vec2::new(0., -1.),
        };

        let vel = Velocity {
            linvel: Vec3::new(0., 0., -0.5),
            angvel: Vec3::ZERO,
        };

        let transform = Transform::from_rotation(set_angle_y(0.)).with_translation(Vec3::NEG_X);

        let delta_time = 0.5;

        let current_net_pos = data.pos + data.vel * delta_time;

        let extr_time = 2.0;

        let extr_net_pos = current_net_pos + data.vel * extr_time;

        let delta_pos =
            Vec3::new(extr_net_pos.x, transform.translation.y, extr_net_pos.y) - transform.translation;
        let local_dir = dir_to_local(&transform, &delta_pos);

        let delta_vel = v2_3(data.vel) - vel.linvel;
        let local_delta_vel = dir_to_local(&transform, &delta_vel);

        let force = local_delta_vel.z*local_dir.z;

     /*   let force = if local_dir.z.abs() > ANGLE_EPSILON {
            (local_dir.z.abs()*100. + 1.) * local_dir.z.signum() * transform.forward()
        } else {
            Vec3::ZERO
        };
    */
        let angle = if local_dir.x.abs() > ANGLE_EPSILON {
            normalize(data.angle - local_dir.x.abs().min(0.3)*local_dir.x.signum())
        } else {
            normalize(data.angle - data.movement.x*0.3*delta_time)
        };

        log::info!("delta_pos:{} local_dir:{} force:{} angle:{}", delta_pos, local_dir, force, angle);
    */
}
*/


fn calc_forse() {
    let data = crate::tank::TankBodyData {
        movement: Vec2::new(0., 0.),
        delta_time_linear: 0,
        delta_time_angular: 0,
        pos: Vec2::new(1., 0.5),
        angle: 0.,
        linvel: Vec2::new(0., -1.),
        angvel: 0.,
    };

    let vel = Velocity {
        linvel: Vec3::new(1., 0., -0.5),
        angvel: Vec3::ZERO,
    };

    let transform = Transform::from_rotation(set_angle_y(0.)).with_translation(Vec3::NEG_X);

    let delta_time = 0.5;

    let current_net_pos = data.pos + data.linvel * delta_time;

    let extr_time = 2.0;

    let mut extr_net_pos = v2_3(current_net_pos + data.linvel * extr_time);
    extr_net_pos.y = transform.translation.y;

    let delta_pos = extr_net_pos - transform.translation;
    let local_delta_pos = dir_to_local(&transform, &delta_pos);

    let delta_vel = v2_3(data.linvel) - vel.linvel;
    let local_delta_vel = dir_to_local(&transform, &delta_vel);

    let force_coorection_value = local_delta_vel.z*local_delta_pos.z;

    let force = -10.*transform.back()*force_coorection_value.abs().min(3.)*force_coorection_value.signum();

    /*   let force = if local_dir.z.abs() > ANGLE_EPSILON {
            (local_dir.z.abs()*100. + 1.) * local_dir.z.signum() * transform.forward()
        } else {
            Vec3::ZERO
        };
    */

    /* 
    let angle = normalize(
        data.angle + 2.0 * local_delta_vel.x.abs().min(0.5) * local_delta_vel.x.signum()
            - 1.0 * local_delta_pos.x.abs().min(0.5) * local_delta_pos.x.signum()
            - data.movement.x * 0.2 * delta_time,
    );
*/

    let vel_mult = 2.0*local_delta_vel.x.abs().min(0.5)*local_delta_vel.x.signum();

    let pos_mult = 1.0*local_delta_pos.x.abs().min(0.5)*local_delta_pos.x.signum();

    let mov_mult = -data.movement.x*0.2*delta_time;

    let angle = normalize_angle(data.angle + vel_mult + pos_mult + mov_mult);

    log::info!("data.angle:{} local_delta_vel.x:{} extr_net_pos.x:{} translation.x:{} local_delta_pos.x:{} data.movement.x:{} vel_mult:{}, pos_mult:{}, mov_mult:{}, angle:{} current:{}", 
    data.angle, local_delta_vel.x, extr_net_pos.x, transform.translation.x, local_delta_pos.x, data.movement.x, vel_mult, pos_mult, mov_mult, angle, get_angle_y(&transform.rotation));
    
//    log::info!("data.angle:{} local_delta_vel.x:{} local_delta_pos.x:{} data.movement.x:{} angle:{} current:{}", 
//    data.angle, local_delta_vel.x, local_delta_pos.x, data.movement.x, angle, get_angle_y(&transform.rotation));

    
    //        log::info!("n_vel:{} s_vel:{} d_vel:{} l_d_vel:{} e_n_pos:{} s_pos:{} d_pos:{} l_d_pos:{} force:{}", 
     //                   data.vel.y, vel.linvel.z, delta_vel.z, local_delta_vel.z, extr_net_pos.z, translation.z, delta_pos.z, local_delta_pos.z, force.force.z);

     //       log::info!("delta_time:{} data.vel:{} delta_vel:{} local_delta_vel:{} delta_pos:{} local_delta_pos:{} force.force:{} angle:{}", 
      //      delta_time, data.vel, delta_vel, local_delta_vel, delta_pos, local_delta_pos, force.force, angle);

}
