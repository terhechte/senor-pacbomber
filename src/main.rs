use bevy::{audio::AudioSink, prelude::*};
use bevy_mod_outline::*;
use bevy_tweening::TweeningPlugin;

mod game_plugin;
mod loading_plugin;
mod lost_plugin;
mod menu_plugin;
mod types;
mod won_plugin;

use game_plugin::BlockType;
use types::CurrentMusic;
pub use types::{MaterialHandles, MeshHandles};

use crate::types::AudioHandles;

#[derive(Copy, Clone, PartialEq, Eq, Debug, Hash)]
pub enum GameState {
    Menu,
    Loading,
    Game,
    Running,
    Lost,
    Won,
}

fn main() {
    App::new()
        .insert_resource(game_plugin::Score::default())
        .insert_resource(ClearColor(Color::rgb(20. / 255., 20. / 255., 20. / 255.)))
        .insert_resource(WindowDescriptor {
            title: "PACBOMBER".to_string(),
            width: 900.,
            height: 660.,
            resizable: false,
            ..default()
        })
        .add_state(GameState::Menu)
        .add_plugins(DefaultPlugins)
        .add_plugin(OutlinePlugin)
        .add_plugin(TweeningPlugin)
        .add_plugin(game_plugin::GamePlugin)
        .add_plugin(menu_plugin::MenuPlugin)
        .add_plugin(won_plugin::WonPlugin)
        .add_plugin(lost_plugin::LostPlugin)
        .add_plugin(loading_plugin::LoadingPlugin)
        .add_startup_system(cache_assets)
        .run();
}

fn cache_assets(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    audio: Res<Audio>,
    audio_sinks: Res<Assets<AudioSink>>,
) {
    // Audio
    let audio_handles = {
        let intro = asset_server.load("sounds/intro.ogg");
        let music = asset_server.load("sounds/music.ogg");
        let coin = asset_server.load("sounds/coin.ogg");
        let kill = asset_server.load("sounds/kill.ogg");
        let explosion = asset_server.load("sounds/explosion.ogg");
        AudioHandles {
            intro,
            music,
            kill,
            coin,
            explosion,
        }
    };
    let weak_handle = audio.play_with_settings(
        audio_handles.intro.clone(),
        PlaybackSettings::LOOP.with_volume(0.5),
    );
    let strong_handle = audio_sinks.get_handle(weak_handle);
    commands.insert_resource(CurrentMusic(strong_handle));
    commands.insert_resource(audio_handles);

    // Materials

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
            base_color: Color::GRAY,
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

        let white = materials.add(StandardMaterial {
            base_color: Color::WHITE,
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
            white,
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

    // Meshes

    let meshes = {
        let s = BlockType::WallBig.size();
        let wall = Mesh::from(shape::Box::new(s.x, s.y, s.z));

        let s = BlockType::WallSmallV.size();
        let wall_v = Mesh::from(shape::Box::new(s.x, s.y, s.z));

        let s = BlockType::WallSmallH.size();
        let wall_h = Mesh::from(shape::Box::new(s.x, s.y, s.z));

        let s = BlockType::Coin.size();
        let coin = Mesh::from(shape::Torus {
            radius: s.x,
            ring_radius: s.x * 0.25,
            subdivisions_segments: 8,
            subdivisions_sides: 6,
        });

        let enemy = Mesh::from(shape::Cube { size: 0.2 });
        let enemy_eye = Mesh::from(shape::Cube { size: 0.08 });

        let s = game_plugin::sizes::field;
        let floor_fg = Mesh::from(shape::Plane { size: s.x });
        let s = game_plugin::sizes::space;
        let floor_bg = Mesh::from(shape::Plane { size: s.x });
        let floor_cube = Mesh::from(shape::Cube { size: s.x });

        MeshHandles {
            wall: meshes.add(wall),
            wall_h: meshes.add(wall_h),
            wall_v: meshes.add(wall_v),
            coin: meshes.add(coin),
            enemy: meshes.add(enemy),
            enemy_eye: meshes.add(enemy_eye),
            floor_fg: meshes.add(floor_fg),
            floor_bg: meshes.add(floor_bg),
            floor_cube: meshes.add(floor_cube),
        }
    };
    commands.insert_resource(meshes);
}
