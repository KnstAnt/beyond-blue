use crate::loading::FontAssets;
use crate::AppState;
use bevy::app::AppExit;
use bevy::prelude::*;
use bevy::text::{Text, TextStyle};

use iyes_loopless::prelude::ConditionSet;
use iyes_loopless::prelude::IntoConditionalSystem;

#[path = "cleanup.rs"] 
mod cleanup;
use cleanup::cleanup_system;



pub struct MenuPlugin;

#[derive(Component)]
pub struct MenuClose;   

#[derive(Component)]
pub struct MainMenu;

#[derive(Component)]
pub struct StartLocalButton;

#[derive(Component)]
pub struct StartNetButton;

#[derive(Component)]
pub struct StartTestButton;

#[derive(Component)]
pub struct ExitButton;



#[derive(Clone, Eq, PartialEq, Debug, Hash)]
pub enum MenuState {
    None,
    Local,
    Network,
    Test,
}

struct ButtonColors {
    clicked: UiColor,
    hovered: UiColor,
    normal: UiColor,
}

impl Default for ButtonColors {
    fn default() -> Self {
        ButtonColors {
            clicked: Color::rgb(0.5, 0.5, 0.5).into(),
            hovered: Color::rgb(0.25, 0.25, 0.25).into(),
            normal: Color::rgb(0.15, 0.15, 0.15).into(),            
        }
    }
}

#[derive(Debug)]
pub struct MenuData {
    state: MenuState,
}
 
impl Default for MenuData {
    fn default() -> Self {
        Self {
            state: MenuState::None,
        }
    }
}
/// This plugin is responsible for the game menu (containing only one button...)
/// The menu is only drawn during the State `AppState::Menu` and is removed when that state is exited
impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<ButtonColors>()
            .init_resource::<MenuData>()
            .add_system_set(SystemSet::on_enter(AppState::Menu).with_system(setup_menu))
            .add_system_set(SystemSet::on_update(AppState::Menu)
                .with_system(obr_buttons_visual)
                .with_system(start_local_game.run_if(on_buttons_action::<StartLocalButton>))  
                .with_system(start_net_game.run_if(on_buttons_action::<StartNetButton>))          
                .with_system(start_test.run_if(on_buttons_action::<StartTestButton>))       
                .with_system(exit_system.run_if(on_buttons_action::<ExitButton>)))
            .add_system_set(SystemSet::on_exit(AppState::Menu).with_system(cleanup_system::<MenuClose>))
            .add_system_set(
                ConditionSet::new()
                .run_if(on_buttons_action::<StartLocalButton>)
                .run_if(on_buttons_action::<StartNetButton>)
                .run_if(on_buttons_action::<ExitButton>)
                .into() 
            );
    }
}

fn setup_menu(
    mut commands: Commands,
    font_assets: Res<FontAssets>,
    button_colors: Res<ButtonColors>,
) {
   let title_style = TextStyle {
        font: font_assets.fira_sans.clone(),
        font_size: 30.0,
        color: Color::rgb(0.9, 0.9, 0.9),
    };

    let button_style = Style {
        size: Size::new(Val::Auto, Val::Auto),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        padding: UiRect::all(Val::Px(2.0)),
        margin: UiRect::all(Val::Px(4.0)),
        flex_grow: 1.0,
        ..Default::default()
    };
    
    commands.spawn_bundle(Camera2dBundle::default());

    commands.spawn_bundle(NodeBundle {
        color: UiColor(Color::rgb(0.0, 0.0, 0.0)),
        style: Style {
            size: Size::new(Val::Auto, Val::Auto),
            margin: UiRect::all(Val::Auto),
            align_self: AlignSelf::Center,
            flex_direction: FlexDirection::ColumnReverse,
            justify_content: JustifyContent::Center,
            ..Default::default()
        },
        ..Default::default()
    })
    .insert(MainMenu)
    .with_children(|menu| {
        menu.spawn_bundle(ButtonBundle {
            style: button_style.clone(),
            color: button_colors.normal,
            ..Default::default()
        })
        .insert(StartLocalButton)
        .with_children(|btn| {
            btn.spawn_bundle(TextBundle {
                text: Text::from_section("Start Local Game", title_style.clone()),
                ..Default::default()
            });
        });

        menu.spawn_bundle(ButtonBundle {
            style: button_style.clone(),
            color: button_colors.normal,
            ..Default::default()
        })
        .insert(StartNetButton)
        .with_children(|btn| {
            btn.spawn_bundle(TextBundle {
                text: Text::from_section("Start Network Game", title_style.clone()),
                ..Default::default()
            });
        });

        menu.spawn_bundle(ButtonBundle {
            style: button_style.clone(),
            color: button_colors.normal,
            ..Default::default()
        })
        .insert(StartTestButton)
        .with_children(|btn| {
            btn.spawn_bundle(TextBundle {
                text: Text::from_section("Start Tests", title_style.clone()),
                ..Default::default()
            });
        });

        menu.spawn_bundle(ButtonBundle {
            style: button_style.clone(),
            color: button_colors.normal,
            ..Default::default()
        })
        .insert(ExitButton)
        .with_children(|btn| {
            btn.spawn_bundle(TextBundle {
                text: Text::from_section("Exit Game", title_style.clone()),
                ..Default::default()
            });
        });
    });

/* 
    commands
        .spawn_bundle(ButtonBundle {
            style: Style {
                size: Size::new(Val::Px(120.0), Val::Px(50.0)),
                margin: UiRect::all(Val::Auto),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..Default::default()
            },
            color: button_colors.normal,
            ..Default::default()
        })
        .with_children(|parent| {
            parent.spawn_bundle(TextBundle {
                text: Text {
                    sections: vec![TextSection {
                        value: "Play".to_string(),
                        style: TextStyle {
                            font: font_assets.fira_sans.clone(),
                            font_size: 40.0,
                            color: Color::rgb(0.9, 0.9, 0.9),
                        },
                    }],
                    alignment: Default::default(),
                },
                ..Default::default()
            });
        });
*/ 
}

fn obr_buttons_visual(
    mut commands: Commands,
    button_colors: Res<ButtonColors>,
    mut interaction_query: Query<
        (&Interaction, &mut UiColor),
        (Changed<Interaction>, With<Button>),
    >,
) {
    for (interaction, mut color) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Clicked => {
                *color = button_colors.clicked;
            }
            Interaction::Hovered => {
                *color = button_colors.hovered;
            }
            Interaction::None => {
                *color = button_colors.normal;
            }
        }
    }
}

fn on_buttons_action<T: Component>(
    mut commands: Commands,
    query: Query<&Interaction, (Changed<Interaction>, With<Button>, With<T>)>,
    menu_query: Query <Entity, With<MainMenu>>,
    camera_query: Query <Entity, With<Camera>>,
) -> bool {
    for interaction in query.iter() {
        if *interaction == Interaction::Clicked {

            if let Ok(main_menu) = menu_query.get_single() {
                commands.entity(main_menu).despawn_recursive();
            }

            if let Ok(camera) = camera_query.get_single() {
                commands.entity(camera).despawn_recursive();
            }

            return true;
        }
    }

    false
}

fn start_local_game(
    mut app_state: ResMut<State<AppState>>,
    mut menu_data: ResMut<MenuData>,) {
        menu_data.state = MenuState::Local;
        app_state.replace(AppState::PreparePlaying).unwrap();
}

fn start_net_game(
    mut app_state: ResMut<State<AppState>>,
    mut menu_data: ResMut<MenuData>,) {
        menu_data.state = MenuState::Network;
        app_state.replace(AppState::Connecting).unwrap();
}
fn start_test(
    mut app_state: ResMut<State<AppState>>,
    mut menu_data: ResMut<MenuData>,) {
        menu_data.state = MenuState::Test;
        app_state.replace(AppState::Test).unwrap();
}

fn exit_system(mut exit: EventWriter<AppExit>) {
    exit.send(AppExit);
}

pub fn is_play_online(menu_data: Res<MenuData>) -> bool {
    //   println!("terrain is_create_assets");
    menu_data.state == MenuState::Network
}
   
pub fn is_play_offline(menu_data: Res<MenuData>) -> bool {
    //   println!("terrain is_create_physics");
    menu_data.state == MenuState::Local
}