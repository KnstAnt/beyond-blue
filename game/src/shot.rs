use std::time::Duration;

use bevy::prelude::*;
use bevy_rapier3d::plugin::*;

use crate::AppState;

use crate::explosion::add_explosion;
use crate::terrain::get_pos_on_ground;



pub struct ShotPlugin;

impl Plugin for ShotPlugin {
    fn build(&self, app: &mut App) {
  //      let before_system_set = SystemSet::on_update(AppState::Playing)
        //      .with_system(print_before_system);

 //   let after_system_set = SystemSet::on_update(AppState::Playing)
        //    .with_system(print_after_system);

    let update_system_set = SystemSet::on_update(AppState::Playing)
        .with_system(remove_shots);

    app
 //       .add_system_set_to_stage(CoreStage::PreUpdate, State::<AppState>::get_driver())
 //       .add_system_set_to_stage(CoreStage::PostUpdate, State::<AppState>::get_driver())
        .add_system_set_to_stage(CoreStage::Update, update_system_set)
        ;
    }
}


#[derive(Component)]
pub struct Data {
    timer: Timer,
    pub explosion_radius: f32,
    pub explosion_force: f32,
}

impl Data {
    pub fn new( live_max_time: f32, explosion_force: f32) -> Self { 
        Self { 
            timer: Timer::new(
                Duration::from_secs_f32(live_max_time),
                false,
            ), 
            explosion_radius: explosion_force.sqrt(), 
            explosion_force,
        }
    }
}

fn remove_shots(
    mut commands: Commands,
    rapier_context: Res<RapierContext>,
    mut query: Query<(Entity, &GlobalTransform, &mut Data)>,
    time: Res<Time>,
) {
    //info!("remove_shots");

    for (entity, global_transform, mut shot_data) in query.iter_mut() {
        // timers gotta be ticked, to work
        shot_data.timer.tick(time.delta());

        //     info!("remove_shots tick");

        // if it finished, despawn the bomb
        if shot_data.timer.finished() {
                        info!("remove_shots finished");
            commands.entity(entity).despawn_recursive();
            continue;
        }

        // bug: test terrain
        if let Some(mut pos) = get_pos_on_ground(
            Vec3::new(global_transform.translation().x,
            0.1,
            global_transform.translation().z),
            &rapier_context
        ) {
            if global_transform.translation().y == 0. || pos.y < global_transform.translation().y {
                continue;
            }

            pos.y += 0.1;

            println!("remove_shots get_pos_on_ground pos: {:?}  translation: {:?}", pos, global_transform.translation());
            
            add_explosion(&mut commands, entity, pos, &shot_data);
            continue;
        }

    //    add_explosion(&mut commands, entity, global_transform.translation, &shot_data);
        continue;
       // commands.entity(entity).despawn_recursive();
    }
}
