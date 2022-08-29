use bevy::prelude::*;
pub struct MaterialHandles {
    pub wall_normal: Handle<StandardMaterial>,
    pub wall_hidden: Handle<StandardMaterial>,
    pub coin: Handle<StandardMaterial>,
    pub player: Handle<StandardMaterial>,
    pub enemy: Handle<StandardMaterial>,
    pub floor_bg: Handle<StandardMaterial>,
    pub floor_fg: Handle<StandardMaterial>,
    pub ground: Handle<StandardMaterial>,
    pub bomb: Handle<StandardMaterial>,
    pub explosion: Handle<StandardMaterial>,
    pub white: Handle<StandardMaterial>,
}

pub struct MeshHandles {
    pub wall: Handle<Mesh>,
    pub wall_h: Handle<Mesh>,
    pub wall_v: Handle<Mesh>,
    pub coin: Handle<Mesh>,
    pub enemy: Handle<Mesh>,
    pub enemy_eye: Handle<Mesh>,
    pub floor_fg: Handle<Mesh>,
    pub floor_bg: Handle<Mesh>,
    pub floor_cube: Handle<Mesh>,
}
