use std::{collections::HashMap, marker::PhantomData};

use std::convert::TryFrom;

use crate::AppState;
use bevy::prelude::*;
use core::fmt::Debug;
use core::hash::Hash;
use std::cmp::Eq;

pub struct InputPlugin<T>(pub PhantomData<T>)
where
    T: 'static + Send + Sync + Default + Debug + Eq + Hash + Clone + TryFrom<u16> + TryInto<u16>;

impl<T> Plugin for InputPlugin<T>
where
    T: 'static + Send + Sync + Default + Debug + Eq + Hash + Clone + TryFrom<u16> + TryInto<u16>,
{
    fn build(&self, app: &mut App) {
        let before_system_set =
            SystemSet::on_update(AppState::Playing).with_system(update_input::<T>);

        app.init_resource::<GameControl<T>>()
            .add_system_set_to_stage(
                CoreStage::PreUpdate,
                before_system_set.label("keys_input").before("player_input"),
            );
    }
}

impl<T> Default for InputPlugin<T>
where
    T: 'static + Send + Sync + Default + Debug + Eq + Hash + Clone + TryFrom<u16> + TryInto<u16>,
{
    fn default() -> Self {
        InputPlugin(PhantomData::<T>)
    }
}

fn update_input<T>(
    time: Res<Time>,
    keyboard_input: Res<Input<KeyCode>>,
    mouse_input: Res<Input<MouseButton>>,
    mut game_control: ResMut<GameControl<T>>,
) where
    T: 'static + Send + Sync + Default + Debug + Eq + Hash + Clone + TryFrom<u16> + TryInto<u16>,
{
    game_control.process_input(&keyboard_input, &mouse_input, time.elapsed_seconds());
}

#[derive(Debug, Default, Eq, PartialEq, Hash, Copy, Clone, PartialOrd, Ord)]
pub struct KeyState {
    pub just_pressed: bool,
    pub pressed: bool,
    pub just_released: bool,
    pub time: u32,
}

impl KeyState {
    pub fn set_time(&mut self, time: f32) {
        self.time = (time * 1000.) as u32;
    }
    pub fn get_time(&self) -> f32 {
        (self.time as f32) / 1000. 
    }
}

#[derive(Debug)]
pub enum InputAction {
    Key(KeyCode),
    Mouse(MouseButton),
}

impl From<KeyCode> for InputAction {
    fn from(key_code: KeyCode) -> Self {
        InputAction::Key(key_code)
    }
}

impl From<MouseButton> for InputAction {
    fn from(mouse_button: MouseButton) -> Self {
        InputAction::Mouse(mouse_button)
    }
}

#[derive(Component, Resource, Debug)]
pub struct GameControl<T>
where
    T: 'static + Send + Sync + Default + Debug + Eq + Hash + Clone + TryFrom<u16> + TryInto<u16>,
{
    keys: HashMap<T, Vec<InputAction>>,
    states: HashMap<T, KeyState>,
}

impl<T> Default for GameControl<T>
where
    T: 'static + Send + Sync + Default + Debug + Eq + Hash + Clone + TryFrom<u16> + TryInto<u16>,
{
    fn default() -> Self {
        GameControl::new()
    }
}

impl<T> GameControl<T>
where
    T: 'static + Send + Sync + Default + Debug + Eq + Hash + Clone + TryFrom<u16> + TryInto<u16>,
{
    pub fn new() -> Self {
        Self {
            keys: HashMap::new(),
            states: HashMap::new(),
        }
    }

    pub fn add_key_action(&mut self, name: T, key: KeyCode) {
        self.add_action(name, InputAction::from(key));
    }

    pub fn add_mouse_action(&mut self, name: T, button: MouseButton) {
        //        log::info!("add_mouse_action {:?}", button);

        self.add_action(name, InputAction::from(button));
    }

    pub fn add_action(&mut self, name: T, action: InputAction) {
        //        log::info!("add_action {:?}", action);

        if let Some(actions) = self.keys.get_mut(&name) {
            actions.push(action);
        //         self.keys.insert(name, *new_actions);
        } else {
            self.keys.insert(name, vec![action]);
        }

        //        log::info!("add_action end {:?}", self.keys);
    }

    pub fn get_key_state(&self, name: T) -> Option<&KeyState> {
        self.states.get(&name)
    }

    /*
        pub fn get_key_states(&self) -> u16 {
            assert!(self.states.keys().count() < 16);

            let mut res = 0;
            let mut shift = 0;

            for code in 0u16..16u16 {
                if let Ok(action) = <u16 as TryInto<T>>::try_into(code) {
                    res |= if let Some(key_state) = self.states.get(&action) {
                        if key_state.just_pressed || key_state.pressed {
                            1
                        } else {
                            0
                        }
                    } else {
                        0
                    } << shift;
                    shift += 1;
                }
        /*        if let Ok(action) = <u16 as TryInto<T>>::try_into(code) {
                    res |= match self.states.get(&action) {
                        Some(KeyState::JustPressed) | Some(KeyState::Pressed) => 1,
                        Some(_) | None => 0,
                    } << shift;
                    shift += 1;
                }
              */
            }

            res
        }

        pub fn set_key_states(&mut self, value: u16) {
            assert!(self.states.keys().count() < 16);

            let mut tmp = 1;

            for code in 0u16..16u16 {
                if let Ok(action) = <u16 as TryInto<T>>::try_into(code) {
                    let pressed = value & tmp == tmp;
                    tmp *= 2;

                    let old_state = self.states.get(&action).unwrap_or(&KeyState::Released);

                    let mut new_state = KeyState::Released;

                    if old_state == &KeyState::JustPressed || old_state == &KeyState::Pressed {
                        if pressed {
                            new_state = KeyState::Pressed;
                        } else {
                            new_state = KeyState::JustReleased;
                        }
                    } else if old_state == &KeyState::JustReleased || old_state == &KeyState::Released {
                        if pressed {
                            new_state = KeyState::JustPressed;
                        } else {
                            new_state = KeyState::Released;
                        }
                    }

                    self.states.insert(action.clone(), new_state);
                }
            }
        }
    */
    fn process_input(
        &mut self,
        keyboard_input: &Res<Input<KeyCode>>,
        mouse_input: &Res<Input<MouseButton>>,
        time: f32,
    ) {
        for (name, actions) in &self.keys {

            let mut key_state = KeyState::default();
            let old_time = if let Some(old_state) = self.states.get(name) {
                old_state.time
            } else {
                0
            };

            for action in actions.iter() {
                //              log::info!("actions process {:?}", action);
                if let InputAction::Key(key_code) = action {
                    if keyboard_input.just_pressed(*key_code) {
                        //                        log::info!("key_code just_pressed {:?}", key_code);
                        key_state.just_pressed = true;
                        key_state.set_time(time);
                    } else if keyboard_input.pressed(*key_code) {
                        key_state.pressed = true;
                        key_state.time = old_time;
                    } else if keyboard_input.just_released(*key_code) {
                        key_state.just_released = true;
                        key_state.set_time(time);
                    } else {
                        key_state.time = old_time;

                        //key_state = KeyState::default();
                    }

                    continue;
                }

                if let InputAction::Mouse(mouse_button) = action {
                    //                    log::info!("mouse_button process {:?}", mouse_button);
                    if mouse_input.just_pressed(*mouse_button) {
                        //                       log::info!("mouse_button just_pressed {:?}", mouse_button);
                        key_state.just_pressed = true;
                        key_state.set_time(time);
                    } else if mouse_input.pressed(*mouse_button) {
                        key_state.pressed = true;
                        key_state.time = old_time;
                    } else if mouse_input.just_released(*mouse_button) {
                        //                       log::info!("mouse_button just_released {:?}", mouse_button);
                        key_state.just_released = true;
                        key_state.set_time(time);
                    } else {
                        //  key_state = KeyState::default();
                        key_state.time = old_time;
                    }

                    continue;
                }
            }

            self.states.insert(name.clone(), key_state);
        }
    }
}
