use std::{collections::HashMap, marker::PhantomData};

use ::std::convert::{TryFrom, TryInto};

use crate::AppState;
use bevy::prelude::*;
use core::fmt::Debug;
use std::cmp::Eq;
use core::hash::Hash;

pub struct InputPlugin<T> (pub PhantomData<T>)
where T: 'static + Send + Sync + Default + Debug + Eq + Hash + Clone + TryFrom<u16> + TryInto<u16>;

impl<T> Plugin for InputPlugin<T>
where T: 'static + Send + Sync + Default + Debug + Eq + Hash + Clone + TryFrom<u16> + TryInto<u16> {
    fn build(&self, app: &mut App) {
        let before_system_set = SystemSet::on_update(AppState::Playing)
            .with_system(update_input::<T>);//.before(rotate_turret_by_key));

        app
            .init_resource::<GameControl<T>>()
//            .add_system_set_to_stage(CoreStage::PreUpdate, 
//                                    State::<AppState>::get_driver())
            .add_system_set_to_stage(CoreStage::PreUpdate, before_system_set)
        ;
    }
}

impl<T> Default for InputPlugin<T> 
where T: 'static + Send + Sync + Default + Debug + Eq + Hash + Clone + TryFrom<u16> + TryInto<u16> {
    fn default() -> Self {
        InputPlugin(PhantomData::<T>)
    }
}

fn update_input<T> (
    keyboard_input: Res<Input<KeyCode>>,    
    mut game_control: ResMut<GameControl<T>>,
) where T: 'static + Send + Sync + Default + Debug + Eq + Hash + Clone + TryFrom<u16> + TryInto<u16> {
    game_control.obr_input(&keyboard_input);
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum KeyState {
    Released,
    JustPressed,
    Pressed,
    JustReleased,    
}

#[derive(Component, Debug)]
pub struct GameControl <T>
where T: 'static + Send + Sync + Default + Debug + Eq + Hash + Clone + TryFrom<u16> + TryInto<u16> {
    keys: HashMap<T, Vec<KeyCode>>,
    states: HashMap<T, KeyState>,
}

impl<T> Default for GameControl<T> 
where T: 'static + Send + Sync + Default + Debug + Eq + Hash + Clone + TryFrom<u16> + TryInto<u16> {
    fn default() -> Self {
        GameControl::new()
    }
}

impl <T> GameControl <T> 
where T: 'static + Send + Sync + Default + Debug + Eq + Hash + Clone + TryFrom<u16> + TryInto<u16> {
    pub fn new() -> Self { 
        Self {
            keys: HashMap::new(),
            states: HashMap::new(),
        }
    }

    pub fn add_key_action (&mut self, action_name: T, key_code: KeyCode) {
        if let Some(key_codes) = self.keys.get(&action_name) {
            let mut new_keys = key_codes.clone();
            new_keys.push(key_code);
            self.keys.insert(action_name, new_keys);
        } else {
            self.keys.insert(action_name, vec![key_code]);
        }
    }

    pub fn get_key_state (&self, action_name: T) -> Option<&KeyState> {
        self.states.get(&action_name)
    }

    pub fn get_key_states (&self) -> u16 {
        assert!(self.states.keys().count() < 16);

        let mut res = 0;
        let mut shift = 0;   

        for code in 0u16..16u16 {
            if let Ok(action) = <u16 as TryInto<T>>::try_into(code) {
                res |= match self.states.get(&action) {
                    Some(KeyState::JustPressed) | Some(KeyState::Pressed) => 1,
                    Some(_) | None => 0,
                } << shift;
                shift += 1;
            }
        }
/*      
        for (actions, values) in &self.states {            
            res |= match values {
                KeyState::JustPressed | KeyState::Pressed => 1,
                _ => 0,
            } << shift;
            shift += 1;
        }
*/
        res
    }

    pub fn set_key_states (&mut self, value: u16) {
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
/*      
        for (actions, values) in &self.states {            
            res |= match values {
                KeyState::JustPressed | KeyState::Pressed => 1,
                _ => 0,
            } << shift;
            shift += 1;
        }
*/
    }

    fn obr_input (&mut self, keyboard_input: &Res<Input<KeyCode>>) {
    'outer: for (action_name, key_codes) in &self.keys {          
            for key_code in key_codes.iter() {
                if keyboard_input.just_pressed(*key_code) {
                    self.states.insert(action_name.clone(), KeyState::JustPressed);
                    continue 'outer;
                }
            }

            for key_code in key_codes.iter() {
                if keyboard_input.pressed(*key_code) {
                    self.states.insert(action_name.clone(), KeyState::Pressed);
                    continue 'outer;
                }
            }

            for key_code in key_codes.iter() {
                if keyboard_input.just_released(*key_code) {
                    self.states.insert(action_name.clone(), KeyState::JustReleased);
                    continue 'outer;
                }
            }

            self.states.insert(action_name.clone(), KeyState::Released);
        }
    }
}

