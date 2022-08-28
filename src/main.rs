//! Tasks
//! - [ ] update the level with player and enemy positions
//! - [ ] make a level twice as high
//! - [ ] when a level is done, open a hole, going in there, falling into the next level
//! - [ ] add bomb update item to increase bomb range
//! - [ ] special item that rotates the level z 90. so that the controls are temporary inverse

use bevy::{prelude::*, window::close_on_esc};
use game_plugin::MaterialHandles;

mod game_plugin;

#[derive(Copy, Clone, PartialEq, Eq, Debug, Hash)]
pub enum GameState {
    Loading,
    Menu,
    Game,
    Ending,
}

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::rgb(0.2, 0.2, 0.2)))
        .insert_resource(WindowDescriptor {
            title: "I am a window!".to_string(),
            width: 844.,
            height: 600.,
            // resizable: false,
            ..default()
        })
        .add_state(GameState::Game)
        .add_plugins(DefaultPlugins)
        .add_plugin(game_plugin::GamePlugin)
        .add_startup_system(init)
        .add_system(close_on_esc)
        .run();
}

fn init(
    mut commands: Commands,
    mut assets: ResMut<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
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
