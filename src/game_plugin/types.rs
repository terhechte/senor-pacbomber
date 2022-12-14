use std::ops::Mul;

use super::statics::{sizes, LEVELS, LEVEL_BOMBS};
use bevy::prelude::*;

// ################################################################################
// General Helper Types
// ################################################################################

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum BlockType {
    WallBig,
    WallSmallV,
    WallSmallH,
    Coin,
    Enemy,
    Player,
    Space,
    Exit,
}

impl BlockType {
    pub fn size(&self) -> Vec3 {
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
            BlockType::Exit => sizes::space,
        }
    }

    pub fn is_wall(&self) -> bool {
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
            'e' => Exit,
            _ => panic!("Unknown Level char {c}"),
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Default, Hash)]
pub struct Position {
    pub x: usize,
    pub z: usize,
}

impl Position {
    pub fn new(x: usize, z: usize) -> Self {
        Self { x, z }
    }

    pub fn apply_direction(&mut self, direction: &BoardDirection) {
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
pub struct BoardDirection {
    pub x: i8,
    pub z: i8,
}

impl BoardDirection {
    pub fn new(x: i8, z: i8) -> Self {
        Self { x, z }
    }
}

impl Mul<Vec2> for BoardDirection {
    type Output = Vec2;
    #[inline]
    fn mul(self, rhs: Vec2) -> Vec2 {
        Vec2 {
            x: (self.x as f32).mul(rhs.x),
            y: (self.z as f32).mul(rhs.y),
        }
    }
}

impl Mul<i8> for BoardDirection {
    type Output = BoardDirection;
    #[inline]
    fn mul(self, rhs: i8) -> BoardDirection {
        BoardDirection {
            x: self.x * rhs,
            z: self.z * rhs,
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Block {
    pub kind: BlockType,
    pub position: Vec3,
    pub level_position: Position,
}

pub struct CurrentLevel(pub usize);

impl CurrentLevel {
    pub fn next(&self) -> Option<CurrentLevel> {
        if (self.0 + 1) >= LEVELS.len() {
            return None;
        }
        Some(CurrentLevel(self.0 + 1))
    }
}

// ################################################################################
// Components
// ################################################################################

#[derive(Component)]
pub struct Wall;

#[derive(Component)]
pub struct Enemy;

#[derive(Component)]
pub struct Coin;

#[derive(Component)]
pub struct Player;

#[derive(Component, Debug)]
pub struct Location(pub Position);

#[derive(Component)]
pub struct Floor;

#[derive(Component, Default, Debug)]
pub struct Movement {
    pub value: f32,
    pub direction: BoardDirection,
}

#[derive(Component)]
pub struct Wobbles(pub f32);

#[derive(Component, Default)]
pub struct Size(pub Vec3);

#[derive(Component, Default)]
pub struct Speed(pub f32);

#[derive(Component)]
pub struct Exit;

#[derive(Component)]
pub struct ExitLight;

pub struct Score {
    pub coins: usize,
    pub moves: usize,
    pub bombs: usize,
}

impl Default for Score {
    fn default() -> Self {
        Self {
            coins: 0,
            moves: 0,
            bombs: LEVEL_BOMBS[0],
        }
    }
}

#[derive(Component)]
pub struct Bomb(pub f32);

impl Bomb {
    pub fn new() -> Self {
        // Default bomb time is 2.5 seconds until explode
        Bomb(1.5)
    }
}

#[derive(Component)]
pub struct BombExplosion;

#[derive(Component)]
pub struct LevelItem;

// ################################################################################
// Events
// ################################################################################

pub struct ShowLevelExitEvent;

pub struct GoNextLevelEvent;

pub struct PlayerDiedEvent;
