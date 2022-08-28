use bevy::{prelude::*, utils::HashMap};
use std::collections::HashSet;

use super::statics::{sizes, LEVELS};
use super::types::*;

#[derive(Debug)]
pub struct Level {
    pub size: Position,
    pub offsets: (f32, f32),
    pub rows: Vec<Vec<Block>>,
    pub player_position: Position,
    pub ending_position: Position,
    pub enemy_positions: HashMap<Entity, Position>,
    pub coin_positions: HashMap<Entity, Position>,
    pub bombs: HashMap<Entity, (usize, Position)>,
    pub bomb_size: usize,
    pub ending_visible: bool,
}

impl Level {
    pub fn new(level: usize) -> Self {
        let data = LEVELS[level];
        let mut rows: Vec<Vec<_>> = Vec::new();

        let lines: Vec<&str> = data.split('\n').filter(|e| !e.is_empty()).collect();
        let z_offset = (sizes::field.z * (lines.len() as f32)) / 2.0;
        let mut x_offset: f32 = 0.0;
        let v_b = sizes::field;

        let z_size = lines.len();
        let mut x_size = 0;

        let mut player_position: Option<Position> = None;
        let mut ending_position: Option<Position> = None;

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

                let level_position = Position::new(z_index, x_index);

                if matches!(block, BlockType::Player) {
                    player_position = Some(level_position);
                }

                if matches!(block, BlockType::Exit) {
                    ending_position = Some(level_position);
                }

                row.push(Block {
                    kind: block,
                    position: Vec3::new(position.1, 0.0, position.0),
                    level_position,
                })
            }

            rows.push(row);
        }

        let player_position = player_position.expect("Expect a player position in the level!");
        let ending_position = ending_position.expect("Expect an ending position in the level!");

        Level {
            size: Position::new(x_size, z_size),
            offsets: (x_offset, z_offset),
            rows,
            player_position,
            ending_position,
            enemy_positions: HashMap::new(),
            coin_positions: HashMap::new(),
            bombs: HashMap::new(),
            bomb_size: 5,
            ending_visible: false,
        }
    }

    pub fn rows(&self) -> impl Iterator<Item = &Vec<Block>> {
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

    pub fn place_bomb(&mut self, entity: Entity, position: Position) {
        self.bombs.insert(entity, (self.bomb_size, position));
    }

    // All positions where the bomb will go except for walls
    // returns: (Position, current range, max range)
    pub fn bomb_explode_positions(&self, entity: Entity) -> Vec<(Position, usize, usize)> {
        let (range, position) = match self.bombs.get(&entity) {
            Some(n) => n,
            None => return Vec::new(),
        };
        let mut results = vec![(*position, 0, *range)];
        fn follow_range(
            level: &Level,
            range: i8,
            position: Position,
            direction: BoardDirection,
            into: &mut Vec<(Position, usize, usize)>,
        ) {
            let mut current_range = 1;
            loop {
                let current = direction * current_range;
                let (x, z) = (position.x as i8 + current.x, position.z as i8 + current.z);
                let item = match level.get(x, z) {
                    Some(n) => n,
                    None => break,
                };
                if item.kind.is_wall() {
                    break;
                }
                into.push((
                    Position::new(x as usize, z as usize),
                    current_range as usize,
                    range as usize,
                ));
                current_range += 1;
                if range == current_range {
                    break;
                }
            }
        }
        // go in all 4 directions
        follow_range(
            self,
            *range as i8,
            *position,
            BoardDirection::new(-1, 0),
            &mut results,
        );
        follow_range(
            self,
            *range as i8,
            *position,
            BoardDirection::new(0, -1),
            &mut results,
        );
        follow_range(
            self,
            *range as i8,
            *position,
            BoardDirection::new(1, 0),
            &mut results,
        );
        follow_range(
            self,
            *range as i8,
            *position,
            BoardDirection::new(0, 1),
            &mut results,
        );

        results
    }

    pub fn translate_from_position(&self, position: Position) -> Vec3 {
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
    pub fn free_directions(&self, position: Position) -> Vec<BoardDirection> {
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
            results.push(BoardDirection::new(mx, mz))
        }
        results
    }

    /// All connected wall positions that are z below +1 from the current position
    pub fn wall_positions(&self, position: Position) -> Vec<Position> {
        let mut new_position = position;
        new_position.apply_direction(&BoardDirection::new(0, 1));
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
                new.apply_direction(&BoardDirection::new(mx, mz));
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
