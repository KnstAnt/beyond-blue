//use bevy::prelude::Component;
//use serde::{Deserialize, Serialize, de::DeserializeOwned};


pub fn delta_dir(target_dir: f32, old_dir: f32) -> f32 {
    normalize(target_dir - old_dir)
}

pub fn delta_dir_max(target_dir: f32, old_dir: f32, rot_speed: f32, delta_time: f32) -> f32 {
    let delta = delta_dir(target_dir, old_dir);
    let max_delta = rot_speed.abs() * delta_time;
    let res = if delta.abs() > max_delta {
        max_delta * delta.signum() + delta*(delta_time/1.0)
    } else {
        delta
    };

    res
}

pub fn calc_dir(target_dir: f32, old_dir: f32, rot_speed: f32, delta_time: f32) -> f32 {
    let delta = delta_dir_max(target_dir, old_dir, rot_speed, delta_time);
    let new_dir = old_dir + delta; //TODO implement ping time

    //  log::info!("Tank calc_dir dir:{:?} old_dir:{:?} rot_speed:{:?} delta_time:{:?} delta:{:?} new_dir:{:?}",
    //      dir, old_dir, rot_speed, delta_time, delta, new_dir );

    normalize(new_dir)
}

pub fn normalize(mut dir: f32) -> f32 {
    if dir > std::f32::consts::PI {
        dir -= std::f32::consts::TAU;
    }

    if dir < -std::f32::consts::PI {
        dir += std::f32::consts::TAU;
    }

    dir
}

pub fn calc_rotation_speed(
    delta_time: f32,
    rotation: f32,
    max_speed: f32,
    old_speed: f32,
    run_time: f32,
    stop_time: f32,
) -> f32 {
    let mut new_speed = if rotation.abs() > 0. {
        (rotation * delta_time * max_speed) / run_time + old_speed
    } else {
        let delta_speed = (delta_time * max_speed.abs()) / stop_time;

        if old_speed.abs() > delta_speed {
            (old_speed.abs() - delta_speed) * old_speed.signum()
        } else {
            0.
        }
    };

    if new_speed.abs() > max_speed {
        new_speed = max_speed * new_speed.signum()
    }

    //                  dbg![time.delta_seconds(), delta_acceleration, old_rotation, new_rotation];
    new_speed
}
