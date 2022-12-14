use std::f32::consts::{PI, FRAC_PI_2, TAU};

use bevy::prelude::{Transform, Quat, Vec3, Vec2, GlobalTransform};


pub fn delta_angle(target_angle: f32, old_angle: f32) -> f32 {
    normalize_angle(target_angle - old_angle)
}

pub fn delta_angle_max(target_angle: f32, old_angle: f32, rot_speed: f32, delta_time: f32) -> f32 {
    let delta = delta_angle(target_angle, old_angle);
    let max_delta = rot_speed.abs() * delta_time;
    let res = if delta.abs() > max_delta {
        max_delta * delta.signum() + delta*(delta_time/1.0)
    } else {
        delta
    };

    res
}

pub fn calc_angle(target_angle: f32, old_angle: f32, rot_speed: f32, delta_time: f32) -> f32 {
    let delta = delta_angle_max(target_angle, old_angle, rot_speed, delta_time);
    let new_angle = old_angle + delta; //TODO implement ping time

    //  log::info!("Tank calc_dir dir:{:?} old_dir:{:?} rot_speed:{:?} delta_time:{:?} delta:{:?} new_dir:{:?}",
    //      dir, old_dir, rot_speed, delta_time, delta, new_dir );

    normalize_angle(new_angle)
}

pub fn normalize_angle(mut angle: f32) -> f32 {
    if angle > PI {
        angle -= TAU;
    }

    if angle < -PI {
        angle += TAU;
    }

    angle
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

pub trait Convert {
    fn dir_to_local(&self, dir: &Vec3) -> Vec3;
    fn dir_to_global(&self, dir: &Vec3) -> Vec3;
    fn pos_to_local(&self, dir: &Vec3) -> Vec3;
    fn pos_to_global(&self, dir: &Vec3) -> Vec3;
}

impl Convert for Transform {
    fn dir_to_local(&self, dir: &Vec3) -> Vec3 {
        dir_to_local(&self.rotation, dir)
    }
    fn dir_to_global(&self, dir: &Vec3) -> Vec3 {
        dir_to_global(&self.rotation, dir)
    }
    fn pos_to_local(&self, pos: &Vec3) -> Vec3 {
        self.compute_matrix()
        .inverse()
        .transform_point3(*pos)
    }
    fn pos_to_global(&self, pos: &Vec3) -> Vec3 {
        self.transform_point(*pos)
    }
}

impl Convert for GlobalTransform {
    fn dir_to_local(&self, dir: &Vec3) -> Vec3 {
        dir_to_local(&self.compute_transform(), dir)
    }
    fn dir_to_global(&self, dir: &Vec3) -> Vec3 {
        dir_to_global(&self.compute_transform(), dir)
    }
    fn pos_to_local(&self, pos: &Vec3) -> Vec3 {
        pos_to_local(&self.compute_transform(), pos)
    }
    fn pos_to_global(&self, pos: &Vec3) -> Vec3 {
        pos_to_global(&self.compute_transform(), pos)
    }
}

impl Convert for Quat {
    fn dir_to_local(&self, dir: &Vec3) -> Vec3 {
        Transform::from_rotation(*self)
        .compute_matrix()
        .inverse()
        .transform_point3(*dir)
    }
    fn dir_to_global(&self, dir: &Vec3) -> Vec3 {
        Transform::from_rotation(*self)
        .transform_point(*dir)
    }
    fn pos_to_local(&self, pos: &Vec3) -> Vec3 {
        panic!("wrong data");
        Vec3::ZERO
    }
    fn pos_to_global(&self, pos: &Vec3) -> Vec3 {
        panic!("wrong data");
        Vec3::ZERO
    }
}

pub fn dir_to_local(src: &impl Convert, dir: &Vec3) -> Vec3 {
    src.dir_to_local(dir)
}

pub fn dir_to_global(src: &impl Convert, dir: &Vec3) -> Vec3 {
    src.dir_to_global(dir)
}

pub fn pos_to_local(src: &impl Convert, pos: &Vec3) -> Vec3 {
    src.pos_to_local(pos)
}

pub fn pos_to_global(src: &impl Convert, pos: &Vec3) -> Vec3 {
    src.pos_to_global(pos)
}


pub trait AngleY {
    fn get(&self) -> f32;
    fn set(a: f32) -> Self;
}

impl AngleY for Transform {
    fn get(&self) -> f32 {
        self.rotation.to_euler(bevy::prelude::EulerRot::YXZ).0
    }
    fn set(a: f32) -> Self {
        Transform::from_rotation(Quat::from_axis_angle(Vec3::Y, a))
    }
}

impl AngleY for Quat {
    fn get(&self) -> f32 {
        self.to_euler(bevy::prelude::EulerRot::YXZ).0
    }

    fn set(a: f32) -> Self {
        Quat::from_axis_angle(Vec3::Y, a)
    }
}

impl AngleY for Vec3 {
    fn get(&self) -> f32 {
        if self.is_normalized() && self.y == 0. {
            self.dot(Vec3::NEG_X) * FRAC_PI_2// * self.dot(Vec3::NEG_Z).signum()
        } else {
  //          let norm_self = Vec2::new(self.x, self.z).normalize();
            Vec2::new(self.x, self.z).normalize().dot(Vec2::NEG_X) * FRAC_PI_2 //* norm_self.dot(Vec2::NEG_Y).signum()
        }
    }

    fn set(a: f32) -> Self {
        Vec3::new(-a.sin(), 0., -a.cos())
    }
}

impl AngleY for Vec2 {
    fn get(&self) -> f32 {
        if self.is_normalized() {
            self.dot(Vec2::NEG_X) * FRAC_PI_2// * self.dot(Vec2::NEG_Y).signum()
        } else {
  //          let norm_self = self.clone().normalize();
            self.clone().normalize().dot(Vec2::NEG_X) * FRAC_PI_2 //* norm_self.dot(Vec2::NEG_Y).signum()
        }
    }

    fn set(a: f32) -> Self {
        Vec2::new(-a.sin(), -a.cos())
    }
}
pub fn get_angle_y(v: &impl AngleY) -> f32 {
    v.get()
}

pub fn set_angle_y<T: AngleY>(angle: f32) -> T  {
    T::set(angle)
}

/* 
pub fn get_angle_y(transform: &Transform) -> f32 {
    transform.rotation.to_euler(bevy::prelude::EulerRot::YXZ).0
}

pub fn set_angle_y(angle: f32) -> Quat  {
    Quat::from_axis_angle(Vec3::Y, angle)
}*/

pub fn v2_3(v2: Vec2) -> Vec3  {
    Vec3::new(v2.x, 0., v2.y)
}
pub fn v3_2(v3: Vec3) -> Vec2  {
    Vec2::new(v3.x, v3.z)
}

#[cfg(test)]
mod tests {
    use super::*;
    

//  The X axis goes from left to right (+X points right).
//  The Y axis goes from bottom to top (+Y points up).
//  The Z axis goes from far to near (+Z points towards you, out of the screen).
//  Rotation counterclock-wise
    #[test]
    fn test_dir_to_local() {
        assert!(dir_to_local(&Transform::from_rotation(Quat::from_axis_angle(Vec3::Y, FRAC_PI_2)), &Vec3::NEG_Z).abs_diff_eq(Vec3::X, f32::EPSILON*10.));
    }

    #[test]
    fn test_dir_to_global() {
        assert!(dir_to_global(&Transform::from_rotation(Quat::from_axis_angle(Vec3::Y, FRAC_PI_2)), &Vec3::NEG_Z).abs_diff_eq(Vec3::NEG_X, f32::EPSILON*10.));
    }
    #[test]
    fn test_pos_to_local() {
        assert!(pos_to_local(&Transform::from_rotation(Quat::from_axis_angle(Vec3::Y, FRAC_PI_2)), &Vec3::NEG_Z).abs_diff_eq(Vec3::X, f32::EPSILON*10.));
    }

    #[test]
    fn test_pos_to_global() {
        assert!(pos_to_global(&Transform::from_rotation(Quat::from_axis_angle(Vec3::Y, FRAC_PI_2)), &Vec3::NEG_Z).abs_diff_eq(Vec3::NEG_X, f32::EPSILON*10.));
    }

    #[test]
    fn test_get_angle_y() {
        assert_eq!(get_angle_y(&Transform::from_rotation(Quat::from_axis_angle(Vec3::Y, 0.))), 0.);
    }

    #[test]
    fn test_v2_3() {
        assert_eq!(v2_3(Vec2::new(2., 3.)), Vec3::new(2., 0., 3.));
    }    
    #[test]
    fn test_v3_2() {
        assert_eq!(v3_2(Vec3::new(2., 0., 3.)), Vec2::new(2., 3.));
    } 
}