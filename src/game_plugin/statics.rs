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

pub const LEVEL_COMPLETED_PAYLOAD: u64 = 42;
pub const USER_DIED_PAYLOAD: u64 = 43;

pub const PLAYER_SPEED: f32 = 0.25;
pub const ENEMY_SPEED_EASY: f32 = 0.5;

pub const LEVELS: &[&str] = &[L1, L2, L3, L4, L5];
pub const LEVEL_BOMBS: &[usize] = &[3, 3, 3, 5, 5];

const L1: &str = r#"
#################
|  *  *  *   #x |
#  #  #---   #  #
#  *  |  *   |  #
#  #  |  #   |  #
#  *  *  *   |  #
#          e    #
|  o            |
#################
"#;

const L2: &str = r#"
#####################
|  *  *  *   #  * * |
###-##-  ##### #### #
| * *     *|*   * * |
#---##-  ##### #### #
|  *  *  * e # * *  |
|          o        |
#  -##-  ##### ---###
| * *|  *  *  *  * x|
#####################
"#;

const L3: &str = r#"
#---------------------#
| x*  *  * | *  *  *x |
#--- ----- # ------ --#
#  *  *  *   *  *  *  #
#   ##   # o #   ##   #
|  *| *  #   #  * |*  |
|  *| *  *   *  * |*  |
|  *| *  * e *  * |*  |
#---##-----------##---#
"#;

const L4: &str = r#"
#---------------------#
|x *  *  *   *  *  * x|
#---- --- -#- ---- ---#
|  *  *  *   *  *  *  |
#--- ----- # -----# # #
|  *  *  # e #  * |***|
##  ------ # -----| # #
| x*| *  * o *  * |***|
#---##-----------##---#
"#;

pub const L5: &str = r#"
#-----------------------#
|x* * * * * o * * * * *x|
| ##---- #-----# ----## |
| #* *   #  x  #   * *# |
| #----  # # # #  ----# |
| * * * * *   * * * * * |
| --#--- ## e ## ---#-- |
|  *|*             *|*  |
#-----------------------#
"#;
