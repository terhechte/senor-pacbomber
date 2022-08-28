pub mod sizes {
    #![allow(non_upper_case_globals)]
    use bevy::prelude::*;
    pub const field: Vec3 = Vec3::new(0.25, 0.25, 0.25);
    pub const space: Vec3 = Vec3::new(0.24, -0.1, 0.24);
    pub const brick: Vec3 = Vec3::new(0.25, 0.25, 0.25);
    pub const brick_small: f32 = 0.1;
    pub const coin: Vec3 = Vec3::new(0.10, 0.05, 0.1);
    pub const enemy: Vec3 = Vec3::new(0.10, 0.05, 0.1);
    pub const bomb_size: f32 = 0.15;
}

pub const FPS: f32 = 60.0;
pub const TIME_STEP: f32 = 1.0 / FPS;

pub const LEVEL_COMPLETED_PAYLOAD: u64 = 42;

pub const LEVEL_DATA: &str = r#"
#----------   ----------#
| * * * * * ** * * * *  |
| ##---- #-----# ----## |
| #* *   #  x  #   * *# |
  #----  # ### #  ----#  
| * * * * *   * * * * * |
| --#--- ## e ## ---#-- |
|  *|* o           *|*  |
#----------   ----------#
"#;

pub const LEVEL_DATA2: &str = r#"
#----------   ----------#
| * * * * * ** * * * *  |
| ##---- #-----# ----## |
| #* *   #x x x#   * *# |
  #----  # ### #  ----#  
| * * * * *   * * * * * |
| --#--- ## o ## ---#-- |
|  *|* e           *|*  |
#----------   ----------#
"#;
