use bevy::prelude::*;

use super::types::{CurrentLevel, Score};

#[derive(Component)]
pub struct UiComponent;

#[derive(Component)]
pub struct BombLabel;

#[derive(Component)]
pub struct LevelLabel;

#[derive(Component)]
pub struct PointLabel;

pub fn setup_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn_bundle(NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(100.0), Val::Auto),
                padding: UiRect::all(Val::Percent(2.)),
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                align_self: AlignSelf::FlexStart,
                ..default()
            },
            color: Color::NONE.into(),
            ..default()
        })
        .with_children(|parent| {
            parent
                .spawn_bundle(TextBundle::from_section(
                    "",
                    TextStyle {
                        font: asset_server.load("fonts/Archivo-SemiBold.ttf"),
                        font_size: 20.0,
                        color: Color::rgb(0.0, 0.9, 0.9),
                    },
                ))
                .insert(BombLabel);
            parent
                .spawn_bundle(TextBundle::from_section(
                    "Level 1",
                    TextStyle {
                        font: asset_server.load("fonts/Archivo-SemiBold.ttf"),
                        font_size: 20.0,
                        color: Color::rgb(0.9, 0.9, 0.9),
                    },
                ))
                .insert(LevelLabel);
            parent
                .spawn_bundle(TextBundle::from_section(
                    "#0",
                    TextStyle {
                        font: asset_server.load("fonts/Archivo-SemiBold.ttf"),
                        font_size: 20.0,
                        color: Color::rgb(0.9, 0.9, 0.0),
                    },
                ))
                .insert(PointLabel);
        })
        .insert(UiComponent);
}

pub fn cleanup_ui(mut commands: Commands, query: Query<Entity, With<UiComponent>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

pub fn update_ui_bombs(score: Res<Score>, mut query: Query<&mut Text, With<BombLabel>>) {
    query.single_mut().sections[0].value = format!("Bombs x{}", score.bombs);
}

pub fn update_ui_score(score: Res<Score>, mut query: Query<&mut Text, With<PointLabel>>) {
    query.single_mut().sections[0].value = format!("#{}", score.coins);
}

pub fn update_ui_level(level: Res<CurrentLevel>, mut query: Query<&mut Text, With<LevelLabel>>) {
    query.single_mut().sections[0].value = format!("Level {}", level.0 + 1);
}
