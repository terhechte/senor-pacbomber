use crate::GameState;
use bevy::prelude::*;

pub struct LoadingPlugin;

#[derive(Component)]
struct LocalEntity;

impl Plugin for LoadingPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(SystemSet::on_enter(GameState::Loading).with_system(setup))
            .add_system_set(SystemSet::on_exit(GameState::Loading).with_system(exit))
            .add_system_set(SystemSet::on_update(GameState::Loading).with_system(forward));
    }
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn_bundle(NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                padding: UiRect::all(Val::Percent(3.)),
                ..default()
            },
            color: Color::NONE.into(),
            ..default()
        })
        .with_children(|parent| {
            parent.spawn_bundle(TextBundle::from_section(
                "Loading...",
                TextStyle {
                    font: asset_server.load("fonts/Archivo-Bold.ttf"),
                    font_size: 50.0,
                    color: Color::rgb(1.0, 1.0, 0.0),
                },
            ));
        })
        .insert(LocalEntity);
}

fn exit(mut commands: Commands, destroy_query: Query<Entity, With<LocalEntity>>) {
    for entity in destroy_query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

// In wasm builds, loading the level blocks. Display a loading screen before the
// blocking begins
fn forward(mut state: ResMut<State<GameState>>, mut enter_time: Local<f32>, time: Res<Time>) {
    if *enter_time <= 0.0 {
        *enter_time = time.time_since_startup().as_secs_f32();
        return;
    }

    let diff = time.time_since_startup().as_secs_f32() - *enter_time;
    if diff > 0.15 && state.current() != &GameState::Game {
        state.set(GameState::Game).unwrap();
    }
}
