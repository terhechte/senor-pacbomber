use bevy::{
    pbr::{MaterialPipeline, MaterialPipelineKey},
    prelude::*,
    reflect::TypeUuid,
    render::{
        mesh::MeshVertexBufferLayout,
        render_resource::{
            AsBindGroup, RenderPipelineDescriptor, ShaderRef, SpecializedMeshPipelineError,
        },
    },
    sprite::collide_aabb::collide,
    time::FixedTimestep,
    window::close_on_esc,
};
use bevy_mod_outline::*;
use std::{
    cmp::Ordering,
    collections::HashSet,
    f32::consts::{PI, TAU},
    ops::Mul,
    time::Duration,
};

const LEVEL_DATA: &str = r#"
#----------   ----------#
| * * * * *o** * * * *  |
| ##---- #-----# ----## |
| #* *   #x    #   * *# |
  #----  # ### #  ----#  
| * * * * *   * * * * * |
| --#--- ##   ## ---#-- |
|  *|*             *|*  |
#----------   ----------#
"#;

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
enum BlockType {
    WallBig,
    WallSmallV,
    WallSmallH,
    Coin,
    Enemy,
    Player,
    Space,
}

impl BlockType {
    fn size(&self) -> Vec3 {
        let v_b = sizes::brick;
        let v_s = sizes::brick_small;
        match self {
            BlockType::WallBig => v_b,
            BlockType::WallSmallV => Vec3::new(v_s, v_b.y, v_b.z),
            BlockType::WallSmallH => Vec3::new(v_b.x, v_b.y, v_s),
            BlockType::Coin => sizes::coin,
            BlockType::Enemy => sizes::enemy,
            BlockType::Player => sizes::enemy,
            BlockType::Space => sizes::space,
        }
    }

    fn is_wall(&self) -> bool {
        matches!(
            self,
            BlockType::WallBig | BlockType::WallSmallH | BlockType::WallSmallV
        )
    }
}

impl From<char> for BlockType {
    fn from(c: char) -> Self {
        use BlockType::*;
        match c {
            '*' => Coin,
            '#' => WallBig,
            '-' => WallSmallH,
            '|' => WallSmallV,
            'o' => Player,
            'x' => Enemy,
            ' ' => Space,
            _ => panic!("Unknown Level char {c}"),
        }
    }
}

pub struct MaterialHandles {
    pub wall_normal: Handle<StandardMaterial>,
    pub wall_hidden: Handle<StandardMaterial>,
    pub coin: Handle<StandardMaterial>,
    pub player: Handle<StandardMaterial>,
    pub enemy: Handle<StandardMaterial>,
    pub floor_bg: Handle<StandardMaterial>,
    pub floor_fg: Handle<StandardMaterial>,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Default, Hash)]
struct Position {
    x: usize,
    z: usize,
}

impl Position {
    fn new(x: usize, z: usize) -> Self {
        Self { x, z }
    }

    fn apply_direction(&mut self, direction: &Direction) {
        let x = self.x as i8 + direction.x;
        if x >= 0 {
            self.x = x as usize;
        }
        let z = self.z as i8 + direction.z;
        if z >= 0 {
            self.z = z as usize;
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Default)]
struct Direction {
    x: i8,
    z: i8,
}

impl Direction {
    fn new(x: i8, z: i8) -> Self {
        Self { x, z }
    }
}

impl Mul<Vec2> for Direction {
    type Output = Vec2;
    #[inline]
    fn mul(self, rhs: Vec2) -> Vec2 {
        Vec2 {
            x: (self.x as f32).mul(rhs.x),
            y: (self.z as f32).mul(rhs.y),
        }
    }
}

#[derive(Debug, Copy, Clone)]
struct Block {
    kind: BlockType,
    position: Vec3,
    level_position: Position,
}

#[derive(Debug)]
struct Level {
    size: Position,
    offsets: (f32, f32),
    rows: Vec<Vec<Block>>,
}

impl Level {
    fn new(data: &str) -> Self {
        let mut rows: Vec<Vec<_>> = Vec::new();

        let lines: Vec<&str> = data.split('\n').filter(|e| !e.is_empty()).collect();
        let z_offset = (sizes::field.z * (lines.len() as f32)) / 2.0;
        let mut x_offset: f32 = 0.0;
        let v_b = sizes::field;

        let z_size = lines.len();
        let mut x_size = 0;

        for (x_index, line) in lines.iter().enumerate() {
            let chars: Vec<char> = line.chars().collect();
            x_size = chars.len();
            x_offset = (sizes::field.x * (chars.len() as f32)) / 2.0;
            let mut row = Vec::new();
            for (z_index, block) in chars.into_iter().map(BlockType::from).enumerate() {
                let position = (
                    ((x_index as f32 * v_b.x) - z_offset) + v_b.x / 2.0,
                    ((z_index as f32 * v_b.z) - x_offset) + v_b.z / 2.0,
                );

                row.push(Block {
                    kind: block,
                    position: Vec3::new(position.1, 0.0, position.0),
                    level_position: Position::new(z_index, x_index),
                })
            }

            rows.push(row);
        }
        Level {
            size: Position::new(x_size, z_size),
            offsets: (x_offset, z_offset),
            rows,
        }
    }

    fn rows(&self) -> impl Iterator<Item = &Vec<Block>> {
        self.rows.iter()
    }

    fn get(&self, ax: i8, az: i8) -> Option<Block> {
        if ax < 0 || az < 0 {
            return None;
        }
        if ax as usize >= self.size.x || az as usize >= self.size.z {
            return None;
        }
        let item = self.rows[az as usize][ax as usize];
        Some(item)
    }

    fn translate_to_position(&self, position: Vec3) -> Position {
        let offsets = Vec3::new(self.offsets.0, 0.0, self.offsets.1);
        let size = offsets * 2.;
        let item = Vec3::new(self.size.x as f32, 0., self.size.z as f32) / size;
        Position {
            x: ((position.x + offsets.x) * item.x) as usize,
            z: ((position.z + offsets.z) * item.z) as usize,
        }
    }

    fn translate_from_position(&self, position: Position) -> Vec3 {
        let (x_offset, z_offset) = self.offsets;
        let (x_index, z_index) = (position.x, position.z);
        let v_b = sizes::field;
        let position = (
            ((x_index as f32 * v_b.x) - x_offset) + v_b.x / 2.0,
            ((z_index as f32 * v_b.z) - z_offset) + v_b.z / 2.0,
        );
        Vec3::new(position.0, 0.0, position.1)
    }

    /// Find all free spaces (e.g. not walls) around a position
    fn free_directions(&self, position: Position) -> Vec<Direction> {
        // traverse all directions around the position and check if they're free
        let (x, z) = (position.x as i8, position.z as i8);
        let mut results = Vec::new();
        'outer: for (mx, mz) in [(1_i8, 0), (-1_i8, 0), (0, 1), (0, -1_i8)] {
            let item = match self.get(x + mx, z + mz) {
                Some(n) => n,
                None => continue,
            };
            if item.kind.is_wall() {
                continue 'outer;
            }
            // otherwise this is free
            results.push(Direction::new(mx, mz))
        }
        results
    }

    /// All connected wall positions that are z below +1 from the current position
    fn wall_positions(&self, position: Position) -> Vec<Position> {
        let mut new_position = position;
        new_position.apply_direction(&Direction::new(0, 1));
        let block = match self.get(new_position.x as i8, new_position.z as i8) {
            Some(n) => n,
            None => return Vec::new(),
        };
        if !block.kind.is_wall() {
            return Vec::new();
        }
        // depth search to find all other connected wall elements
        let mut results = vec![new_position];
        let mut tested = HashSet::new();
        tested.insert(new_position);
        fn recursive_search(
            level: &Level,
            position: Position,
            into: &mut Vec<Position>,
            tested: &mut HashSet<Position>,
        ) {
            for (mx, mz) in [(1_i8, 0), (-1_i8, 0), (0, 1), (0, -1_i8)] {
                let mut new = position;
                new.apply_direction(&Direction::new(mx, mz));
                if tested.contains(&new) {
                    continue;
                }
                tested.insert(new);
                let block = match level.get(new.x as i8, new.z as i8) {
                    Some(n) => n,
                    None => {
                        continue;
                    }
                };
                // we ignore the | walls as connectors
                if block.kind.is_wall() && !matches!(block.kind, BlockType::WallSmallV) {
                    into.push(new);
                    recursive_search(level, new, into, tested);
                }
            }
        }
        recursive_search(self, new_position, &mut results, &mut tested);
        results
    }
}

mod sizes {
    #![allow(non_upper_case_globals)]
    use bevy::prelude::*;
    pub const field: Vec3 = Vec3::new(0.25, 0.25, 0.25);
    pub const space: Vec3 = Vec3::new(0.24, -0.1, 0.24);
    pub const brick: Vec3 = Vec3::new(0.25, 0.25, 0.25);
    pub const brick_small: f32 = 0.1;
    pub const coin: Vec3 = Vec3::new(0.10, 0.05, 0.1);
    pub const enemy: Vec3 = Vec3::new(0.10, 0.05, 0.1);
}

#[derive(Component)]
struct Wall;

#[derive(Component)]
struct Enemy;

#[derive(Component)]
struct Coin;

#[derive(Component)]
struct Player;

#[derive(Component, Debug)]
struct Location(Position);

#[derive(Component)]
struct Floor;

#[derive(Component, Default, Debug)]
struct Movement {
    value: f32,
    direction: Direction,
}

#[derive(Component)]
struct Wobbles(f32);

#[derive(Component, Default)]
struct Size(Vec3);

#[derive(Component, Default)]
struct Speed(f32);

const FPS: f32 = 60.0;
const TIME_STEP: f32 = 1.0 / FPS;

fn main() {
    App::new()
        .insert_resource(WindowDescriptor {
            title: "I am a window!".to_string(),
            width: 844.,
            height: 600.,
            // resizable: false,
            ..default()
        })
        .insert_resource(Level::new(LEVEL_DATA))
        .add_plugins(DefaultPlugins)
        .add_plugin(OutlinePlugin)
        .add_startup_system(setup)
        .add_system(close_on_esc)
        .add_system(wobble)
        .add_system(keyboard_input_system)
        .add_system(wall_visibility)
        .add_system_set(
            SystemSet::new()
                .with_run_criteria(FixedTimestep::step(TIME_STEP as f64))
                .with_system(enemy_logic)
                .with_system(move_entities),
        )
        .run();
}

fn setup(
    mut commands: Commands,
    mut assets: ResMut<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    level: Res<Level>,
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
            base_color: Color::WHITE,
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

        MaterialHandles {
            wall_normal,
            wall_hidden,
            coin,
            player,
            enemy,
            floor_bg,
            floor_fg,
        }
    };

    for row in level.rows() {
        for block in row.iter() {
            // Each entry also needs a floor
            setup_space(
                &mut commands,
                &mut meshes,
                &material_handles,
                (block.position.x, block.position.z),
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
                BlockType::Coin => setup_coin(&mut commands, &mut meshes, &material_handles, block),
                BlockType::Player => {
                    setup_player(&mut commands, &mut meshes, &material_handles, block)
                }
                BlockType::Enemy => {
                    setup_enemy(&mut commands, &mut meshes, &material_handles, block)
                }
                BlockType::Space => {}
            }
        }
    }

    // light
    commands.spawn_bundle(PointLightBundle {
        point_light: PointLight {
            intensity: 1500.0,
            shadows_enabled: false,
            ..default()
        },
        transform: Transform::from_xyz(0.0, 8.0, 0.0),
        ..default()
    });
    // camera
    commands.spawn_bundle(Camera3dBundle {
        transform: Transform::from_xyz(0.0, 5.5, 3.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });
    commands.insert_resource(material_handles);
}

fn setup_wall(
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

fn setup_coin(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &MaterialHandles,
    block: &Block,
) {
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
        .insert(Coin);
}

fn setup_player(
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

fn setup_enemy(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &MaterialHandles,
    block: &Block,
) {
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
        .insert(Enemy);
}

fn setup_space(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &MaterialHandles,
    position: (f32, f32),
) {
    commands
        .spawn_bundle(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Plane {
                size: sizes::field.x,
            })),
            material: materials.floor_fg.clone(),
            transform: Transform::from_xyz(position.0, sizes::space.y - 0.01, position.1),
            ..default()
        })
        .insert(Floor);

    commands
        .spawn_bundle(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Plane {
                size: sizes::space.x,
            })),
            material: materials.floor_bg.clone(),
            transform: Transform::from_xyz(position.0, sizes::space.y, position.1),
            ..default()
        })
        .insert(Floor);
}

fn wobble(mut query: Query<(&mut Transform, &Wobbles)>, timer: Res<Time>, mut t: Local<f32>) {
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

fn enemy_logic(
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

fn keyboard_input_system(
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<(&mut Movement, &Location), With<Player>>,
    level: Res<Level>,
) {
    for (mut velocity, location) in query.iter_mut() {
        // if we're in movement, do nothing
        if velocity.value > 0.0 {
            continue;
        }
        // make sure we only move into directions we can
        for (code, direction) in [
            (KeyCode::Left, Direction::new(-1, 0)),
            (KeyCode::Right, Direction::new(1, 0)),
            (KeyCode::Up, Direction::new(0, -1)),
            (KeyCode::Down, Direction::new(0, 1)),
        ] {
            if keyboard_input.pressed(code) {
                let directions = level.free_directions(location.0);
                if directions.contains(&direction) {
                    velocity.direction = direction;
                    velocity.value = sizes::field.x;
                }
            }
        }
    }
}

fn move_entities(
    // We need the entities that are being moved
    mut query: Query<(&mut Transform, &mut Movement, &mut Location, &Speed), Without<Wall>>,
    level: Res<Level>,
) {
    for (mut transform, mut velocity, mut location, speed) in query.iter_mut() {
        // Ignore non-moving objects
        if velocity.value <= 0.0 {
            velocity.direction = Direction::default();
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
            velocity.direction = Direction::default();
            velocity.value = 0.0;
            transform.translation = level.translate_from_position(location.0);
        }
    }
}

fn wall_visibility(
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wall_positions() {
        let level_data = r#"
          x
###########
##        #
#         x
*         x
-----******
"#;
        let level = Level::new(level_data);
        let pos = level.wall_positions(Position::new(0, 0));
        assert_eq!(pos.len(), 15);
    }
}
