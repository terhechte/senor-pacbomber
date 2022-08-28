mod level;
mod logic;
mod statics;
mod types;

use bevy::{prelude::*, time::FixedTimestep};
use bevy_mod_outline::*;
use bevy_tweening::TweeningPlugin;

use super::GameState;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        dbg!("BUILD");
        app.insert_resource(types::Score::default())
            .insert_resource(level::Level::new(statics::LEVEL_DATA))
            .add_plugin(OutlinePlugin)
            .add_plugin(TweeningPlugin)
            // .add_startup_system(logic::setup)
            .add_system_set(SystemSet::on_enter(GameState::Game).with_system(logic::setup))
            .add_system_set(
                SystemSet::on_update(GameState::Game)
                    .with_system(logic::wobble)
                    .with_system(logic::keyboard_input_system)
                    .with_system(logic::wall_visibility)
                    .with_system(logic::update_level)
                    .with_system(logic::tween_done_remove_handler)
                    .with_system(logic::bomb_counter)
                    .with_system(logic::bomb_explosion_destruction)
                    .with_system(logic::enemy_logic)
                    .with_system(logic::move_entities),
            );
    }
}
