use std::{collections::LinkedList, f32::consts::PI};

use bevy::prelude::*;
use bevy_rapier3d::{prelude::*, rapier::prelude::JointAxis};

use super::TankControlBody;

//use crate::input::MyInput;

//use nalgebra as nalg;
//use nalgebra::vector;

//use bevy::render::mesh::shape as render_shape;

#[derive(Component)]
pub struct NameComponent {
    pub name: String,
}

#[derive(Component)]
pub struct WheelTag;

#[derive(Component)]
pub struct AxleTag;

#[derive(Component)]
pub struct BodyTag;

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum Tag {
    Body,
    Axle,
    LeftJoint,
    RightJoint,
    LeftWheel,
    RightWheel,
}

#[derive(Component, Debug)]
pub struct WheelData {
    pub tag: Tag,
    pub movement: Option<Vec2>,
}

pub struct VehicleConfig {
    body_half_size: Vec3,

    axle_half_size: f32,

    wheel_hh: f32,
    wheel_r: f32,

    offset_x: f32,
    offset_y: f32,
    offset_z: f32,

    wheel_offset: f32,
}

impl VehicleConfig {
    pub fn new(body_size: Vec3) -> Self {
        Self {
            body_half_size: body_size * 0.5,

            axle_half_size: body_size.z * 0.03,

            wheel_hh: body_size.x * 0.1,
            wheel_r: body_size.z * 0.12,

            offset_x: body_size.x * 0.6,
            offset_y: -body_size.y * 0.6,
            offset_z: body_size.z * 0.4,

            wheel_offset: body_size.x * 0.25,
        }
    }
}

pub fn create_body(
    body: Entity,
    mut commands: &mut Commands,
    body_pos: Vec3,
    body_angle: f32,
    vehicle_cfg: VehicleConfig,
    collision_groups: CollisionGroups,
    solver_groups: SolverGroups,
) -> (Entity, LinkedList<Entity>) {
    let friction_central_wheel = 1.5;
    let friction_outside_wheel = 0.5;

    let mut wheels = LinkedList::new();

    add_components_to_body(
        body,
        body_pos,
        body_angle,
        vehicle_cfg.body_half_size,
        &mut commands,
        collision_groups,
        solver_groups,
    );

    {
        let offset = Vec3::new(
            vehicle_cfg.offset_x,
            vehicle_cfg.offset_y,
            vehicle_cfg.offset_z,
        );
        wheels.push_back(spawn_attached_wheel(
            "RF".to_string(),
            Tag::RightJoint,
            Tag::RightWheel,
            body,
            body_pos,
            offset,
            friction_outside_wheel,
            &vehicle_cfg,
            collision_groups,
            solver_groups,
            &mut commands,
        ));
    }

    {
        let offset = Vec3::new(vehicle_cfg.offset_x, vehicle_cfg.offset_y, 0.0);
        wheels.push_back(spawn_attached_wheel(
            "RM".to_string(),
            Tag::RightJoint,
            Tag::RightWheel,
            body,
            body_pos,
            offset,
            friction_central_wheel,
            &vehicle_cfg,
            collision_groups,
            solver_groups,
            &mut commands,
        ));
    }

    {
        let offset = Vec3::new(
            vehicle_cfg.offset_x,
            vehicle_cfg.offset_y,
            -vehicle_cfg.offset_z,
        );
        wheels.push_back(spawn_attached_wheel(
            "RF".to_string(),
            Tag::RightJoint,
            Tag::RightWheel,
            body,
            body_pos,
            offset,
            friction_outside_wheel,
            &vehicle_cfg,
            collision_groups,
            solver_groups,
            &mut commands,
        ));
    }

    {
        let offset = Vec3::new(
            -vehicle_cfg.offset_x,
            vehicle_cfg.offset_y,
            vehicle_cfg.offset_z,
        );
        wheels.push_back(spawn_attached_wheel(
            "LF".to_string(),
            Tag::LeftJoint,
            Tag::LeftWheel,
            body,
            body_pos,
            offset,
            friction_outside_wheel,
            &vehicle_cfg,
            collision_groups,
            solver_groups,
            &mut commands,
        ));
    }

    {
        let offset = Vec3::new(-vehicle_cfg.offset_x, vehicle_cfg.offset_y, 0.0);
        wheels.push_back(spawn_attached_wheel(
            "LM".to_string(),
            Tag::LeftJoint,
            Tag::LeftWheel,
            body,
            body_pos,
            offset,
            friction_central_wheel,
            &vehicle_cfg,
            collision_groups,
            solver_groups,
            &mut commands,
        ));
    }

    {
        let offset = Vec3::new(
            -vehicle_cfg.offset_x,
            vehicle_cfg.offset_y,
            -vehicle_cfg.offset_z,
        );
        wheels.push_back(spawn_attached_wheel(
            "LR".to_string(),
            Tag::LeftJoint,
            Tag::LeftWheel,
            body,
            body_pos,
            offset,
            friction_outside_wheel,
            &vehicle_cfg,
            collision_groups,
            solver_groups,
            &mut commands,
        ));
    }

    (body, wheels)

    /*    let mut body = commands
            .spawn_bundle(TransformBundle {
                local: Transform::from_translation(
                    get_pos_on_ground(
                        Vec3::new(start_pos_x + 2.0, 0.7, start_pos_z + 2.0),
                        &rapier_context,
                    )
                    .unwrap(),
                ),
                global: GlobalTransform::identity(),
            })
            .insert(PlayerControlBody)
            .insert(bevy_rapier3d::prelude::RigidBody::Dynamic)
            .insert(bevy_rapier3d::prelude::Collider::cuboid(0.68, 0.25, 0.9))
            .insert(CollisionGroups::new(0b0010, 0b1111))
            .insert(SolverGroups::new(0b0010, 0b1111))
            .insert(Restitution::coefficient(0.7))
            .insert(Damping {
                linear_damping: 5.0,
                angular_damping: 20.0,
            })
            .insert(ColliderMassProperties::Density(2.0))
            //     .insert(Transform::from_xyz(-6.0, 0.25, -2.0))
            .insert(ExternalImpulse {
                impulse: Vec3::new(0.0, 0.0, 0.0),
                torque_impulse: Vec3::new(0.0, 0.0, 0.0),
            })
            .id();


            let y = -0.2;

            for x in -1..= 1 {
                for z in -1 ..= 1 {
                    if x == 0 {
                        continue;
                    }

                    let pos = Vec3::new(x as f32, y, z as f32);
                    let revolute = RevoluteJointBuilder::new(Vec3::X)
                    .local_anchor1(pos)
                    .motor_velocity(0.0, 10.0)
                    .motor_max_force(100.0);

                    let revolute_joint = ImpulseJoint::new(body, revolute);
                    let mut wheel = commands
                    .spawn_bundle(TransformBundle::from(Transform::from_xyz(pos.x, pos.y, pos.z)))
                    .insert(RigidBody::Dynamic)
                    .insert(bevy_rapier3d::prelude::Collider::ball(0.3))
                    .insert(Restitution::coefficient(0.01))
                    .insert(Friction::coefficient(3.0))
                    .insert(revolute_joint)
                    .insert(PlayerControlBodyWheelMotor {dx: x as f32})
                    .id();
                }
            }
    */
}

fn add_components_to_body(
    body: Entity,
    pos: Vec3,
    angle: f32,
    half_size: Vec3,
    commands: &mut Commands,
    collision_groups: CollisionGroups,
    solver_groups: SolverGroups,
) {
    let linear_damping = 5.0;
    let angular_damping = 5.0;
    let restitution = 0.7;
    let friction = 0.7;
    let density = 1.0;
    let local_center_of_mass = Vec3::new(0.0, -half_size.y * 0.9, 0.0);
    let ballast_mass = 100.0;

    commands
        .entity(body)
        .insert(RigidBody::Dynamic)
        .insert(Sleeping::default())
        .insert(Restitution::coefficient(restitution))
        .insert(Friction::coefficient(friction))
        .insert(Collider::cuboid(half_size.x, half_size.y, half_size.z))
        .insert(ColliderMassProperties::Density(density))
        .insert(AdditionalMassProperties::MassProperties(MassProperties {
            local_center_of_mass,
            ..Default::default()
        }))
        .insert(Damping {
            linear_damping,
            angular_damping,
        })
        .insert(ExternalForce {
            force: Vec3::new(0.0, 0.0, 0.0),
            torque: Vec3::new(0.0, 0.0, 0.0),
        })
        .insert(ExternalImpulse {
            impulse: Vec3::new(0.0, 0.0, 0.0),
            torque_impulse: Vec3::new(0.0, 0.0, 0.0),
        })
        //		.insert(ColliderDebugRender::default())
        .insert(NameComponent {
            name: format!("Body"),
        })
        .insert(TankControlBody {
            movement: Vec2::ZERO,
            pos: Vec2::new(pos.x, pos.z),
            dir: angle,
        })
        .insert(collision_groups)
        .insert(solver_groups)
        .with_children(|parent| {
            parent
                .spawn()
                .insert(Collider::cuboid(
                    half_size.x * 0.3,
                    half_size.y * 0.2,
                    half_size.z * 0.3,
                ))
                .insert(Transform::from_translation(local_center_of_mass))
                .insert(ColliderMassProperties::Density(ballast_mass));
        });
}

fn spawn_attached_wheel(
    prefix: String,
    joint_tag: Tag,
    wheel_tag: Tag,
    body: Entity,
    body_pos: Vec3,
    main_offset: Vec3,
    friction: f32,
    vehicle_cfg: &VehicleConfig,
    collision_groups: CollisionGroups,
    solver_groups: SolverGroups,
    mut commands: &mut Commands,
) -> Entity {
    let x_sign = main_offset.x * (1.0 / main_offset.x.abs());
    let wheel_offset = Vec3::X * vehicle_cfg.wheel_offset * x_sign;

    let axle_pos = body_pos + main_offset;
    let axle = spawn_axle(
        &prefix,
        axle_pos,
        vehicle_cfg.axle_half_size,
        collision_groups,
        solver_groups,
        &mut commands,
    );

    let mut anchor1 = main_offset;
    let mut anchor2 = Vec3::ZERO;
    let _axle_joint = spawn_axle_joint(body, axle, anchor1, anchor2, &mut commands);

    let wheel_pos = axle_pos + wheel_offset;
    let wheel = spawn_wheel(
        &prefix,
        WheelData {
            tag: wheel_tag,
            movement: None,
        },
        wheel_pos,
        vehicle_cfg.wheel_hh,
        vehicle_cfg.wheel_r,
        friction,
        collision_groups,
        solver_groups,
        &mut commands,
    );

    anchor1 = wheel_offset;
    anchor2 = Vec3::ZERO;
    let wheel_joint = spawn_wheel_joint(
        WheelData {
            tag: wheel_tag,
            movement: None,
        },
        axle,
        wheel,
        anchor1,
        anchor2,
        &mut commands,
    );

    //	(axle_joint, wheel_joint, wheel)
    wheel_joint
}

fn spawn_axle(
    prefix: &String,
    pos_in: Vec3,
    half_size: f32, //Vec3,
    collision_groups: CollisionGroups,
    solver_groups: SolverGroups,
    commands: &mut Commands,
) -> Entity {
    //	let tmp_pos = pos_in + Vec3::new(0.0, 0.3, 0.0);

    let linear_damping = 5.0;
    let angular_damping = 5.0;

    let bundle = TransformBundle {
        local: Transform::from_translation(pos_in),
        global: GlobalTransform::identity(),
    };

    commands
        .spawn_bundle(bundle)
        .insert(RigidBody::Dynamic)
        //		.insert(LockedAxes::ROTATION_LOCKED | LockedAxes::TRANSLATION_LOCKED_X | LockedAxes::TRANSLATION_LOCKED_Z )
        //		.insert(Collider::cuboid(half_size, half_size*3., half_size))
        .insert(Collider::ball(half_size))
        //		.insert(bevy_rapier3d::prelude::Collider::ball(half_size))
        //		.insert(ColliderMassProperties::Density(1000.0))
        .insert(Damping {
            linear_damping,
            angular_damping,
        })
        //		.insert(ColliderDebugRender::default())
        .insert(NameComponent {
            name: format!("{} Axle", prefix),
        })
        //		.insert(Tag::Axle)
        .insert(collision_groups)
        .insert(solver_groups)
        .id()
}

fn spawn_wheel(
    prefix: &String,
    wheel_data: WheelData,
    pos_in: Vec3,
    half_height: f32,
    radius: f32,
    friction: f32,
    collision_groups: CollisionGroups,
    solver_groups: SolverGroups,
    commands: &mut Commands,
) -> Entity {
    let restitution = 0.3;
    let density = 1.0;
    let linear_damping = 5.0;
    let angular_damping = 5.0;

    let transform = Transform::from_translation(pos_in);

    let bundle = TransformBundle {
        local: transform,
        global: GlobalTransform::from(transform),
    };

    commands
        .spawn()
        .insert(NameComponent {
            name: format!("{} Wheel", prefix),
        })
        .insert_bundle(bundle)
        .insert(RigidBody::Dynamic)
        .with_children(|parent| {
            parent
                .spawn()
                .insert(Transform::from_rotation(Quat::from_rotation_z(
                    90.0_f32.to_radians(),
                )))
                .insert(Collider::cylinder(half_height, radius))
                .insert(Restitution::coefficient(restitution))
                .insert(Friction::coefficient(friction))
                .insert(ColliderMassProperties::Density(density))
                .insert(Damping {
                    linear_damping,
                    angular_damping,
                })
                .insert(wheel_data)
                .insert(collision_groups)
                .insert(solver_groups);
        })
        .id()
}

fn spawn_axle_joint(
    body: Entity,
    axle: Entity,
    body_anchor: Vec3,
    axle_anchor: Vec3,
    commands: &mut Commands,
) -> Entity {
    let target_vel = 0.0;
    let factor = 1.;
    let max_force = 5.;

    let axle_joint_builder = PrismaticJointBuilder::new(Vec3::Y)
        .local_anchor1(body_anchor)
        .local_anchor2(axle_anchor)
        .motor_velocity(target_vel, factor)
        .motor_max_force(max_force)
        .motor_position(-0.1, 1.0, 0.1); // by default we want axle joint to stay fixed

    commands
        .entity(axle)
        .insert(MultibodyJoint::new(body, axle_joint_builder))
        .id()
}

fn spawn_wheel_joint(
    wheel_data: WheelData,
    axle: Entity,
    wheel: Entity,
    axle_anchor: Vec3,
    wheel_anchor: Vec3,
    commands: &mut Commands,
) -> Entity {
    let target_vel = 0.0;
    let factor = 0.1;
    let max_force = 2.;

    let wheel_joint = RevoluteJointBuilder::new(Vec3::X)
        .local_anchor1(axle_anchor)
        .local_anchor2(wheel_anchor)
        .motor_velocity(target_vel, factor)
        .motor_max_force(max_force);

    //		println!("tank_body_physics spawn_wheel_joint, tag: {:?}", wheel_data.tag);

    commands
        .entity(wheel)
        .insert(MultibodyJoint::new(axle, wheel_joint))
        .insert(wheel_data)
        .id()
}

pub fn update_body_moving(
    //	mut commands: Commands,
    mut joint_query: Query<&mut MultibodyJoint>,
    entity_query: Query<(Entity, &Transform, ChangeTrackers<WheelData>, &WheelData)>,
) {
    let multipler = 10.;
    let factor = 0.1; //if velosity_left != 0. && velosity_right != 0. { 0.1 } else { 2.0 };

    for (entity, transform, tracker, wheel_data) in entity_query.iter() {
        if !tracker.is_changed() {
            continue;
        }

        if let Ok(mut joint) = joint_query.get_mut(entity) {
            if let Some(movement) = wheel_data.movement {
                let velosity_left = (movement.y - movement.x) * multipler;
                let velosity_right = (movement.y + movement.x) * multipler;

                match wheel_data.tag {
                    Tag::LeftWheel => {
                        joint.data.set_motor_velocity(
                            JointAxis::AngX,
                            velosity_left, //target_vel
                            factor,        //damping
                        );
                        //			joint.data.set_motor_velocity(JointAxis::AngX, velosity_left, factor);
                        joint.data.set_limits(JointAxis::AngX, [f32::MIN, f32::MAX]);
                    }
                    Tag::RightWheel => {
                        joint.data.set_motor_velocity(
                            JointAxis::AngX,
                            velosity_right, //target_vel
                            factor,         //damping
                        );
                        //			joint.data.set_motor_velocity(JointAxis::AngX, velosity_right, factor);
                        joint.data.set_limits(JointAxis::AngX, [f32::MIN, f32::MAX]);
                    }
                    _ => continue,
                }

            //            println!("-------------------- joint: {:?}", &joint);

            //  if let Some(mut joint) = joints.get_mut(joint_comp) {
            } else {
	/* 			//TODO make a brake for this case
				dbg!["update_body_moving", transform.rotation.to_euler(EulerRot::XYZ)];

                let target_pos = transform.rotation.to_euler(EulerRot::XYZ).0;//joint.data.motor(JointAxis::AngX).unwrap().target_pos;
				
				dbg!["update_body_moving", target_pos];

                joint.data.set_motor_position(
                    JointAxis::AngX,
                    target_pos, //target_pos:
                    1.,        //stiffness
                    1.,        //damping
                );
*/
				joint.data.set_motor_velocity(
					JointAxis::AngX,
					0., //target_vel
					factor,         //damping
				);

				joint.data.set_limits(JointAxis::AngX, [f32::MIN, f32::MAX]);

                //			joint.data.set_motor_velocity(JointAxis::AngX, 0., 200.);
                //			joint.data.set_limits(JointAxis::AngX, [f32::MIN, f32::MAX]);
            }
        }
    }
}

/*
pub fn update_body_moving(
//	mut commands: Commands,
    mut joint_query: Query<&mut MultibodyJoint>,
    entity_query: Query<(Entity, &WheelData), With<WheelData>>,
) {
    let multipler = 10.;
    let factor = 0.1;//if velosity_left != 0. && velosity_right != 0. { 0.1 } else { 2.0 };

    for (entity, wheel_data) in entity_query.iter() {

        if let Ok(mut joint) = joint_query.get_mut(entity) {
            if let Some(movement) = wheel_data.movement {

                let velosity_left = (movement.y - movement.x)*multipler;
                let velosity_right = (movement.y + movement.x)*multipler;

                match wheel_data.tag {
                    Tag::LeftWheel => {
                        joint.data.set_motor(
                            JointAxis::AngX,
                            0.,//target_pos:
                            velosity_left,//target_vel
                            0.,//stiffness
                            factor, //damping
                        );
            //			joint.data.set_motor_velocity(JointAxis::AngX, velosity_left, factor);
                        joint.data.set_limits(JointAxis::AngX, [f32::MIN, f32::MAX]);
                    },
                    Tag::RightWheel => {
                        joint.data.set_motor(
                            JointAxis::AngX,
                            0.,//target_pos:
                            velosity_right,//target_vel
                            0.,//stiffness
                            factor, //damping
                        );
            //			joint.data.set_motor_velocity(JointAxis::AngX, velosity_right, factor);
                        joint.data.set_limits(JointAxis::AngX, [f32::MIN, f32::MAX]);
                    },
                    _ => continue,
                }

    //            println!("-------------------- joint: {:?}", &joint);

            //  if let Some(mut joint) = joints.get_mut(joint_comp) {

            } else {
                let target_pos = joint.data.motor(JointAxis::AngX).unwrap().target_pos;
                joint.data.set_motor(
                    JointAxis::AngX,
                    target_pos,//target_pos:
                    0.,//target_vel
                    10.,//stiffness
                    10., //damping
                );

    //			joint.data.set_motor_velocity(JointAxis::AngX, 0., 200.);
    //			joint.data.set_limits(JointAxis::AngX, [f32::MIN, f32::MAX]);
            }
        }
    }
}
*/
