use crate::GameState;
use bevy::prelude::*;

pub struct MenuPlugin;

#[derive(Component)]
struct LocalEntity;

const NORMAL_BUTTON: Color = Color::rgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::rgb(0.25, 0.25, 0.25);
const PRESSED_BUTTON: Color = Color::rgb(0.35, 0.75, 0.35);

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(SystemSet::on_enter(GameState::Menu).with_system(setup))
            .add_system_set(SystemSet::on_exit(GameState::Menu).with_system(exit))
            .add_system_set(
                SystemSet::on_update(GameState::Menu)
                    .with_system(keyboard_input_system)
                    .with_system(button_system),
            );
    }
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn_bundle(NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                justify_content: JustifyContent::SpaceBetween,
                ..default()
            },
            color: Color::NONE.into(),
            ..default()
        })
        .with_children(|parent| {
            parent
                .spawn_bundle(NodeBundle {
                    style: Style {
                        size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                        position_type: PositionType::Absolute,
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        flex_direction: FlexDirection::Column,
                        ..default()
                    },
                    color: Color::NONE.into(),
                    ..default()
                })
                .with_children(|parent| {
                    // start button
                    parent
                        .spawn_bundle(ButtonBundle {
                            style: Style {
                                size: Size::new(Val::Px(150.0), Val::Px(65.0)),
                                // center button
                                margin: UiRect::all(Val::Auto),
                                // horizontally center child text
                                justify_content: JustifyContent::Center,
                                // vertically center child text
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            color: NORMAL_BUTTON.into(),
                            ..default()
                        })
                        .with_children(|parent| {
                            parent.spawn_bundle(TextBundle::from_section(
                                "Start",
                                TextStyle {
                                    font: asset_server.load("fonts/Archivo-Bold.ttf"),
                                    font_size: 20.0,
                                    color: Color::rgb(0.9, 0.9, 0.9),
                                },
                            ));
                        });
                    // bevy logo (image)
                    parent.spawn_bundle(ImageBundle {
                        style: Style {
                            size: Size::new(Val::Px(900.0), Val::Px(538.0)),
                            ..default()
                        },
                        image: asset_server.load("images/title.png").into(),
                        ..default()
                    });
                });
        })
        .insert(LocalEntity);
}

fn exit(mut commands: Commands, destroy_query: Query<Entity, With<LocalEntity>>) {
    for entity in destroy_query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

fn keyboard_input_system(
    keyboard_input: Res<Input<KeyCode>>,
    mut app_state: ResMut<State<GameState>>,
) {
    if keyboard_input.pressed(KeyCode::Return) {
        app_state.set(GameState::Game).unwrap();
    }
}

#[allow(clippy::complexity)]
fn button_system(
    mut interaction_query: Query<
        (&Interaction, &mut UiColor, &Children),
        (Changed<Interaction>, With<Button>),
    >,
    mut text_query: Query<&mut Text>,
    mut app_state: ResMut<State<GameState>>,
) {
    for (interaction, mut color, children) in &mut interaction_query {
        let mut text = text_query.get_mut(children[0]).unwrap();
        match *interaction {
            Interaction::Clicked => {
                *color = PRESSED_BUTTON.into();
                app_state.set(GameState::Game).unwrap();
            }
            Interaction::Hovered => {
                *color = HOVERED_BUTTON.into();
            }
            Interaction::None => {
                text.sections[0].value = "Start".to_string();
                *color = NORMAL_BUTTON.into();
            }
        }
    }
}
