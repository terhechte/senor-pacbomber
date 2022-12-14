use bevy::{audio::AudioSink, prelude::*};
use bevy_mod_outline::*;
use bevy_tweening::{
    lens::{TransformPositionLens, TransformRotationLens, TransformScaleLens},
    Animator, Delay, EaseFunction, Sequence, Tracks, Tween, TweenCompleted, TweeningType,
};
use std::{cmp::Ordering, f32::consts::TAU, time::Duration};

use crate::{
    types::{AudioHandles, CurrentMusic},
    GameState, MaterialHandles, MeshHandles,
};

use super::statics::{self, sizes, FPS, LEVEL_BOMBS, USER_DIED_PAYLOAD};
use super::types::*;
use super::{level::Level, statics::LEVEL_COMPLETED_PAYLOAD};

pub fn first_level(
    mut commands: Commands,
    audio_sinks: Res<Assets<AudioSink>>,
    audio: Res<Audio>,
    mut playback: ResMut<CurrentMusic>,
    audio_handles: Res<AudioHandles>,
) {
    commands.insert_resource(super::level::Level::new(0));
    commands.insert_resource(CurrentLevel(0));
    commands.insert_resource(super::types::Score::default());

    if let Some(sink) = audio_sinks.get(&playback.0) {
        sink.stop();
    }

    // to simplify things, we just continue playing music from now on
    let weak_handle = audio.play_with_settings(
        audio_handles.music.clone(),
        PlaybackSettings::LOOP.with_volume(0.5),
    );
    let strong_handle = audio_sinks.get_handle(weak_handle);
    playback.0 = strong_handle;
}

pub fn level_loading(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut level: ResMut<Level>,
    current_level: Res<CurrentLevel>,
    material_handles: Res<MaterialHandles>,
    mesh_handles: Res<MeshHandles>,
    mut state: ResMut<State<GameState>>,
) {
    // only setup a new level if the level changed
    if !current_level.is_changed() {
        return;
    }

    if level.done_loading {
        return;
    }

    let mut enemies = Vec::new();
    let mut coins = Vec::new();

    let mut children = Vec::new();

    for row in level.rows() {
        for block in row.iter() {
            // Each entry also needs a floor
            let is_exit = matches!(block.kind, BlockType::Exit);
            children.push(setup_space(
                &mut commands,
                &mesh_handles,
                &material_handles,
                (block.position.x, block.position.z),
                is_exit,
            ));
            match block.kind {
                BlockType::WallBig => children.push(setup_wall(
                    &mut commands,
                    mesh_handles.wall.clone(),
                    &material_handles,
                    block,
                )),
                BlockType::WallSmallV => children.push(setup_wall(
                    &mut commands,
                    mesh_handles.wall_v.clone(),
                    &material_handles,
                    block,
                )),
                BlockType::WallSmallH => children.push(setup_wall(
                    &mut commands,
                    mesh_handles.wall_h.clone(),
                    &material_handles,
                    block,
                )),
                BlockType::Coin => {
                    let id = setup_coin(&mut commands, &mesh_handles, &material_handles, block);
                    coins.push((id, block.level_position));
                    children.push(id);
                }
                BlockType::Player => children.push(setup_player(
                    &mut commands,
                    &mut meshes,
                    &material_handles,
                    block,
                )),
                BlockType::Enemy => {
                    let id = setup_enemy(&mut commands, &mesh_handles, &material_handles, block);
                    enemies.push((id, block.level_position));
                    children.push(id);
                }
                BlockType::Space => {}
                BlockType::Exit => {
                    let p = block.position;
                    let id = commands
                        .spawn_bundle(SpotLightBundle {
                            spot_light: SpotLight {
                                intensity: 100.0,
                                shadows_enabled: true,
                                radius: 8.0,
                                ..default()
                            },
                            transform: Transform::from_xyz(p.x, 0.2, p.z).looking_at(p, Vec3::Z),
                            visibility: Visibility { is_visible: false },
                            ..default()
                        })
                        .insert(ExitLight)
                        .id();
                    children.push(id);
                }
            }
        }
    }

    for (id, pos) in enemies {
        level.enemy_positions.insert(id, pos);
    }

    for (id, pos) in coins {
        level.coin_positions.insert(id, pos);
    }

    for id in children {
        commands.entity(id).insert(LevelItem);
    }
    level.done_loading = true;
    if state.current() != &GameState::Running {
        state.set(GameState::Running).unwrap();
    }
}

pub fn finish_level(
    mut commands: Commands,
    mut reader: EventReader<GoNextLevelEvent>,
    query: Query<Entity, With<LevelItem>>,
    current: ResMut<CurrentLevel>,
    mut app_state: ResMut<State<GameState>>,
    mut score: ResMut<Score>,
) {
    for _ in reader.iter() {
        for entity in query.iter() {
            commands.entity(entity).despawn_recursive();
        }
        let next = match current.next() {
            Some(n) => n,
            None => {
                // Transition to the ending
                app_state.set(GameState::Won).unwrap();
                return;
            }
        };

        // replenish the bombs
        score.bombs = LEVEL_BOMBS[next.0];

        commands.insert_resource(super::level::Level::new(next.0));
        commands.insert_resource(next);
    }
}

pub fn cleanup_level(mut commands: Commands, query: Query<Entity, With<LevelItem>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

pub fn setup_wall(
    commands: &mut Commands,
    mesh: Handle<Mesh>,
    materials: &MaterialHandles,
    block: &Block,
) -> Entity {
    let p = block.position;
    let s = block.kind.size();
    commands
        .spawn_bundle(PbrBundle {
            mesh,
            material: materials.wall_normal.clone(),
            transform: Transform::from_xyz(p.x, p.y, p.z),
            ..default()
        })
        .insert(Size(s))
        .insert(Location(block.level_position))
        .insert(Wall)
        .id()
}

pub fn setup_coin(
    commands: &mut Commands,
    meshes: &MeshHandles,
    materials: &MaterialHandles,
    block: &Block,
) -> Entity {
    let p = block.position;
    commands
        .spawn_bundle(MaterialMeshBundle {
            mesh: meshes.coin.clone(),
            material: materials.coin.clone(),
            transform: Transform::from_xyz(p.x, p.y, p.z),
            ..default()
        })
        .insert(Wobbles(p.x * p.z))
        .insert(Coin)
        .id()
}

pub fn setup_player(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &MaterialHandles,
    block: &Block,
) -> Entity {
    let s = block.kind.size();
    let p = block.position;
    let mut player_mesh = Mesh::from(shape::Icosphere {
        radius: s.x,
        subdivisions: 1,
    });
    player_mesh.generate_outline_normals().unwrap();
    let id = commands
        .spawn_bundle(PbrBundle {
            mesh: meshes.add(player_mesh),
            material: materials.player.clone(),
            transform: Transform::from_xyz(p.x, 1.0, p.z),
            ..default()
        })
        .insert_bundle(OutlineBundle {
            outline: Outline {
                visible: true,
                colour: Color::rgba(0.0, 1.0, 0.0, 1.0),
                width: 1.0,
            },
            ..default()
        })
        .insert(Size(s))
        .insert(Movement::default())
        .insert(Location(block.level_position))
        .insert(Speed(statics::PLAYER_SPEED))
        .insert(Player)
        .id();
    // add a tween so the player falls into the game
    let tween = Tween::new(
        EaseFunction::BounceOut,
        TweeningType::Once,
        Duration::from_secs_f32(1.0),
        TransformPositionLens {
            start: Vec3::new(p.x, 1.0, p.z),
            end: p,
        },
    );
    commands.entity(id).insert(Animator::new(tween));
    id
}

pub fn setup_enemy(
    commands: &mut Commands,
    meshes: &MeshHandles,
    materials: &MaterialHandles,
    block: &Block,
) -> Entity {
    let s = block.kind.size();
    let p = block.position;
    let parent = commands
        .spawn_bundle(PbrBundle {
            mesh: meshes.enemy.clone(),
            material: materials.enemy.clone(),
            transform: Transform::from_xyz(p.x, p.y, p.z),
            ..default()
        })
        .id();
    let eye1 = commands
        .spawn_bundle(PbrBundle {
            mesh: meshes.enemy_eye.clone(),
            material: materials.white.clone(),
            transform: Transform::from_xyz(0.1, 0.01, 0.01),
            ..default()
        })
        .id();
    let eye2 = commands
        .spawn_bundle(PbrBundle {
            mesh: meshes.enemy_eye.clone(),
            material: materials.white.clone(),
            transform: Transform::from_xyz(-0.1, 0.01, 0.01),
            ..default()
        })
        .id();
    let eye3 = commands
        .spawn_bundle(PbrBundle {
            mesh: meshes.enemy_eye.clone(),
            material: materials.white.clone(),
            transform: Transform::from_xyz(0.01, 0.01, 0.1),
            ..default()
        })
        .id();
    let eye4 = commands
        .spawn_bundle(PbrBundle {
            mesh: meshes.enemy_eye.clone(),
            material: materials.white.clone(),
            transform: Transform::from_xyz(0.01, 0.01, -0.1),
            ..default()
        })
        .id();
    commands
        .entity(parent)
        .push_children(&[eye1, eye2, eye3, eye4])
        .insert(Size(s))
        .insert(Movement::default())
        .insert(Location(block.level_position))
        .insert(Speed(statics::ENEMY_SPEED_EASY))
        .insert(Enemy);
    parent
}

pub fn add_bomb(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &MaterialHandles,
    level_position: Position,
    position: Vec3,
) -> Entity {
    let mesh = Mesh::from(shape::Cube {
        size: sizes::bomb_size,
    });
    let parent = commands
        .spawn_bundle(PbrBundle {
            mesh: meshes.add(mesh),
            material: materials.bomb.clone(),
            transform: Transform::from_xyz(position.x, position.y, position.z),
            ..default()
        })
        .id();
    let head = commands
        .spawn_bundle(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Cube {
                size: sizes::bomb_size / 4.0,
            })),
            material: materials.white.clone(),
            transform: Transform::from_xyz(0.0, sizes::bomb_size / 2.0, 0.0),
            ..default()
        })
        .id();
    let fire = commands
        .spawn_bundle(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Cube {
                size: sizes::bomb_size / 5.0,
            })),
            material: materials.explosion.clone(),
            transform: Transform::from_xyz(0.0, sizes::bomb_size / 2.0 + 0.05, 0.0),
            ..default()
        })
        .id();
    commands
        .entity(parent)
        .push_children(&[head, fire])
        .insert(Location(level_position))
        .insert(Bomb::new());
    parent
}

pub fn add_bomb_explosion(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &MaterialHandles,
    level_position: Position,
    position: Vec3,
) -> Entity {
    let mesh = Mesh::from(shape::Cube {
        size: sizes::bomb_size,
    });
    commands
        .spawn_bundle(PbrBundle {
            mesh: meshes.add(mesh),
            material: materials.explosion.clone(),
            transform: Transform::from_xyz(position.x, position.y, position.z)
                .with_scale(Vec3::ZERO),
            ..default()
        })
        .insert(Location(level_position))
        .insert(BombExplosion)
        .id()
}

pub fn setup_space(
    commands: &mut Commands,
    meshes: &MeshHandles,
    materials: &MaterialHandles,
    position: (f32, f32),
    hides_exit: bool,
) -> Entity {
    let parent = commands
        .spawn_bundle(PbrBundle {
            mesh: meshes.floor_fg.clone(),
            material: materials.floor_fg.clone(),
            transform: Transform::from_xyz(position.0, sizes::space.y - 0.01, position.1),
            ..default()
        })
        .insert(Floor)
        .id();

    if hides_exit {
        commands.entity(parent).insert(Exit);
    }

    let child1 = commands
        .spawn_bundle(PbrBundle {
            mesh: meshes.floor_bg.clone(),
            material: materials.floor_bg.clone(),
            transform: Transform::from_xyz(0.0, 0.01, 0.0),
            ..default()
        })
        .id();
    let child2 = commands
        .spawn_bundle(PbrBundle {
            mesh: meshes.floor_cube.clone(),
            material: materials.ground.clone(),
            transform: Transform::from_xyz(0.0, -sizes::field.y, 0.0),
            ..default()
        })
        .id();
    commands.entity(parent).push_children(&[child1, child2]);
    parent
}

pub fn wobble(mut query: Query<(&mut Transform, &Wobbles)>, timer: Res<Time>, mut t: Local<f32>) {
    let ta = *t;
    *t = (ta + 0.5 * timer.delta_seconds()) % TAU;
    let tb = *t;
    let i1 = tb.cos() - ta.cos();
    let i2 = ta.sin() - tb.sin();
    for (mut transform, Wobbles(value)) in query.iter_mut() {
        transform.rotate(Quat::from_rotation_z(
            TAU * (20.0 + value * 10.) * i1 * timer.delta_seconds(),
        ));
        transform.rotate(Quat::from_rotation_y(
            TAU * (20.0 + value * 15.) * i2 * timer.delta_seconds(),
        ));
    }
}

pub fn wobble_enemy(mut query: Query<&mut Transform, With<Enemy>>, timer: Res<Time>) {
    for mut transform in query.iter_mut() {
        transform.translation.y = (timer.seconds_since_startup().sin() as f32) / 50.0;
    }
}

pub fn enemy_logic(
    mut query: Query<(&mut Movement, &Transform, &Location, &Speed), With<Enemy>>,
    level: Res<Level>,
    player_query: Query<&Transform, With<Player>>,
) {
    // find the player location
    let player_location = match player_query.iter().next() {
        Some(n) => Vec2::new(n.translation.x, n.translation.z),
        None => return,
    };

    for (mut velocity, transform, position, _) in query.iter_mut() {
        // if we're still moving, do nothing
        if velocity.value > 0.0 {
            continue;
        }
        let v = Vec2::new(transform.translation.x, transform.translation.z);
        // find the free directions
        let mut directions = level.free_directions(position.0);
        if directions.is_empty() {
            continue;
        }
        // just to check if a change by this value brings as closer to the player
        let mov = Vec2::new(0.05, 0.05);

        // order directions by pointing towards the player
        directions.sort_unstable_by(|a, b| {
            // apply the direction and return distance
            let ax: Vec2 = v + (*a * mov);
            let bx: Vec2 = v + (*b * mov);
            ax.distance(player_location)
                .partial_cmp(&bx.distance(player_location))
                .unwrap_or(Ordering::Equal)
        });

        // calculate the new velocity value based on the current speed and time
        // the size of the field on the timestep and the speed step
        // let frames = FPS * speed.0;
        // let value = sizes::field.x / frames;
        velocity.direction = directions[0];
        velocity.value = sizes::field.x;
    }
}

pub fn keyboard_input_system(
    mut commands: Commands,
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<(&mut Movement, &Location), With<Player>>,
    mut level: ResMut<Level>,
    mut score: ResMut<Score>,
    material_handles: Res<MaterialHandles>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    for (mut velocity, location) in query.iter_mut() {
        // if we're in movement, do nothing
        if velocity.value > 0.0 {
            continue;
        }
        // make sure we only move into directions we can
        for (code, direction) in [
            (KeyCode::Left, BoardDirection::new(-1, 0)),
            (KeyCode::Right, BoardDirection::new(1, 0)),
            (KeyCode::Up, BoardDirection::new(0, -1)),
            (KeyCode::Down, BoardDirection::new(0, 1)),
        ] {
            if keyboard_input.pressed(code) {
                let directions = level.free_directions(location.0);
                if directions.contains(&direction) {
                    velocity.direction = direction;
                    velocity.value = sizes::field.x;
                    score.moves += 1;
                }
            }
        }
    }
    // if the user tried to place a bomb
    let level_position = level.player_position;
    if keyboard_input.just_pressed(KeyCode::Space) {
        // if we don't have bombs left
        if score.bombs == 0 {
            return;
        }
        score.bombs -= 1;
        // if there is no bomb yet
        for (_, position) in level.bombs.values() {
            if &level_position == position {
                return;
            }
        }
        let position = level.translate_from_position(level_position);
        let id = add_bomb(
            &mut commands,
            &mut meshes,
            &material_handles,
            level_position,
            position,
        );
        level.place_bomb(id, level_position);
    }
}

pub fn move_entities(
    // We need the entities that are being moved
    mut query: Query<(&mut Transform, &mut Movement, &mut Location, &Speed), Without<Wall>>,
    level: Res<Level>,
) {
    for (mut transform, mut velocity, mut location, speed) in query.iter_mut() {
        // Ignore non-moving objects
        if velocity.value <= 0.0 {
            velocity.direction = BoardDirection::default();
            continue;
        }
        let frames = FPS * speed.0;
        let value = sizes::field.x / frames;
        let vector = velocity.direction * Vec2::new(1.0, 1.0) * value;
        let new_translation = Vec3::new(
            vector.x + transform.translation.x,
            transform.translation.y,
            vector.y + transform.translation.z,
        );
        transform.translation = new_translation;

        // reduce the velocity based on the frame
        if velocity.value >= 0.0 {
            velocity.value -= value;
        }
        // Otherwise, apply everything
        if velocity.value <= 0.0 {
            location.0.apply_direction(&velocity.direction);
            velocity.direction = BoardDirection::default();
            velocity.value = 0.0;
            transform.translation = level.translate_from_position(location.0);
        }
    }
}

pub fn wall_visibility(
    mut commands: Commands,
    query: Query<(Entity, &Location), With<Wall>>,
    level: Res<Level>,
    player_query: Query<&Location, (With<Player>, Changed<Location>)>,
    materials: Res<MaterialHandles>,
) {
    let player_location = match player_query.iter().next() {
        Some(n) => n,
        None => return,
    };
    let walls_below = level.wall_positions(player_location.0);
    for (entity, location) in query.iter() {
        if walls_below.contains(&location.0) {
            commands
                .entity(entity)
                .remove::<Handle<StandardMaterial>>()
                .insert(materials.wall_hidden.clone());
        } else {
            commands
                .entity(entity)
                .remove::<Handle<StandardMaterial>>()
                .insert(materials.wall_normal.clone());
        }
    }
}

/// Updates the level whenever player or enemy change their location
#[allow(clippy::type_complexity)]
#[allow(clippy::too_many_arguments)]
pub fn update_level(
    mut commands: Commands,
    mut level: ResMut<Level>,
    player_query: Query<(Entity, &Location, &Transform), (With<Player>, Changed<Location>)>,
    enemy_query: Query<(Entity, &Location), (With<Enemy>, Changed<Location>)>,
    mut score: ResMut<Score>,
    mut player_sender: EventWriter<PlayerDiedEvent>,
    audio: Res<Audio>,
    sounds: Res<AudioHandles>,
) {
    for (entity, location) in enemy_query.iter() {
        level.enemy_positions.insert(entity, location.0);
        if level.player_position == location.0 {
            player_sender.send(PlayerDiedEvent);
        }
    }
    if let Some((player_entity, player_location, player_transform)) = player_query.iter().next() {
        level.player_position = player_location.0;
        // check if player and enemies collide
        for position in level.enemy_positions.values() {
            if position == &player_location.0 {
                player_sender.send(PlayerDiedEvent);
            }
        }
        // check if the player is over the exit
        if level.ending_position == player_location.0 && level.ending_visible {
            // somehow jump to the next level
            player_enter_exit(&mut commands, player_entity, player_transform)
        }
        let mut deleted_coins = Vec::new();
        for (entity, position) in level.coin_positions.iter() {
            if position == &player_location.0 {
                destroy_coin(&mut commands, entity);
                score.coins += 1;
                deleted_coins.push(*entity);
                audio.play(sounds.coin.clone());
            }
        }
        for coin in deleted_coins {
            level.coin_positions.remove(&coin);
        }
    }
}

fn destroy_coin(commands: &mut Commands, entity: &Entity) {
    let mut tween = Tween::new(
        EaseFunction::QuadraticInOut,
        TweeningType::Once,
        Duration::from_secs_f32(0.5),
        TransformScaleLens {
            start: Vec3::new(1.0, 1.0, 1.0),
            end: Vec3::ZERO,
        },
    );
    tween.set_completed_event(0);
    commands.entity(*entity).insert(Animator::new(tween));
}

pub fn player_did_die_system(
    mut commands: Commands,
    player: Query<(Entity, &Transform), With<Player>>,
    mut player_reader: EventReader<PlayerDiedEvent>,
    audio: Res<Audio>,
    sounds: Res<AudioHandles>,
) {
    for _ in player_reader.iter() {
        let (entity, transform) = player.single();
        implode_entity(&mut commands, entity, transform, USER_DIED_PAYLOAD);
        commands
            .entity(entity)
            .remove::<Movement>()
            .remove::<Speed>();
        // send a brief delay before going to loose
        audio.play(sounds.kill.clone());
    }
}

/// This removes all tweens that are done and had a complete handler set up
pub fn tween_done_remove_handler(
    mut commands: Commands,
    mut done: EventReader<TweenCompleted>,
    mut writer: EventWriter<GoNextLevelEvent>,
    mut app_state: ResMut<State<GameState>>,
) {
    for ev in done.iter() {
        if ev.user_data == LEVEL_COMPLETED_PAYLOAD {
            writer.send(GoNextLevelEvent);
        } else if ev.user_data == USER_DIED_PAYLOAD {
            app_state.set(GameState::Lost).unwrap();
        } else {
            commands.entity(ev.entity).despawn_recursive();
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub fn bomb_counter(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Bomb, &mut Transform)>,
    time: Res<Time>,
    mut level: ResMut<Level>,
    material_handles: Res<MaterialHandles>,
    mut meshes: ResMut<Assets<Mesh>>,
    audio: Res<Audio>,
    sounds: Res<AudioHandles>,
) {
    let change = time.delta_seconds();
    for (entity, mut bomb, mut transform) in query.iter_mut() {
        bomb.0 -= change;
        if bomb.0 <= 0.0 {
            commands.entity(entity).despawn_recursive();
            // spawn the explosions
            for (level_position, strength, max) in level.bomb_explode_positions(entity) {
                let delay_sec = (strength as f32 / max as f32) / 2.0;
                let position = level.translate_from_position(level_position);
                let id = add_bomb_explosion(
                    &mut commands,
                    &mut meshes,
                    &material_handles,
                    level_position,
                    position,
                );
                insert_bomb_explosion_tween(&mut commands, id, delay_sec);
            }
            if level.bombs.contains_key(&entity) {
                audio.play(sounds.explosion.clone());
            }
            level.bombs.remove(&entity);
        } else if bomb.0 <= 0.5 {
            // the closer to zero we get, the more the bomb shakes
            transform.translation.y = change.sin() * 10.;
        }
    }
}

// if enemy or player interacts with a bomb explosion, remove them
#[allow(clippy::too_many_arguments)]
pub fn bomb_explosion_destruction(
    mut commands: Commands,
    explosion_query: Query<(Entity, &Location), With<BombExplosion>>,
    enemy_query: Query<(Entity, &Transform), With<Enemy>>,
    mut level: ResMut<Level>,
    mut level_exit_writer: EventWriter<ShowLevelExitEvent>,
    mut player_sender: EventWriter<PlayerDiedEvent>,
    audio: Res<Audio>,
    sounds: Res<AudioHandles>,
) {
    let mut removable_enemies = Vec::new();
    for (_, location) in explosion_query.iter() {
        if level.player_position == location.0 {
            player_sender.send(PlayerDiedEvent);
        }
        for (entity, transform) in enemy_query.iter() {
            if level.enemy_positions[&entity] == location.0 {
                implode_entity(&mut commands, entity, transform, 0);
                removable_enemies.push(entity);
                commands
                    .entity(entity)
                    .remove::<Enemy>()
                    .remove::<Movement>()
                    .remove::<Speed>();
            }
        }
    }
    for entity in removable_enemies {
        if level.enemy_positions.contains_key(&entity) {
            audio.play(sounds.kill.clone());
        }
        level.enemy_positions.remove(&entity);
    }
    // if there're no enemies left, start the end level condition
    if level.enemy_positions.is_empty() && !level.ending_visible && level.done_loading {
        level_exit_writer.send(ShowLevelExitEvent);
        level.ending_visible = true;
    }
}

pub fn show_level_exit(
    mut commands: Commands,
    mut event: EventReader<ShowLevelExitEvent>,
    exits: Query<(Entity, &Transform), With<Exit>>,
    coins: Query<Entity, With<Coin>>,
    mut lamps: Query<&mut Visibility, With<ExitLight>>,
    mut level: ResMut<Level>,
) {
    for _ in event.iter() {
        for (entity, transform) in exits.iter() {
            let tween = Tween::new(
                EaseFunction::BounceOut,
                TweeningType::Once,
                Duration::from_secs_f32(2.5),
                TransformPositionLens {
                    start: transform.translation,
                    end: Vec3::new(
                        transform.translation.x,
                        transform.translation.y - 1.1,
                        transform.translation.z,
                    ),
                },
            );
            commands.entity(entity).insert(Animator::new(tween));
            let mut vis_map = lamps.single_mut();
            vis_map.is_visible = true;
        }
        // Destory all remaining coins
        for entity in coins.iter() {
            destroy_coin(&mut commands, &entity);
        }
        level.coin_positions.clear();
    }
}

fn player_enter_exit(commands: &mut Commands, entity: Entity, transform: &Transform) {
    let mut tween = Tween::new(
        EaseFunction::BounceOut,
        TweeningType::Once,
        Duration::from_secs_f32(0.7),
        TransformPositionLens {
            start: transform.translation,
            end: Vec3::new(
                transform.translation.x,
                transform.translation.y - 1.5,
                transform.translation.z,
            ),
        },
    );
    tween.set_completed_event(LEVEL_COMPLETED_PAYLOAD);
    commands
        .entity(entity)
        .remove_bundle::<OutlineBundle>()
        .insert(Animator::new(tween));
}

fn implode_entity(commands: &mut Commands, entity: Entity, transform: &Transform, payload: u64) {
    let duration = 0.3;
    // We scale the enemy
    let tween1 = Tween::new(
        EaseFunction::BounceOut,
        TweeningType::Once,
        Duration::from_secs_f32(duration),
        TransformScaleLens {
            start: Vec3::new(1.0, 1.0, 1.0),
            end: Vec3::new(0.5, 0.5, 0.5),
        },
    );
    // We move it up
    let tween2 = Tween::new(
        EaseFunction::BounceOut,
        TweeningType::Once,
        Duration::from_secs_f32(duration),
        TransformPositionLens {
            start: transform.translation,
            end: Vec3::new(
                transform.translation.x,
                transform.translation.y + 2.1,
                transform.translation.z,
            ),
        },
    );
    // We rotate it
    let tween3 = Tween::new(
        EaseFunction::BounceOut,
        TweeningType::Once,
        Duration::from_secs_f32(duration),
        TransformRotationLens {
            start: transform.rotation,
            end: transform
                .rotation
                .mul_quat(Quat::from_euler(EulerRot::XYZ, 1.3, 1.5, 0.7)),
        },
    );
    let step1 = Tracks::new([tween1, tween2, tween3]);
    // finally, we pop it out
    let mut step2 = Tween::new(
        EaseFunction::ExponentialOut,
        TweeningType::Once,
        Duration::from_secs_f32(0.1),
        TransformScaleLens {
            start: Vec3::new(0.5, 0.5, 0.5),
            end: Vec3::ZERO,
        },
    );
    step2.set_completed_event(payload);
    let series = Sequence::from_single(step1).then(step2);
    commands.entity(entity).insert(Animator::new(series));
}

fn insert_bomb_explosion_tween(commands: &mut Commands, entity: Entity, delay_sec: f32) {
    let step = 0.10;
    // build up the explosion tweens
    let tween1 = Tween::new(
        EaseFunction::BounceInOut,
        TweeningType::Once,
        Duration::from_secs_f32(step),
        TransformScaleLens {
            start: Vec3::ZERO,
            end: Vec3::new(1.0, 1.0, 1.0),
        },
    );
    let tween2 = Tween::new(
        EaseFunction::BounceInOut,
        TweeningType::Once,
        Duration::from_secs_f32(step),
        TransformScaleLens {
            start: Vec3::new(1.0, 1.0, 1.0),
            end: Vec3::new(0.75, 0.75, 0.75),
        },
    );
    let tween3 = Tween::new(
        EaseFunction::BounceInOut,
        TweeningType::Once,
        Duration::from_secs_f32(step),
        TransformScaleLens {
            start: Vec3::new(0.75, 0.75, 0.75),
            end: Vec3::new(0.85, 0.85, 0.85),
        },
    );
    let tween4 = Tween::new(
        EaseFunction::BounceInOut,
        TweeningType::Once,
        Duration::from_secs_f32(step),
        TransformScaleLens {
            start: Vec3::new(0.85, 0.85, 0.85),
            end: Vec3::new(0.5, 0.5, 0.5),
        },
    );
    let mut tween5 = Tween::new(
        EaseFunction::BounceInOut,
        TweeningType::Once,
        Duration::from_secs_f32(step),
        TransformScaleLens {
            start: Vec3::new(0.5, 0.5, 0.5),
            end: Vec3::ZERO,
        },
    );
    tween5.set_completed_event(0);
    let delay = Delay::new(Duration::from_secs_f32(delay_sec));
    let s = delay
        .then(tween1)
        .then(tween2)
        .then(tween3)
        .then(tween4)
        .then(tween5);
    commands.entity(entity).insert(Animator::new(s));
}
