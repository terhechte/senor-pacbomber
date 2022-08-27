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
    window::close_on_esc,
};
use bevy_mod_outline::*;
use std::f32::consts::{PI, TAU};

const LEVEL_DATA: &str = r#"
#----------   ----------#
| * * * * * ** * * * *  |
| ##---- #-----# ----## |
| #* *   #x   x#   * *# |
  #----  # ### #  ----#  
| * * * * *   * * * * * |
| --#--- ##   ## ---#-- |
|  *|* o           *|*  |
#----------   ----------#
"#;

#[derive(Debug)]
enum Block {
    WallBig,
    WallSmallV,
    WallSmallH,
    Coin,
    Enemy,
    Player,
    Space,
}

impl From<char> for Block {
    fn from(c: char) -> Self {
        use Block::*;
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
struct Level {
    size: (usize, usize),
    rows: Vec<Vec<Block>>,
}

impl Level {
    fn new(data: &str) -> Self {
        let mut rows: Vec<Vec<_>> = Vec::new();
        for line in data.split('\n').filter(|e| !e.is_empty()) {
            rows.push(line.chars().map(Block::from).collect());
        }
        Level {
            size: (rows.len(), rows[0].len()),
            rows,
        }
    }
}

mod sizes {
    #![allow(non_upper_case_globals)]
    use bevy::prelude::*;
    pub const field: Vec3 = Vec3::new(0.25, 0.25, 0.25);
    pub const brick: Vec3 = Vec3::new(0.25, 0.25, 0.25);
    pub const brick_small: f32 = 0.1;
    pub const coin: Vec3 = Vec3::new(0.10, 0.05, 0.1);
}

#[derive(Component)]
struct Wall;

#[derive(Component)]
struct Enemy;

#[derive(Component)]
struct Coin;

#[derive(Component)]
struct Player;

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
        .run();
}

fn setup(
    mut commands: Commands,
    mut assets: ResMut<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut custom_materials: ResMut<Assets<CustomMaterial>>,
    level: Res<Level>,
) {
    let wall_material = materials.add(Color::rgba(0.8, 0.7, 0.6, 0.0).into());
    // let coin_material = materials.add(Color::rgb(1.0, 1.0, 0.0).into());

    let coin_material = custom_materials.add(CustomMaterial { color: Color::BLUE });

    let player_material = materials.add(StandardMaterial {
        base_color: Color::BLUE,
        metallic: 0.5,
        reflectance: 0.15,
        ..Default::default()
    });

    let x_offset = (sizes::field.x * (level.size.1 as f32)) / 2.0;
    let z_offset = (sizes::field.z * (level.size.0 as f32)) / 2.0;

    // plane
    commands.spawn_bundle(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Box::new(
            x_offset * 2.0,
            0.1,
            z_offset * 2.0,
        ))),
        material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
        transform: Transform::from_xyz(0.0, -0.15, 0.0),
        ..default()
    });

    for (z_index, row) in level.rows.iter().enumerate() {
        for (x_index, block) in row.iter().enumerate() {
            let v_b = sizes::brick;
            let v_s = sizes::brick_small;
            let position = (
                ((x_index as f32 * v_b.x) - x_offset) + v_b.x / 2.0,
                ((z_index as f32 * v_b.z) - z_offset) + v_b.z / 2.0,
            );
            match block {
                Block::WallBig => setup_wall(
                    &mut commands,
                    &mut meshes,
                    wall_material.clone(),
                    position,
                    (v_b.x, v_b.z),
                ),
                Block::WallSmallV => setup_wall(
                    &mut commands,
                    &mut meshes,
                    wall_material.clone(),
                    position,
                    (v_s, v_b.z),
                ),
                Block::WallSmallH => setup_wall(
                    &mut commands,
                    &mut meshes,
                    wall_material.clone(),
                    position,
                    (v_b.x, v_s),
                ),
                Block::Coin => {
                    setup_coin(&mut commands, &mut meshes, coin_material.clone(), position)
                }
                Block::Player => setup_player(
                    &mut commands,
                    &mut meshes,
                    player_material.clone(),
                    position,
                ),
                _ => (),
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
    position: (f32, f32),
    size: (f32, f32),
) {
    let mut wall_mesh = Mesh::from(shape::Box::new(size.0, sizes::brick.y, size.1));
    wall_mesh.generate_outline_normals().unwrap();
    commands
        .spawn_bundle(PbrBundle {
            mesh: meshes.add(wall_mesh),
            material,
            transform: Transform::from_xyz(position.0, 0.0, position.1),
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
        .insert(Wobbles)
        .insert(Wall);
}

fn setup_coin(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    material: Handle<CustomMaterial>,
    position: (f32, f32),
) {
    commands
        .spawn_bundle(MaterialMeshBundle {
            mesh: meshes.add(Mesh::from(shape::Icosphere {
                radius: sizes::coin.x,
                subdivisions: 2,
            })),
            material,
            transform: Transform::from_xyz(position.0, sizes::coin.y, position.1),
            ..default()
        })
        .insert(Coin);
}

fn setup_player(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    material: Handle<StandardMaterial>,
    position: (f32, f32),
) {
    let mut player_mesh = Mesh::from(shape::Icosphere {
        radius: sizes::coin.x,
        subdivisions: 4,
    });
    player_mesh.generate_outline_normals().unwrap();
    commands
        .spawn_bundle(PbrBundle {
            mesh: meshes.add(player_mesh),
            material,
            transform: Transform::from_xyz(position.0, sizes::coin.y, position.1),
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
        .insert(Player);
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
struct Wobbles;

#[derive(Component)]
struct Orbits;

fn wobble(mut query: Query<&mut Transform, With<Wobbles>>, timer: Res<Time>, mut t: Local<f32>) {
    let ta = *t;
    *t = (ta + 0.5 * timer.delta_seconds()) % TAU;
    let tb = *t;
    let i1 = tb.cos() - ta.cos();
    let i2 = ta.sin() - tb.sin();
    for mut transform in query.iter_mut() {
        transform.rotate(Quat::from_rotation_z(
            TAU * 20.0 * i1 * timer.delta_seconds(),
        ));
        transform.rotate(Quat::from_rotation_y(
            TAU * 20.0 * i2 * timer.delta_seconds(),
        ));
    }
}

fn orbit(mut query: Query<&mut Transform, With<Orbits>>, timer: Res<Time>) {
    for mut transform in query.iter_mut() {
        transform.translate_around(
            Vec3::ZERO,
            Quat::from_rotation_y(0.4 * timer.delta_seconds()),
        )
    }
}
