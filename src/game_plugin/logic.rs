use bevy::prelude::*;
use bevy_mod_outline::*;
use bevy_tweening::{
    lens::{TransformPositionLens, TransformRotationLens, TransformScaleLens},
    Animator, Delay, EaseFunction, Sequence, Tracks, Tween, TweenCompleted, TweeningType,
};
use std::{cmp::Ordering, f32::consts::TAU, time::Duration};

use super::level::Level;
use super::statics::{sizes, FPS};
use super::types::*;

pub fn setup(
    mut commands: Commands,
    mut assets: ResMut<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut level: ResMut<Level>,
) {
    let material_handles = {
        let wall_normal = materials.add(Color::rgb(0.8, 0.7, 0.6).into());
        let wall_hidden = materials.add(Color::rgba(0.8, 0.7, 0.6, 0.3).into());
        let coin = materials.add(StandardMaterial {
            base_color: Color::YELLOW,
            emissive: Color::rgb(0.1, 0.1, 0.1),
            ..Default::default()
        });

        let player = materials.add(StandardMaterial {
            base_color: Color::BLUE,
            metallic: 0.5,
            reflectance: 0.15,
            ..Default::default()
        });

        let enemy = materials.add(StandardMaterial {
            base_color: Color::RED,
            ..Default::default()
        });

        let floor_bg = materials.add(StandardMaterial {
            base_color: Color::DARK_GRAY,
            metallic: 0.0,
            reflectance: 0.15,
            ..Default::default()
        });

        let floor_fg = materials.add(StandardMaterial {
            base_color: Color::LIME_GREEN,
            metallic: 0.5,
            reflectance: 0.75,
            ..Default::default()
        });

        let ground = materials.add(StandardMaterial {
            base_color: Color::DARK_GRAY,
            ..Default::default()
        });

        let bomb = materials.add(StandardMaterial {
            base_color: Color::BLACK,
            metallic: 1.0,
            ..Default::default()
        });

        let explosion = materials.add(StandardMaterial {
            base_color: Color::YELLOW,
            emissive: Color::YELLOW,
            ..Default::default()
        });

        MaterialHandles {
            wall_normal,
            wall_hidden,
            coin,
            player,
            enemy,
            floor_bg,
            floor_fg,
            bomb,
            explosion,
            ground,
        }
    };

    let mut enemies = Vec::new();
    let mut coins = Vec::new();

    for row in level.rows() {
        for block in row.iter() {
            // Each entry also needs a floor
            let is_exit = matches!(block.kind, BlockType::Exit);
            setup_space(
                &mut commands,
                &mut meshes,
                &material_handles,
                (block.position.x, block.position.z),
                is_exit,
            );
            match block.kind {
                BlockType::WallBig => {
                    setup_wall(&mut commands, &mut meshes, &material_handles, block)
                }
                BlockType::WallSmallV => {
                    setup_wall(&mut commands, &mut meshes, &material_handles, block)
                }
                BlockType::WallSmallH => {
                    setup_wall(&mut commands, &mut meshes, &material_handles, block)
                }
                BlockType::Coin => {
                    let id = setup_coin(&mut commands, &mut meshes, &material_handles, block);
                    coins.push((id, block.level_position));
                }
                BlockType::Player => {
                    setup_player(&mut commands, &mut meshes, &material_handles, block)
                }
                BlockType::Enemy => {
                    let id = setup_enemy(&mut commands, &mut meshes, &material_handles, block);
                    enemies.push((id, block.level_position));
                }
                BlockType::Space => {}
                BlockType::Exit => {
                    let p = block.position;
                    commands
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
                        .insert(ExitLight);
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

    // light
    commands.spawn_bundle(PointLightBundle {
        point_light: PointLight {
            intensity: 1500.0,
            shadows_enabled: false,
            ..default()
        },
        transform: Transform::from_xyz(0.0, 7.0, 0.5),
        ..default()
    });
    commands.spawn_bundle(SpotLightBundle {
        spot_light: SpotLight {
            intensity: 2500.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(0.0, 7.0, 0.5).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });
    // camera
    commands.spawn_bundle(Camera3dBundle {
        transform: Transform::from_xyz(0.0, 5.5, 3.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });
    commands.insert_resource(material_handles);
}

pub fn setup_wall(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &MaterialHandles,
    block: &Block,
) {
    let s = block.kind.size();
    let p = block.position;
    let wall_mesh = Mesh::from(shape::Box::new(s.x, s.y, s.z));
    commands
        .spawn_bundle(PbrBundle {
            mesh: meshes.add(wall_mesh),
            material: materials.wall_normal.clone(),
            transform: Transform::from_xyz(p.x, p.y, p.z),
            ..default()
        })
        .insert(Size(s))
        .insert(Location(block.level_position))
        .insert(Wall);
}

pub fn setup_coin(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &MaterialHandles,
    block: &Block,
) -> Entity {
    let s = block.kind.size();
    let p = block.position;
    let coin_mesh = Mesh::from(shape::Torus {
        radius: s.x,
        ring_radius: s.x * 0.25,
        subdivisions_segments: 8,
        subdivisions_sides: 6,
    });
    commands
        .spawn_bundle(MaterialMeshBundle {
            mesh: meshes.add(coin_mesh),
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
) {
    let s = block.kind.size();
    let p = block.position;
    let mut player_mesh = Mesh::from(shape::Icosphere {
        radius: s.x,
        subdivisions: 4,
    });
    player_mesh.generate_outline_normals().unwrap();
    commands
        .spawn_bundle(PbrBundle {
            mesh: meshes.add(player_mesh),
            material: materials.player.clone(),
            transform: Transform::from_xyz(p.x, p.y, p.z),
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
        .insert(Speed(0.3))
        .insert(Player);
}

pub fn setup_enemy(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &MaterialHandles,
    block: &Block,
) -> Entity {
    let s = block.kind.size();
    let p = block.position;
    let enemy_mesh = Mesh::from(shape::Cube { size: 0.2 });
    commands
        .spawn_bundle(PbrBundle {
            mesh: meshes.add(enemy_mesh),
            material: materials.enemy.clone(),
            transform: Transform::from_xyz(p.x, p.y, p.z),
            ..default()
        })
        .insert(Size(s))
        .insert(Movement::default())
        .insert(Location(block.level_position))
        .insert(Speed(0.7))
        .insert(Enemy)
        .id()
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
    commands
        .spawn_bundle(PbrBundle {
            mesh: meshes.add(mesh),
            material: materials.bomb.clone(),
            transform: Transform::from_xyz(position.x, position.y, position.z),
            ..default()
        })
        .insert(Location(level_position))
        .insert(Bomb::new())
        .id()
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
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &MaterialHandles,
    position: (f32, f32),
    hides_exit: bool,
) {
    let parent = commands
        .spawn_bundle(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Plane {
                size: sizes::field.x,
            })),
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
            mesh: meshes.add(Mesh::from(shape::Plane {
                size: sizes::space.x,
            })),
            material: materials.floor_bg.clone(),
            transform: Transform::from_xyz(0.0, 0.01, 0.0),
            ..default()
        })
        .id();
    let child2 = commands
        .spawn_bundle(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Cube {
                size: sizes::field.x,
            })),
            material: materials.ground.clone(),
            transform: Transform::from_xyz(0.0, -sizes::field.y, 0.0),
            ..default()
        })
        .id();
    commands.entity(parent).push_children(&[child1, child2]);
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

pub fn enemy_logic(
    mut query: Query<(&mut Movement, &Transform, &Location, &Speed), With<Enemy>>,
    timer: Res<Time>,
    // mut last_step: Local<f32>,
    level: Res<Level>,
    player_query: Query<&Transform, With<Player>>,
) {
    // find the player location
    let player_location = match player_query.iter().next() {
        Some(n) => Vec2::new(n.translation.x, n.translation.z),
        None => return,
    };

    for (mut velocity, transform, position, speed) in query.iter_mut() {
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
pub fn update_level(
    mut commands: Commands,
    mut level: ResMut<Level>,
    player_query: Query<(Entity, &Location, &Transform), (With<Player>, Changed<Location>)>,
    enemy_query: Query<(Entity, &Location), (With<Enemy>, Changed<Location>)>,
    mut score: ResMut<Score>,
) {
    for (entity, location) in enemy_query.iter() {
        level.enemy_positions.insert(entity, location.0);
    }
    if let Some((player_entity, player_location, player_transform)) = player_query.iter().next() {
        level.player_position = player_location.0;
        // check if player and enemies collide
        for position in level.enemy_positions.values() {
            if position == &player_location.0 {
                // FIXME: State trigger
            }
        }
        // check if the player is over the exit
        if level.ending_position == player_location.0 {
            // somehow jump to the next level
            player_enter_exit(&mut commands, player_entity, player_transform)
        }
        let mut deleted_coins = Vec::new();
        for (entity, position) in level.coin_positions.iter() {
            if position == &player_location.0 {
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
                score.coins += 1;
                deleted_coins.push(*entity);
            }
        }
        for coin in deleted_coins {
            level.coin_positions.remove(&coin);
        }
    }
}

/// This removes all tweens that are done and had a complete handler set up
pub fn tween_done_remove_handler(mut commands: Commands, mut done: EventReader<TweenCompleted>) {
    for ev in done.iter() {
        commands.entity(ev.entity).despawn_recursive();
    }
}

pub fn bomb_counter(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Bomb)>,
    time: Res<Time>,
    mut level: ResMut<Level>,
    material_handles: Res<MaterialHandles>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let change = time.delta_seconds();
    for (entity, mut bomb) in query.iter_mut() {
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
            level.bombs.remove(&entity);
        }
    }
}

// if enemy or player interacts with a bomb explosion, remove them
pub fn bomb_explosion_destruction(
    mut commands: Commands,
    explosion_query: Query<(Entity, &Location), With<BombExplosion>>,
    enemy_query: Query<(Entity, &Transform), With<Enemy>>,
    mut level: ResMut<Level>,
    mut level_exit_writer: EventWriter<ShowLevelExitEvent>,
) {
    let mut removable_enemies = Vec::new();
    for (entity, location) in explosion_query.iter() {
        if level.player_position == location.0 {
            //println!("GAME OVER cause bombs");
        }
        for (entity, transform) in enemy_query.iter() {
            if level.enemy_positions[&entity] == location.0 {
                kill_enemy(&mut commands, entity, &transform);
                removable_enemies.push(entity);
            }
        }
    }
    for entity in removable_enemies {
        level.enemy_positions.remove(&entity);
    }
    // if there're no enemies left, start the end level condition
    if level.enemy_positions.is_empty() && !level.ending_visible {
        level_exit_writer.send(ShowLevelExitEvent);
        level.ending_visible = true;
    }
}

pub fn show_level_exit(
    mut commands: Commands,
    mut event: EventReader<ShowLevelExitEvent>,
    exits: Query<(Entity, &Transform), With<Exit>>,
    mut lamps: Query<&mut Visibility, With<ExitLight>>,
) {
    for ev in event.iter() {
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
    }
}

fn player_enter_exit(commands: &mut Commands, entity: Entity, transform: &Transform) {
    let tween = Tween::new(
        EaseFunction::BounceOut,
        TweeningType::Once,
        Duration::from_secs_f32(0.5),
        TransformPositionLens {
            start: transform.translation,
            end: Vec3::new(
                transform.translation.x,
                transform.translation.y - 1.5,
                transform.translation.z,
            ),
        },
    );
    commands
        .entity(entity)
        .remove_bundle::<OutlineBundle>()
        .insert(Animator::new(tween));
}

fn kill_enemy(commands: &mut Commands, entity: Entity, transform: &Transform) {
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
    step2.set_completed_event(0);
    let series = Sequence::from_single(step1).then(step2);
    commands
        .entity(entity)
        .remove::<Enemy>()
        .remove::<Movement>()
        .remove::<Speed>()
        .insert(Animator::new(series));
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