mod level;
mod logic;
mod statics;
mod types;
pub mod ui;

use bevy::prelude::*;

use self::types::{GoNextLevelEvent, PlayerDiedEvent, ShowLevelExitEvent};

use super::GameState;

pub use statics::sizes;
pub use types::{BlockType, CurrentLevel, Score};

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ShowLevelExitEvent>()
            .add_event::<GoNextLevelEvent>()
            .add_event::<PlayerDiedEvent>()
            .add_system_set(SystemSet::on_enter(GameState::Game).with_system(ui::setup_ui))
            .add_system_set(SystemSet::on_update(GameState::Game).with_system(logic::level_loading))
            .add_system_set(SystemSet::on_enter(GameState::Game).with_system(logic::first_level))
            .add_system_set(
                SystemSet::on_exit(GameState::Running).with_system(logic::cleanup_level),
            )
            .add_system_set(SystemSet::on_exit(GameState::Running).with_system(ui::cleanup_ui))
            .add_system_set(
                SystemSet::on_update(GameState::Running)
                    .with_system(logic::level_loading)
                    .with_system(logic::wobble)
                    .with_system(logic::wobble_enemy)
                    .with_system(logic::keyboard_input_system)
                    .with_system(logic::wall_visibility)
                    .with_system(logic::update_level)
                    .with_system(logic::tween_done_remove_handler)
                    .with_system(logic::bomb_counter)
                    .with_system(logic::bomb_explosion_destruction)
                    .with_system(logic::enemy_logic)
                    .with_system(logic::move_entities)
                    .with_system(logic::show_level_exit)
                    .with_system(logic::player_did_die_system)
                    .with_system(logic::finish_level)
                    .with_system(ui::update_ui_bombs)
                    .with_system(ui::update_ui_level)
                    .with_system(ui::update_ui_score),
            );
    }
}
