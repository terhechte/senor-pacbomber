//! Tasks
//! - [ ] update the level with player and enemy positions
//! - [ ] make a level twice as high
//! - [ ] when a level is done, open a hole, going in there, falling into the next level
//! - [ ] add bomb update item to increase bomb range
//! - [ ] special item that rotates the level z 90. so that the controls are temporary inverse

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
    utils::HashMap,
    window::close_on_esc,
};
use bevy_mod_outline::*;
use bevy_tweening::{
    lens::{
        TransformPositionLens, TransformRotateXLens, TransformRotationLens, TransformScaleLens,
    },
    Animator, Delay, EaseFunction, Sequence, Tracks, Tween, TweenCompleted, TweeningPlugin,
    TweeningType,
};
use std::{
    cmp::Ordering,
    collections::HashSet,
    f32::consts::{PI, TAU},
    ops::Mul,
    time::Duration,
};

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
        .add_system(close_on_esc)
        .run();
}
