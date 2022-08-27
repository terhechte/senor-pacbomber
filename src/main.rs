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
    f32::consts::{PI, TAU},
};

const LEVEL_DATA: &str = r#"
#----------   ----------#
| * * * * * ** * * * *  |
| ##---- #-----# ----## |
| #* *   #x    #   * *# |
  #----  # ### #  ----#  
| * * * * *   * * * * * |
| --#--- ##   ## ---#-- |
|  *|* o           *|*  |
#----------   ----------#
"#;

#[derive(Debug, Eq, PartialEq)]
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

#[derive(Debug)]
struct Block {
    kind: BlockType,
    position: Vec3,
    level_pos: (usize, usize),
}

#[derive(Debug)]
struct Level {
    size: (usize, usize),
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
                    level_pos: (x_index, z_index),
                })
            }

            rows.push(row);
        }
        Level {
            size: (x_size, z_size),
            offsets: (x_offset, z_offset),
            rows,
        }
    }

    fn rows(&self) -> impl Iterator<Item = &Vec<Block>> {
        self.rows.iter()
    }

    /// Find all block collisions in block space.
    fn free_directions(&self, position: Vec3) -> Vec<Vec2> {
        let blocking_kinds = [
            BlockType::WallBig,
            BlockType::WallSmallH,
            BlockType::WallSmallV,
        ];
        // dbg!(position, self.offsets, self.size);

        // let position = Vec3::new(-0.5, 0.0, -0.25);
        // let offsets = Vec3::new(3.125, 0.0, 1.125);
        let offsets = Vec3::new(self.offsets.0, 0.0, self.offsets.1);
        let size = offsets * 2.;
        let item = Vec3::new(self.size.0 as f32, 0., self.size.1 as f32) / size;
        let pos = Vec3::new(
            (position.x + offsets.x) * item.x,
            0.0,
            (position.z + offsets.z) * item.z,
        );

        // first, convert the position into board pixels
        let x = pos.x as i8;
        let z = pos.z as i8;
        // if x > self.size.0 { return Vec::new() }
        // if z > self.size.1 { return Vec::new() }
        // traverse all directions around the position and check if they're free
        println!("enemy is at {x} {z}");
        let mut results = Vec::new();
        'outer: for (mx, mz) in [
            (1_i8, 0),
            (-1_i8, 0),
            (0, 1),
            (0, -1_i8),
            // (x + 1, z + 1),
            // (x - 1, z - 1),
            // (x + 1, z - 1),
            // (x - 1, z + 1),
        ] {
            let (ax, az) = (x + mx, z + mz);
            if ax < 0 || az < 0 {
                continue;
            }
            let item = &self.rows[az as usize][ax as usize];
            for blocking in &blocking_kinds {
                if blocking == &item.kind {
                    continue 'outer;
                }
            }
            // otherwise this is free
            results.push(Vec2::new(mx as f32, mz as f32))
        }
        dbg!(results)
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

#[derive(Component)]
struct Floor;

#[derive(Component, Default)]
struct Velocity(Vec2);

#[derive(Component, Default)]
struct Size(Vec3);

const TIME_STEP: f32 = 1.0 / 60.0;

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
        .add_plugin(MaterialPlugin::<CustomMaterial>::default())
        .add_startup_system(setup)
        .add_system(close_on_esc)
        .add_system(wobble)
        .add_system(keyboard_input_system)
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
    let wall_material = materials.add(Color::rgb(0.8, 0.7, 0.6).into());
    let coin_material = materials.add(StandardMaterial {
        base_color: Color::YELLOW,
        emissive: Color::rgb(0.1, 0.1, 0.1),
        ..Default::default()
    });

    let player_material = materials.add(StandardMaterial {
        base_color: Color::BLUE,
        metallic: 0.5,
        reflectance: 0.15,
        ..Default::default()
    });

    let enemy_material = materials.add(StandardMaterial {
        base_color: Color::WHITE,
        ..Default::default()
    });

    let floor_bg_material = materials.add(StandardMaterial {
        base_color: Color::DARK_GRAY,
        metallic: 0.0,
        reflectance: 0.15,
        ..Default::default()
    });

    let floor_fg_material = materials.add(StandardMaterial {
        base_color: Color::LIME_GREEN,
        metallic: 0.5,
        reflectance: 0.75,
        ..Default::default()
    });

    for row in level.rows() {
        for block in row.iter() {
            // Each entry also needs a floor
            setup_space(
                &mut commands,
                &mut meshes,
                floor_fg_material.clone(),
                floor_bg_material.clone(),
                (block.position.x, block.position.z),
            );
            match block.kind {
                BlockType::WallBig => {
                    setup_wall(&mut commands, &mut meshes, wall_material.clone(), block)
                }
                BlockType::WallSmallV => {
                    setup_wall(&mut commands, &mut meshes, wall_material.clone(), block)
                }
                BlockType::WallSmallH => {
                    setup_wall(&mut commands, &mut meshes, wall_material.clone(), block)
                }
                BlockType::Coin => {
                    setup_coin(&mut commands, &mut meshes, coin_material.clone(), block)
                }
                BlockType::Player => {
                    setup_player(&mut commands, &mut meshes, player_material.clone(), block)
                }
                BlockType::Enemy => {
                    setup_enemy(&mut commands, &mut meshes, enemy_material.clone(), block)
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
        //projection: Projection::Orthographic(OrthographicProjection::default()),
        //transform: Transform::from_xyz(0.0, 0.5, 4.0).looking_at(Vec3::ZERO, Vec3::Y),
        // working
        transform: Transform::from_xyz(0.0, 5.5, 3.0).looking_at(Vec3::ZERO, Vec3::Y),
        // transform: Transform::from_xyz(0.0, 5.5, 7.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });
}

fn setup_wall(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    material: Handle<StandardMaterial>,
    block: &Block,
) {
    let s = block.kind.size();
    let p = block.position;
    let wall_mesh = Mesh::from(shape::Box::new(s.x, s.y, s.z));
    commands
        .spawn_bundle(PbrBundle {
            mesh: meshes.add(wall_mesh),
            material,
            transform: Transform::from_xyz(p.x, p.y, p.z),
            ..default()
        })
        .insert(Size(s))
        .insert(Wall);
}

fn setup_coin(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    material: Handle<StandardMaterial>,
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
            material,
            transform: Transform::from_xyz(p.x, p.y, p.z),
            ..default()
        })
        .insert(Wobbles(p.x * p.z))
        .insert(Coin);
}

fn setup_player(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    material: Handle<StandardMaterial>,
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
            material,
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
        .insert(Velocity::default())
        .insert(Player);
}

fn setup_enemy(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    material: Handle<StandardMaterial>,
    block: &Block,
) {
    let s = block.kind.size();
    let p = block.position;
    let enemy_mesh = Mesh::from(shape::Cube { size: 0.2 });
    commands
        .spawn_bundle(PbrBundle {
            mesh: meshes.add(enemy_mesh),
            material,
            transform: Transform::from_xyz(p.x, p.y, p.z),
            ..default()
        })
        .insert(Size(s))
        .insert(Velocity::default())
        .insert(Enemy);
}

fn setup_space(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    floor_fg: Handle<StandardMaterial>,
    floor_bg: Handle<StandardMaterial>,
    position: (f32, f32),
) {
    commands
        .spawn_bundle(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Plane {
                size: sizes::field.x,
            })),
            material: floor_fg,
            transform: Transform::from_xyz(position.0, sizes::space.y - 0.01, position.1),
            ..default()
        })
        .insert(Floor);

    commands
        .spawn_bundle(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Plane {
                size: sizes::space.x,
            })),
            material: floor_bg,
            transform: Transform::from_xyz(position.0, sizes::space.y, position.1),
            ..default()
        })
        .insert(Floor);
}

// Materials

// This is the struct that will be passed to your shader
#[derive(AsBindGroup, Clone, TypeUuid)]
#[uuid = "4ee9c363-1124-4113-890e-199d81b00281"]
pub struct CustomMaterial {
    #[uniform(0)]
    color: Color,
}

/// The Material trait is very configurable, but comes with sensible defaults for all methods.
/// You only need to implement functions for features that need non-default behavior. See the Material api docs for details!
/// When using the GLSL shading language for your shader, the specialize method must be overriden.
impl Material for CustomMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/line_material.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Opaque
    }
}

#[derive(Component)]
struct Wobbles(f32);

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
    mut query: Query<(&mut Velocity, &Transform), With<Enemy>>,
    timer: Res<Time>,
    mut last_step: Local<f32>,
    level: Res<Level>,
    player_query: Query<&Transform, With<Player>>,
) {
    let current = timer.time_since_startup().as_secs_f32();
    let last = *last_step;
    let diff = current - last;
    if diff < 0.5 {
        return;
    }

    // find the player location
    let player_location = match player_query.iter().next() {
        Some(n) => Vec2::new(n.translation.x, n.translation.z),
        None => return,
    };

    *last_step = current;
    for (mut velocity, transform) in query.iter_mut() {
        let v = Vec2::new(transform.translation.x, transform.translation.z);
        // find the free directions
        let mut directions = level.free_directions(transform.translation);
        if directions.is_empty() {
            return;
        }
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

        dbg!(directions[0]);

        velocity.0.x = directions[0].x * mov.x;
        velocity.0.y = directions[0].y * mov.y;
    }
}

fn keyboard_input_system(
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<&mut Velocity, With<Player>>,
) {
    let speed = 0.05;
    for mut velocity in query.iter_mut() {
        for (code, vector) in [
            (KeyCode::Left, Vec2::new(-speed, 0.0)),
            (KeyCode::Right, Vec2::new(speed, 0.0)),
            (KeyCode::Up, Vec2::new(0.0, -speed)),
            (KeyCode::Down, Vec2::new(0.0, speed)),
        ] {
            if keyboard_input.pressed(code) {
                velocity.0 = vector;
            } else if keyboard_input.just_released(code) {
                velocity.0 = Vec2::default();
            }
        }
    }
}

fn move_entities(
    // We need the entities that are being moved
    mut query: Query<(&mut Transform, &Size, &Velocity), Without<Wall>>,
    // And the walls
    wall_query: Query<(&Transform, &Size), With<Wall>>,
) {
    for (mut transform, size, velocity) in query.iter_mut() {
        let new_translation = Vec3::new(
            velocity.0.x + transform.translation.x,
            transform.translation.y,
            velocity.0.y + transform.translation.z,
        );
        // Check if we collide with a wall
        for (wall_transform, wall_size) in wall_query.iter() {
            // we perform a 2d collision
            let c = collide(
                Vec3::new(
                    wall_transform.translation.x,
                    wall_transform.translation.z,
                    0.0,
                ),
                Vec2::new(wall_size.0.x, wall_size.0.z),
                Vec3::new(new_translation.x, new_translation.z, 0.0),
                Vec2::new(size.0.x, size.0.z),
            );
            if c.is_some() {
                return;
            }
        }
        transform.translation = new_translation;
    }
}
