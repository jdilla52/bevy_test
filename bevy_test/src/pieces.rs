use bevy::asset::{AssetServer, Assets, Handle};
use bevy::hierarchy::BuildChildren;
use bevy::math::Vec3;
use bevy::pbr::{PbrBundle, StandardMaterial};
use bevy::prelude::{Color, Commands, Component, Mesh, Res, ResMut, Transform};
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PieceType {
    Pawn,
    Rook,
    Knight,
    Bishop,
    Queen,
    King,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PieceColor {
    White,
    Black,
}

fn color_of_square(pos: (u8, u8), pieces: &Vec<Piece>) -> Option<PieceColor> {
    for piece in pieces {
        if piece.x == pos.0 && piece.y == pos.1 {
            return Some(piece.color);
        }
    }
    None
}

fn is_path_empty(begin: (u8, u8), end: (u8, u8), pieces: &Vec<Piece>) -> bool {
    if begin.0 == end.0 {
        for piece in pieces {
            if piece.x == begin.0
                && ((piece.y > begin.1 && piece.y < end.1)
                    || (piece.y > end.1 && piece.y > begin.1))
            {
                return false;
            }
        }
    }

    if begin.1 == end.1 {
        for piece in pieces {
            if piece.y == begin.1
                && ((piece.x > begin.0 && piece.x < end.0)
                    || (piece.x > end.0 && piece.x > begin.0))
            {
                return false;
            }
        }
    }

    let x_diff = (begin.0 as i8 - end.0 as i8).abs();
    let y_diff = (begin.1 as i8 - end.1 as i8).abs();
    if x_diff == y_diff {
        for i in 1..x_diff {
            let pos = if begin.0 < end.0 && begin.1 < end.1 {
                (begin.0 + i as u8, begin.1 + i as u8)
            } else if begin.0 < end.0 && begin.1 > end.1 {
                (begin.0 + i as u8, begin.1 - i as u8)
            } else if begin.0 > end.0 && begin.1 < end.1 {
                (begin.0 - i as u8, begin.1 + i as u8)
            } else {
                (begin.0 - i as u8, begin.1 - i as u8)
            };
            if color_of_square(pos, pieces).is_some() {
                return false;
            }
        }
    }
    true
}

#[derive(Debug, Clone, Copy, Component)]
pub struct Piece {
    pub piece_type: PieceType,
    pub color: PieceColor,
    pub x: u8,
    pub y: u8,
}

const KNIGHT_MOVES: [(i8, i8); 8] = [(-2, -1), (-2, 1), (-1, -2), (-1, 2), (1, -2), (1, 2), (2, -1), (2, 1)];

const KING_MOVES: [(i8, i8); 8] = [
    (-1, -1),
    (-1, 0),
    (-1, 1),
    (0, -1),
    (0, 1),
    (1, -1),
    (1, 0),
    (1, 1),
];

impl Piece {
    pub fn possible_moves(&self, pieces: HashMap<(u8, u8), &Piece>)-> Vec<(u8, u8)> {
        // todo add takes
        match &self.piece_type {
            PieceType::Pawn => {
                let mut moves = vec![];
                if self.color == PieceColor::Black {
                    let y1 = self.y as i8 - 1;
                    if y1 >= 0 {
                        let y1 = y1 as u8;
                        if let Some(p) = pieces.get(&(self.x, y1)) {
                            if p.color != self.color {
                                moves.push((self.x, y1));
                            }
                        } else {
                            moves.push((self.x, y1));
                        }
                    } else if self.y == 6 {
                        // todo implement en passant

                        let y2 = self.y as i8 - 2;
                        let uy2 = y2 as u8;
                        if let Some(p) = pieces.get(&(self.x, uy2)) {
                            if p.color != self.color {
                                moves.push((self.x, uy2));
                            }
                        } else {
                            moves.push((self.x, uy2));
                        }
                    }
                } else {
                    let y1 = self.y as i8 + 1;
                    if y1 <= 7 {
                        let y1 = y1 as u8;
                        if let Some(p) = pieces.get(&(self.x, y1)) {
                            if p.color != self.color {
                                moves.push((self.x, y1));
                            }
                        } else {
                            moves.push((self.x, y1));
                        }
                    } else if self.y == 1 {
                        // todo implement en passant
                        let y2 = self.y as i8 + 2;
                        let uy2 = y2 as u8;
                        if let Some(p) = pieces.get(&(self.x, uy2)) {
                            if p.color != self.color {
                                moves.push((self.x, uy2));
                            }
                        } else {
                            moves.push((self.x, uy2));
                        }
                    }
                }
                moves
            }
            PieceType::Rook => {
                let mut moves = vec![];
                for i in self.x + 1..8 {
                    if pieces.get(&(i, self.y)).is_some() {
                        break;
                    }
                    moves.push((i, self.y));
                }
                for i in 0..self.x {
                    if pieces.get(&(i, self.y)).is_some() {
                        break;
                    }
                    moves.push((i, self.y));
                }
                for i in self.y + 1..8 {
                    if pieces.get(&(self.x, i)).is_some() {
                        break;
                    }
                    moves.push((self.x, i));
                }
                for i in 0..self.y {
                    if pieces.get(&(self.x, i)).is_some() {
                        break;
                    }
                    moves.push((self.x, i));
                }
                moves
            },
            PieceType::Knight => {
                let mut moves = vec![];
                for i in KNIGHT_MOVES {
                    let pos = (self.x as i8 + i.0, self.y as i8 + i.1);
                    if pos.0 >= 0 && pos.0  < 8 && pos.1 >= 0 && pos.1 < 8 {
                        if pieces.get(&(pos.0 as u8, pos.1 as u8)).is_none() {
                            moves.push((pos.0 as u8, pos.1 as u8));
                        }
                    }
                }
                moves
            }
            PieceType::Bishop => {
                let mut moves = vec![];

                let sx = self.x as i8;
                let sy = self.y as i8;

                for i in 1..8 {
                    let x = sx+i;
                    let y = sy-i;
                    if x > 7 || y < 0 {
                        break;
                    } else if let Some(p) = pieces.get(&(x as u8, y as u8)) {
                        if p.color != self.color {
                            moves.push((x as u8, y as u8));
                        }
                        break;
                    }
                    moves.push((x as u8, y as u8));
                }

                for i in 1..8 {
                    let x = sx-i;
                    let y = sy+i;
                    if x < 0 || y > 7 {
                        break;
                    } else if let Some(p) = pieces.get(&(x as u8, y as u8)) {
                        if p.color != self.color {
                            moves.push((x as u8, y as u8));
                        }
                        break;
                    }
                    moves.push((x as u8, y as u8));
                }

                for i in 1..8 {
                    let x = sx+i;
                    let y = sy-i;
                    if x < 7 || y < 0 {
                        break;
                    } else if let Some(p) = pieces.get(&(x as u8, y as u8)) {
                        if p.color != self.color {
                            moves.push((x as u8, y as u8));
                        }
                        break;
                    }
                    moves.push((x as u8, y as u8));
                }

                for i in 1..8 {
                    let x = sx-i;
                    let y = sy-i;
                    if x < 0 || y < 0 {
                        break;
                    } else if let Some(p) = pieces.get(&(x as u8, y as u8)) {
                        if p.color != self.color {
                            moves.push((x as u8, y as u8));
                        }
                        break;
                    }
                    moves.push((x as u8, y as u8));
                }

                moves
            },
            PieceType::Queen => {
                let mut moves = vec![];
                for i in self.x + 1..8 {
                    if pieces.get(&(i, self.y)).is_some() {
                        break;
                    }
                    moves.push((i, self.y));
                }
                for i in 0..self.x {
                    if pieces.get(&(i, self.y)).is_some() {
                        break;
                    }
                    moves.push((i, self.y));
                }
                for i in self.y + 1..8 {
                    if pieces.get(&(self.x, i)).is_some() {
                        break;
                    }
                    moves.push((self.x, i));
                }
                for i in 0..self.y {
                    if pieces.get(&(self.x, i)).is_some() {
                        break;
                    }
                    moves.push((self.x, i));
                }

                for i in self.x + 1..8 {
                    for j in self.y + 1..8 {
                        if pieces.get(&(i, j)).is_some() {
                            break;
                        }
                        moves.push((i, j));
                    }
                }

                for i in 0..self.x {
                    for j in self.y + 1..8 {
                        if pieces.get(&(i, j)).is_some() {
                            break;
                        }
                        moves.push((i, j));
                    }
                }

                for i in self.x + 1..8 {
                    for j in 0..self.y {
                        if pieces.get(&(i, j)).is_some() {
                            break;
                        }
                        moves.push((i, j));
                    }
                }

                for i in 0..self.x {
                    for j in 0..self.y {
                        if pieces.get(&(i, j)).is_some() {
                            break;
                        }
                        moves.push((i, j));
                    }
                }

                moves
            }
            PieceType::King => {
                let mut moves = vec![];
                for i in KING_MOVES.iter() {
                    let pos = (self.x as i8 + i.0, self.y as i8 + i.1);
                    if pos.0 >= 0 && pos.0  < 8 && pos.1 >= 0 && pos.1 < 8 {
                        if pieces.get(&(pos.0 as u8, pos.1 as u8)).is_none() {
                            moves.push((pos.0 as u8, pos.1 as u8));
                        }
                    }
                }
                moves
            }
        }
    }
    pub fn is_move_valid(&self, new_position: (u8, u8), pieces: Vec<Piece>) -> bool {
        match self.piece_type {
            PieceType::Pawn => {
                if self.color == PieceColor::White {
                    // Normal move
                    if new_position.0 as i8 - self.x as i8 == 1 && (self.y == new_position.1) {
                        if color_of_square(new_position, &pieces).is_none() {
                            return true;
                        }
                    }

                    // Move 2 squares
                    if self.x == 1
                        && new_position.0 as i8 - self.x as i8 == 2
                        && (self.y == new_position.1)
                        && is_path_empty((self.x, self.y), new_position, &pieces)
                    {
                        if color_of_square(new_position, &pieces).is_none() {
                            return true;
                        }
                    }

                    // Take piece
                    if new_position.0 as i8 - self.x as i8 == 1
                        && (self.y as i8 - new_position.1 as i8).abs() == 1
                    {
                        if color_of_square(new_position, &pieces) == Some(PieceColor::Black) {
                            return true;
                        }
                    }
                } else {
                    // Normal move
                    if new_position.0 as i8 - self.x as i8 == -1 && (self.y == new_position.1) {
                        if color_of_square(new_position, &pieces).is_none() {
                            return true;
                        }
                    }

                    // Move 2 squares
                    if self.x == 6
                        && new_position.0 as i8 - self.x as i8 == -2
                        && (self.y == new_position.1)
                        && is_path_empty((self.x, self.y), new_position, &pieces)
                    {
                        if color_of_square(new_position, &pieces).is_none() {
                            return true;
                        }
                    }

                    // Take piece
                    if new_position.0 as i8 - self.x as i8 == -1
                        && (self.y as i8 - new_position.1 as i8).abs() == 1
                    {
                        if color_of_square(new_position, &pieces) == Some(PieceColor::White) {
                            return true;
                        }
                    }
                }

                false
            }
            PieceType::Rook => {
                is_path_empty((self.x, self.y), new_position, &pieces)
                    && ((self.x == new_position.0 && self.y != new_position.1)
                        || (self.y == new_position.1 && self.x != new_position.0))
            }
            PieceType::Knight => {
                ((self.x as i8 - new_position.0 as i8).abs() == 2
                    && (self.y as i8 - new_position.1 as i8).abs() == 1)
                    || ((self.x as i8 - new_position.0 as i8).abs() == 1
                        && (self.y as i8 - new_position.1 as i8).abs() == 2)
            }
            PieceType::Bishop => {
                is_path_empty((self.x, self.y), new_position, &pieces)
                    && (self.x as i8 - new_position.0 as i8).abs()
                        == (self.y as i8 - new_position.1 as i8).abs()
            }
            PieceType::Queen => {
                is_path_empty((self.x, self.y), new_position, &pieces)
                    && ((self.x as i8 - new_position.0 as i8).abs()
                        == (self.y as i8 - new_position.1 as i8).abs()
                        || ((self.x == new_position.0 && self.y != new_position.1)
                            || (self.y == new_position.1 && self.x != new_position.0)))
            }
            PieceType::King => {
                // Horizontal
                ((self.x as i8 - new_position.0 as i8).abs() == 1
                        && (self.y == new_position.1))
                        // Vertical
                        || ((self.y as i8 - new_position.1 as i8).abs() == 1
                        && (self.x == new_position.0))
                        // Diagonal
                        || ((self.x as i8 - new_position.0 as i8).abs() == 1
                        && (self.y as i8 - new_position.1 as i8).abs() == 1)
            }
        }
    }
}

pub fn spawn_two(
    commands: Rc<RefCell<Commands>>,
    material: Handle<StandardMaterial>,
    piece_type: PieceType,
    piece_color: PieceColor,
    mesh: Handle<Mesh>,
    mesh_cross: Handle<Mesh>,
    position: Vec3,
    transform1: Vec3,
    transform2: Vec3,
) {
    commands
        .borrow_mut()
        .spawn(PbrBundle {
            transform: Transform::from_translation(position),
            ..Default::default()
        })
        .insert(Piece {
            piece_type,
            color: piece_color,
            x: position[0] as u8,
            y: position[2] as u8,
        })
        .with_children(|parent| {
            parent.spawn(PbrBundle {
                mesh,
                material: material.clone(),
                transform: {
                    let mut transform = Transform::from_translation(transform1);
                    transform.scale *= Vec3::new(0.2, 0.2, 0.2);
                    transform
                },
                ..Default::default()
            });
            parent.spawn(PbrBundle {
                mesh: mesh_cross,
                material,
                transform: {
                    let mut transform = Transform::from_translation(transform2);
                    transform.scale *= Vec3::new(0.2, 0.2, 0.2);
                    transform
                },
                ..Default::default()
            });
        });
}

fn spawn_one(
    commands: Rc<RefCell<Commands>>,
    material: Handle<StandardMaterial>,
    piece_type: PieceType,
    piece_color: PieceColor,
    mesh: Handle<Mesh>,
    position: Vec3,
    transform: Vec3,
) {
    commands
        .borrow_mut()
        .spawn(PbrBundle {
            transform: Transform::from_translation(position),
            ..Default::default()
        })
        .insert(Piece {
            piece_type,
            color: piece_color,
            x: position[0] as u8,
            y: position[2] as u8,
        })
        .with_children(|parent| {
            parent.spawn(PbrBundle {
                mesh,
                material,
                transform: {
                    let mut transform = Transform::from_translation(transform);
                    transform.scale *= Vec3::new(0.2, 0.2, 0.2);
                    transform
                },
                ..Default::default()
            });
        });
}

pub fn create_pieces(
    commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let king_handle: Handle<Mesh> =
        asset_server.load("models/chess_kit/pieces.glb#Mesh0/Primitive0");
    let king_cross_handle: Handle<Mesh> =
        asset_server.load("models/chess_kit/pieces.glb#Mesh1/Primitive0");
    let pawn_handle: Handle<Mesh> =
        asset_server.load("models/chess_kit/pieces.glb#Mesh2/Primitive0");
    let knight_1_handle: Handle<Mesh> =
        asset_server.load("models/chess_kit/pieces.glb#Mesh3/Primitive0");
    let knight_2_handle: Handle<Mesh> =
        asset_server.load("models/chess_kit/pieces.glb#Mesh4/Primitive0");
    let rook_handle: Handle<Mesh> =
        asset_server.load("models/chess_kit/pieces.glb#Mesh5/Primitive0");
    let bishop_handle: Handle<Mesh> =
        asset_server.load("models/chess_kit/pieces.glb#Mesh6/Primitive0");
    let queen_handle: Handle<Mesh> =
        asset_server.load("models/chess_kit/pieces.glb#Mesh7/Primitive0");

    let white_material = materials.add(Color::rgb(1.0, 0.8, 0.8).into());
    let black_material = materials.add(Color::rgb(0., 0.2, 0.2).into());
    let commands = Rc::new(RefCell::new(commands));
    // rook
    spawn_one(
        commands.clone(),
        white_material.clone(),
        PieceType::Rook,
        PieceColor::White,
        rook_handle.clone(),
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(-0.1, 0., 1.8),
    );
    spawn_two(
        commands.clone(),
        white_material.clone(),
        PieceType::Knight,
        PieceColor::White,
        knight_1_handle.clone(),
        knight_2_handle.clone(),
        Vec3::new(0., 0., 1.),
        Vec3::new(-0.2, 0., 0.9),
        Vec3::new(-0.2, 0., 0.9),
    );
    spawn_one(
        commands.clone(),
        white_material.clone(),
        PieceType::Bishop,
        PieceColor::White,
        bishop_handle.clone(),
        Vec3::new(0.0, 0.0, 2.),
        Vec3::new(-0.1, 0., 0.),
    );
    spawn_one(
        commands.clone(),
        white_material.clone(),
        PieceType::Queen,
        PieceColor::White,
        queen_handle.clone(),
        Vec3::new(0.0, 0.0, 3.),
        Vec3::new(-0.2, 0., -0.95),
    );
    spawn_two(
        commands.clone(),
        white_material.clone(),
        PieceType::King,
        PieceColor::White,
        king_handle.clone(),
        king_cross_handle.clone(),
        Vec3::new(0., 0., 4.),
        Vec3::new(-0.2, 0., -1.9),
        Vec3::new(-0.2, 0., -1.9),
    );
    spawn_one(
        commands.clone(),
        white_material.clone(),
        PieceType::Bishop,
        PieceColor::White,
        bishop_handle.clone(),
        Vec3::new(0.0, 0.0, 5.),
        Vec3::new(-0.1, 0., 0.),
    );
    spawn_two(
        commands.clone(),
        white_material.clone(),
        PieceType::Knight,
        PieceColor::White,
        knight_1_handle.clone(),
        knight_2_handle.clone(),
        Vec3::new(0., 0., 6.),
        Vec3::new(-0.2, 0., 0.9),
        Vec3::new(-0.2, 0., 0.9),
    );
    spawn_one(
        commands.clone(),
        white_material.clone(),
        PieceType::Rook,
        PieceColor::White,
        rook_handle.clone(),
        Vec3::new(0.0, 0.0, 7.),
        Vec3::new(-0.1, 0., 1.8),
    );

    for i in 0..8 {
        spawn_one(
            commands.clone(),
            white_material.clone(),
            PieceType::Pawn,
            PieceColor::White,
            pawn_handle.clone(),
            Vec3::new(1.0, 0., i as f32),
            Vec3::new(-0.2, 0., 2.6),
        );
    }
    spawn_one(
        commands.clone(),
        black_material.clone(),
        PieceType::Rook,
        PieceColor::Black,
        rook_handle.clone(),
        Vec3::new(7., 0.0, 0.0),
        Vec3::new(-0.1, 0., 1.8),
    );
    spawn_two(
        commands.clone(),
        black_material.clone(),
        PieceType::Knight,
        PieceColor::Black,
        knight_1_handle.clone(),
        knight_2_handle.clone(),
        Vec3::new(7., 0., 1.),
        Vec3::new(-0.2, 0., 0.9),
        Vec3::new(-0.2, 0., 0.9),
    );
    spawn_one(
        commands.clone(),
        black_material.clone(),
        PieceType::Bishop,
        PieceColor::Black,
        bishop_handle.clone(),
        Vec3::new(7., 0.0, 2.),
        Vec3::new(-0.1, 0., 0.),
    );
    spawn_one(
        commands.clone(),
        black_material.clone(),
        PieceType::Queen,
        PieceColor::Black,
        queen_handle.clone(),
        Vec3::new(7., 0.0, 3.),
        Vec3::new(-0.2, 0., -0.95),
    );
    spawn_two(
        commands.clone(),
        black_material.clone(),
        PieceType::King,
        PieceColor::Black,
        king_handle.clone(),
        king_cross_handle.clone(),
        Vec3::new(7., 0., 4.),
        Vec3::new(-0.2, 0., -1.9),
        Vec3::new(-0.2, 0., -1.9),
    );
    spawn_one(
        commands.clone(),
        black_material.clone(),
        PieceType::Bishop,
        PieceColor::Black,
        bishop_handle.clone(),
        Vec3::new(7., 0.0, 5.),
        Vec3::new(-0.1, 0., 0.),
    );
    spawn_two(
        commands.clone(),
        black_material.clone(),
        PieceType::Knight,
        PieceColor::Black,
        knight_1_handle.clone(),
        knight_2_handle.clone(),
        Vec3::new(7., 0., 6.),
        Vec3::new(-0.2, 0., 0.9),
        Vec3::new(-0.2, 0., 0.9),
    );
    spawn_one(
        commands.clone(),
        black_material.clone(),
        PieceType::Rook,
        PieceColor::Black,
        rook_handle.clone(),
        Vec3::new(7., 0.0, 7.),
        Vec3::new(-0.1, 0., 1.8),
    );

    for i in 0..8 {
        spawn_one(
            commands.clone(),
            black_material.clone(),
            PieceType::Pawn,
            PieceColor::Black,
            pawn_handle.clone(),
            Vec3::new(6., 0., i as f32),
            Vec3::new(-0.2, 0., 2.6),
        );
    }
}
use bevy::prelude::*;

fn move_pieces(time: Res<Time>, mut query: Query<(&mut Transform, &Piece)>) {
    for (mut transform, piece) in query.iter_mut() {
        let direction = Vec3::new(piece.x as f32, 0., piece.y as f32) - transform.translation;

        if direction.length() > 0.1 {
            transform.translation += direction.normalize() * time.delta_seconds();
        }
    }
}

pub struct PiecesPlugin;

impl Plugin for PiecesPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(create_pieces)
            .add_system(move_pieces);
    }
}
