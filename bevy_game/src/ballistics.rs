
// from https://habr.com/ru/post/538952/ https://github.com/forrestthewoods/lib_fts

use std::f32::consts::{PI, FRAC_PI_4};

use bevy::prelude::Vec3;

// Solve firing angles for a ballistic projectile with speed and gravity to hit a fixed position.
//
// proj_pos (Vector3): point projectile will fire from
// proj_speed (float): scalar speed of projectile
// target (Vector3): point projectile is trying to hit
// gravity (float): force of gravity, positive down
//
// return firing solution (low angle)
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct Angle {
    pub angle: f32,
    pub time: f32,
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct Result {
    pub low: Option<Angle>,
    pub high: Option<Angle>,
}

pub fn solve_ballistic_arc(
    proj_pos: Vec3,    
    proj_speed: f32,
    target: Vec3,
    gravity: f32,
) -> Result  {
    // Handling these cases is up to your project's coding standards
    assert!(
        proj_pos != target && proj_speed > 0. && gravity > 0.,
        "fts.solve_ballistic_arc called with invalid data"
    );

    let mut res: Result = Result::default();

    let diff = target - proj_pos;
    let diff_xz = Vec3::new(diff.x, 0., diff.z);
    let ground_dist = diff_xz.length();

    let speed2 = proj_speed*proj_speed;
    let y = diff.y;
    let x = ground_dist;
    let gx = gravity*x;

    let root = speed2*speed2 - gravity*(gravity*x*x + 2.*y*speed2);

    // No solution
    if root < 0. {
        return res;
    }

    let low_ang = (speed2 - root.sqrt()).atan2(gx);
    res.low = Some(Angle{angle: low_ang, time: proj_speed*low_ang.cos()});

    let high_ang = (speed2 + root.sqrt()).atan2(gx);

    if low_ang == high_ang {
        return res;
    }

    res.high = Some(Angle{angle: high_ang, time: proj_speed*high_ang.cos()});

    return res;
}

pub fn calc_shot_dir(
    start_shell_pos: Vec3,
    target_pos: Vec3, 
    shell_speed: f32,
    shell_radius: f32,
    gravity: f32,
) -> Vec3 {
    let mut tmp_dir = target_pos - start_shell_pos;
    tmp_dir.y += shell_radius;	    

    let tmp_dir_xz = Vec3::new(tmp_dir.x, 0., tmp_dir.z).normalize();

    let result = solve_ballistic_arc(
        start_shell_pos,
        shell_speed,
        target_pos,
        gravity,
    );

    let angle = if result.low.is_some() {
        result.low.unwrap().angle
    } else {
        FRAC_PI_4
    };
//    let angle = Mathf.Max(Mathf.Min(result.low.unwrap().angle, maxCannonAngle), minCannonAngle);

//            Debug.Log("calcShotDir startShellSpeed " + startShellSpeed + " " + m_FireTransform.position + " " + tmpDir  + " " + angle);
    let cos_angle = angle.cos();

    return Vec3::new(tmp_dir_xz.x*cos_angle, angle.sin(), tmp_dir_xz.z*cos_angle).normalize();
}
