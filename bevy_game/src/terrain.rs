use bevy::gltf::*;
use bevy::prelude::*;
use bevy::prelude::shape::*;
use bevy_rapier3d::prelude::*;
use iyes_loopless::prelude::ConditionSet;
use iyes_loopless::prelude::IntoConditionalSystem;


#[derive(Clone, Eq, PartialEq, Debug, Hash)]
pub enum TerrainState {
    None,
    CreateAssets,
    CreatePhysics,
    Complete,
}

#[derive(Debug)]
pub(crate) struct TerrainScene {
    scene_handle: Option<Handle<Gltf>>,
    loading_state: TerrainState,
}
 
impl Default for TerrainScene {
    fn default() -> Self {
        Self {
            scene_handle: None,
            loading_state: TerrainState::None,
        }
    }
}

impl TerrainScene {
    pub fn new(scene_handle: Handle<Gltf>) -> Self  {
        Self {
            scene_handle: Some(scene_handle),
            loading_state: TerrainState::CreateAssets,
        }
    }

    pub fn is_completed(&self) -> bool  {
        self.loading_state == TerrainState::Complete
    }
}

#[derive(Component)]
pub struct TerrainRootEntity;

#[derive(Component)]
pub struct TerrainEntity {
    pub mesh: Handle<Mesh>,
    pub scale: Vec3,
}


pub struct TerrainPlugin;

impl Plugin for TerrainPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<TerrainScene>()
            .add_system( setup_terrain_assets.run_if(is_create_assets), )
            .add_system( setup_terrain_physics.run_if(is_create_physics), )
//            .add_system( setup_terrain_complete.run_if(is_complete), )
            .add_system_set(
                ConditionSet::new()
                .run_if(is_create_assets)
                .run_if(is_create_physics)
//                .run_if(is_complete)
                .into() 
            )
            ;
    }
}


fn is_create_assets(terrain_scene: Res<TerrainScene>) -> bool {
 //   println!("terrain is_create_assets");
    terrain_scene.loading_state == TerrainState::CreateAssets && terrain_scene.scene_handle.is_some()
}

fn is_create_physics(terrain_scene: Res<TerrainScene>) -> bool {
 //   println!("terrain is_create_physics");
    terrain_scene.loading_state == TerrainState::CreatePhysics
}
/* 
fn is_complete(terrain_scene: Res<TerrainScene>) -> bool {
    terrain_scene.loading_state == TerrainState::Complete
}
*/


fn setup_terrain_assets(
    commands: Commands,
//    mut terrain_state: ResMut<State<TerrainState>>,
    mut terrain_scene: ResMut<TerrainScene>,
    gltfs: Res<Assets<Gltf>>,
    gltf_nodes: Res<Assets<GltfNode>>,
    gltf_meshes: Res<Assets<GltfMesh>>,
    mut _meshes: ResMut<Assets<Mesh>>,
    mut _materials: ResMut<Assets<StandardMaterial>>,
) {
    if false {
        if let Some(scene_handle) = &terrain_scene.scene_handle {
            println!("terrain setup_terrain_assets");

                if add_terrain_assets(commands, scene_handle, gltfs, gltf_nodes, gltf_meshes/*, meshes, materials*/) {
                terrain_scene.loading_state = TerrainState::CreatePhysics;
                println!("terrain setup_terrain_assets complete");
            }
        }
    } else {
        add_test_plane(commands, terrain_scene, _meshes, _materials);
    }
}

fn add_terrain_assets(
    mut commands: Commands,
//    mut terrain_state: ResMut<State<TerrainState>>,
    gltf_handle: &Handle<Gltf>,
    gltfs: Res<Assets<Gltf>>,
    gltf_nodes: Res<Assets<GltfNode>>,
    gltf_meshes: Res<Assets<GltfMesh>>,
 //   mut meshes: ResMut<Assets<Mesh>>,
 //   mut materials: ResMut<Assets<StandardMaterial>>,
) -> bool {
    if let Some(gltf) = gltfs.get(gltf_handle) {
        println!("terrain add_terrain_assets gltfs.get(&model_assets.terrain) ok");

        if let Some(first_node_handle) = gltf.nodes.first() {
            if let Some(first_node) = gltf_nodes.get(first_node_handle) {
                println!("terrain add_terrain_assets first_node ok");

                if let Some(entity) = create_gltf_entity(&mut commands, &gltf_meshes, first_node) {

                    //   .with_translation(Vec3::new(300., 0., -400.))

                    commands.entity(entity).insert(TerrainRootEntity);
        
                    if !first_node.children.is_empty() {
                        create_gltf_entities(&mut commands, entity, &gltf_meshes, &first_node.children);
                    }
                }
            
                println!("TerrainAssetsComplete");
   //             terrain_state.replace(TerrainState::CreatePhysics).unwrap();

//                commands.insert_resource(PhysicsHooksWithQueryResource(std::boxed::Box::new(SameUserDataFilter {},)));
                return true;
            }
        }
    } else {
        println!("terrain add_terrain_assets gltfs.get(&model_assets.terrain) fault");        
    }

    return false;
}

fn create_gltf_entities<'n>(
    commands: &mut Commands,
    parent: Entity,
    gltf_meshes: &Res<Assets<GltfMesh>>,
    gltf_node: impl IntoIterator<Item = &'n GltfNode>,
) { 
//    println!("terrain create_gltf_entities"); 

    for node in gltf_node { 
        if let Some(entity) = create_gltf_entity(commands, gltf_meshes, &node) {
            commands.entity(parent).add_child(entity);
            if !node.children.is_empty() {
                create_gltf_entities(commands, parent, gltf_meshes, &node.children)
            }
        }
    }
}

fn create_gltf_entity(
    commands: &mut Commands,
    gltf_meshes: &Res<Assets<GltfMesh>>,
    gltf_node: &GltfNode,
) -> Option<Entity> {
    println!("terrain create_gltf_entity"); 

    let primitive = gltf_node
        .mesh
        .as_ref()
        .and_then(|mesh| gltf_meshes.get(mesh))
        .and_then(|mesh| mesh.primitives.first());

    if primitive.is_some() && 
        primitive.unwrap().material.as_ref().is_some() {
  //          println!("terrain create_gltf_entity primitive.material ok"); 
            let primitive = primitive.unwrap();
            let material = primitive.material.as_ref().unwrap();

            return Some(commands.spawn_bundle(PbrBundle {
                    mesh: primitive.mesh.clone(),
                    material: material.clone(),
                    transform: gltf_node.transform.clone(),
         //           global_transform: todo!(),
         //           visibility: todo!(),
         //           computed_visibility: todo!(),
                    ..Default::default()
                })
                .insert(TerrainEntity {
                    scale: gltf_node.transform.scale,
                    mesh: primitive.mesh.clone(),
                })
                .id()
            );                    
    } else if !gltf_node.children.is_empty() {
 //       println!("terrain create_gltf_entity TransformBundle ok");  
        return Some(commands.spawn_bundle(SpatialBundle {
            transform: gltf_node.transform.clone(),
           ..Default::default()
            })
            .id()
        );
    }

    println!("terrain create_gltf_entity fault"); 

    return None;
}

fn add_test_plane(
    mut commands: Commands,
    mut terrain_scene: ResMut<TerrainScene>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Spawn a plane that will represent the ground. It will be used to pick the mouse location in 3D space
    commands
        .spawn_bundle(PbrBundle {
            mesh: meshes.add(Mesh::from(Plane { size: 50.0 })),
            material: materials.add(Color::rgb(0.3, 0.3, 0.4).into()),
            ..Default::default()
        })
        .insert(bevy_rapier3d::prelude::Collider::cuboid(25.0, 0.001, 25.0))
        .insert(Transform::from_xyz(0.0, -1., 0.0))
        .insert(bevy_rapier3d::prelude::RigidBody::Fixed)
//        .insert(CustomFilterTag::GroupTerrain)
        .insert(CollisionGroups::new(0b0001, 0b0111))
        .insert(SolverGroups::new(0b0001, 0b0111))
        .insert(TerrainRootEntity)
        .insert(Friction::coefficient(0.3))
        ;

 //   terrain_state.replace(TerrainState::Complete).unwrap();
    terrain_scene.loading_state = TerrainState::Complete;
    //        commands.insert_resource(NextState(SetupState::TerrainPhysicsComplete));
}


fn setup_terrain_physics(
    mut commands: Commands,
    mut terrain_scene: ResMut<TerrainScene>,
    meshes: ResMut<Assets<Mesh>>,
//    mut materials: ResMut<Assets<StandardMaterial>>,
    query: Query<(Entity, &GlobalTransform, &TerrainEntity), With<TerrainEntity>>,
) {
    for (entity, _global_transform, terrain) in query.iter() {
        if let Some(mesh) = meshes.get(&terrain.mesh.clone()) {
            if mesh.count_vertices() <= 0 {
                continue;
            }

            if let Some(mut collider) = bevy_rapier3d::geometry::Collider::from_bevy_mesh(
                &mesh,
                &ComputedColliderShape::TriMesh,
            ) {
                //make `bevy_mesh` colliders

                println!("terrain setup_terrain_physics collider ok: {}", mesh.count_vertices());

                collider.set_scale(terrain.scale, 7);

                commands
                    .entity(entity)
                    //                    .insert(Transform::from_xyz(100.0, -100.1, 0.0))
                    .insert(collider)
                    .insert(bevy_rapier3d::prelude::RigidBody::Fixed)
                    .insert(Friction::coefficient(0.3))
                    .insert(CollisionGroups::new(0b0001, 0b0111))
                    .insert(SolverGroups::new(0b0001, 0b0111))
 //                   .insert(CustomFilterTag::GroupTerrain)
                    ;

                println!("terrain TerrainPhysicsComplete");
                terrain_scene.loading_state = TerrainState::Complete;
 //               terrain_state.replace(TerrainState::Complete).unwrap();
            }
        }
    }
}

pub fn get_pos_on_ground(pos: Vec3, rapier_context: &RapierContext) -> Option<Vec3> {
 //   println!("terrain get_pos_on_ground");

    let result = rapier_context.cast_ray(
        Vec3::new(pos.x, 2000., pos.z),
        Vec3::new(0., -1., 0.),
        f32::MAX,
        true,
        QueryFilter {
            flags: bevy_rapier3d::rapier::prelude::QueryFilterFlags::EXCLUDE_SENSORS,
            groups: Some(InteractionGroups::new(0b0001, 0b0001)),
            exclude_collider: None,
            exclude_rigid_body: None,
            predicate: None,
        }
    );

    /*    let result = physics_world.ray_cast(
            Vec3::new(pos.x, pos.y + 1000., pos.z),
            Vec3::new(0., -2000., 0.),
            true,
        );
    */

    if let Some((_entity, _toi)) = result {
//        println!("terrain get_pos_on_ground ok {}", Vec3::new(pos.x, pos.y + 2000. - _toi, pos.z));
        return Some(Vec3::new(pos.x, pos.y + 2000. - _toi, pos.z));
    }

    println!("terrain get_pos_on_ground fail");

    None
}


/*

#[derive(PartialEq, Eq, Clone, Copy, Component)]
enum CustomFilterTag {
    GroupTerrain,
    GroupShot,
}

// A custom filter that allows contacts only between rigid-bodies with the
// same user_data value.
// Note that using collision groups would be a more efficient way of doing
// this, but we use custom filters instead for demonstration purpose.
struct SameUserDataFilter;
impl<'a> PhysicsHooksWithQuery<&'a CustomFilterTag> for SameUserDataFilter {
    fn filter_contact_pair(
        &self,
        context: PairFilterContextView,
        tags: &Query<&'a CustomFilterTag>,
    ) -> Option<SolverFlags> {
        //     println!("PhysicsHooksWithQuery start");

        if tags.get(context.collider1()).ok().copied()
            != tags.get(context.collider2()).ok().copied()
        {
            //         println!("filter_contact_pair match");

            return Some(SolverFlags::empty());
        }

        Some(SolverFlags::COMPUTE_IMPULSES)
    }
    /*
    fn filter_intersection_pair(
        &self,
        context: PairFilterContextView,
        tags: &Query<&'a CustomFilterTag>,
    ) -> bool {
        if tags.get(context.collider1()).ok().copied()
            != tags.get(context.collider2()).ok().copied() {

            println!("filter_intersection_pair match");

            return true;
        }

        false
    }*/
}

*/
