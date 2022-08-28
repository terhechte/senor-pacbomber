mod level;
mod logic;
mod statics;
mod types;

use bevy::prelude::*;
use bevy_mod_outline::*;
use bevy_tweening::TweeningPlugin;

use self::types::{GoNextLevelEvent, LevelEntity, ShowLevelExitEvent};

use super::GameState;

pub use types::MaterialHandles;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(types::Score::default())
            .insert_resource(level::Level::new(statics::LEVEL_DATA))
            .add_plugin(OutlinePlugin)
            .add_plugin(TweeningPlugin)
            .add_event::<ShowLevelExitEvent>()
            .add_event::<GoNextLevelEvent>()
            .add_system_set(SystemSet::on_enter(GameState::Game).with_system(logic::first_level))
            // .add_system_set(SystemSet::on_exit(GameState::Game).with_system(logic::exit))
            .add_system_set(
                SystemSet::on_update(GameState::Game)
                    .with_system(logic::level_loading)
                    .with_system(logic::wobble)
                    .with_system(logic::keyboard_input_system)
                    .with_system(logic::wall_visibility)
                    .with_system(logic::update_level)
                    .with_system(logic::tween_done_remove_handler)
                    .with_system(logic::bomb_counter)
                    .with_system(logic::bomb_explosion_destruction)
                    .with_system(logic::enemy_logic)
                    .with_system(logic::move_entities)
                    .with_system(logic::show_level_exit)
                    .with_system(logic::finish_level),
            );
    }
}

pub fn exit(
    q_parent: Query<(&LevelEntity, &Children)>,
    q_child: Query<Entity>,
    mut commands: Commands,
    mut reader: EventReader<GoNextLevelEvent>,
) {
    let event = match reader.iter().next() {
        Some(n) => n,
        None => return,
    };
    println!("PROCESS EXIT");
    for (_, children) in q_parent.iter() {
        for &child in children.iter() {
            dbg!(child);
        }
        // commands.entity(entity).despawn();
    }
}
