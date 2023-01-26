use crate::pieces::{Piece, PieceColor, PieceType};
use bevy::app::AppExit;
use bevy::prelude::*;
use bevy_mod_picking::{PickableBundle, PickingCamera};
use std::collections::{HashMap, HashSet};
#[derive(Component, Debug)]
pub struct Square {
    pub x: u8,
    pub y: u8,
}

impl Square {
    fn is_white(&self) -> bool {
        (self.x + self.y + 1) % 2 == 0
    }
}

fn create_board(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mesh = meshes.add(Mesh::from(shape::Plane { size: 1.0 }));

    for i in 0..8 {
        for j in 0..8 {
            let material = if (i + j) % 2 == 0 {
                Color::rgb(1., 0.9, 0.9)
            } else {
                Color::rgb(0., 0.1, 0.1)
            };
            let material = materials.add(material.into());
            commands.spawn((
                PbrBundle {
                    mesh: mesh.clone(),
                    material,
                    transform: Transform::from_translation(Vec3::new(i as f32, 0.0, j as f32)),
                    ..Default::default()
                },
                PickableBundle::default(),
                Square { x: i, y: j },
            ));
        }
    }
}

#[derive(Default, Resource)]
pub struct SelectedSquare {
    entity: Option<Entity>,
}

#[derive(Default, Resource)]
pub struct HighlightedSquares {
    entities: HashSet<(u8, u8)>,
}

#[derive(Default, Resource)]
struct SelectedPiece {
    entity: Option<Entity>,
}

#[derive(Resource)]
pub struct PlayerTurn(pub PieceColor);
impl Default for PlayerTurn {
    fn default() -> Self {
        Self(PieceColor::White)
    }
}
impl PlayerTurn {
    fn change(&mut self) {
        self.0 = match self.0 {
            PieceColor::White => PieceColor::Black,
            PieceColor::Black => PieceColor::White,
        }
    }
}

#[derive(Resource)]
pub struct SquareMaterials {
    highlight_color: Handle<StandardMaterial>,
    selected_color: Handle<StandardMaterial>,
    black_color: Handle<StandardMaterial>,
    white_color: Handle<StandardMaterial>,
}

impl FromWorld for SquareMaterials {
    fn from_world(world: &mut World) -> Self {
        let world = world.cell();
        let mut materials = world
            .get_resource_mut::<Assets<StandardMaterial>>()
            .unwrap();
        SquareMaterials {
            highlight_color: materials.add(Color::rgb(0.8, 0.3, 0.3).into()),
            selected_color: materials.add(Color::rgb(0.9, 0.1, 0.1).into()),
            black_color: materials.add(Color::rgb(0., 0.1, 0.1).into()),
            white_color: materials.add(Color::rgb(1., 0.9, 0.9).into()),
        }
    }
}

fn color_squares(
    selected_square: Res<SelectedSquare>,
    highlighted_squares: Res<HighlightedSquares>,
    materials: Res<SquareMaterials>,
    mut query: Query<(Entity, &Square, &mut Handle<StandardMaterial>)>,
    picking_camera_query: Query<&PickingCamera>,
) {
    // Get entity under the cursor, if there is one
    let top_entity = match picking_camera_query.iter().last() {
        Some(picking_camera) => match picking_camera.get_intersections() {
            Some([(entity, i)]) => Some(entity),
            _ => None,
        },
        None => return,
    };

    for (entity, square, mut material) in query.iter_mut() {
        // Change the material
        *material = if Some(&entity) == top_entity {
            materials.highlight_color.clone()
        } else if Some(entity) == selected_square.entity {
            materials.selected_color.clone()
        } else if highlighted_squares.entities.contains(&(square.x, square.y)) {
            materials.highlight_color.clone()
        } else if square.is_white() {
            materials.white_color.clone()
        } else {
            materials.black_color.clone()
        };
    }
}

fn select_square(
    mouse_button_inputs: Res<Input<MouseButton>>,
    mut selected_square: ResMut<SelectedSquare>,
    mut selected_piece: ResMut<SelectedPiece>,
    mut highlighted_squares: ResMut<HighlightedSquares>,
    squares_query: Query<&Square>,
    picking_camera_query: Query<&PickingCamera>,
) {
    // Only run if the left button is pressed
    if !mouse_button_inputs.just_pressed(MouseButton::Left) {
        return;
    }

    // Get the square under the cursor and set it as the selected
    if let Some(picking_camera) = picking_camera_query.iter().last() {
        if let Some((square_entity, _intersection)) = picking_camera.get_nearest_intersection() {
            if let Ok(_square) = squares_query.get(square_entity) {
                // Mark it as selected
                println!("select_square: {:?}", _square);
                selected_square.entity = Some(square_entity);
            }
        } else {
            // Player clicked outside the board, deselect everything
            selected_square.entity = None;
            selected_piece.entity = None;
            highlighted_squares.entities.clear();
        }
    }
}

fn select_piece(
    selected_square: Res<SelectedSquare>,
    mut selected_piece: ResMut<SelectedPiece>,
    mut highlighted_squares: ResMut<HighlightedSquares>,
    turn: Res<PlayerTurn>,
    squares_query: Query<&Square>,
    pieces_query: Query<(Entity, &Piece)>,
) {
    if !selected_square.is_changed() {
        return;
    }

    let square_entity = if let Some(entity) = selected_square.entity {
        entity
    } else {
        return;
    };

    let square = if let Ok(square) = squares_query.get(square_entity) {
        square
    } else {
        return;
    };

    // always clear the highlighted squares
    highlighted_squares.entities.clear();

    if selected_piece.entity.is_none() {
        // Select the piece in the currently selected square
        for (piece_entity, piece) in pieces_query.iter() {
            if piece.x == square.x && piece.y == square.y && piece.color == turn.0 {
                // high
                let pieces_map = pieces_query
                    .iter()
                    .map(|(e, p)| ((p.x, p.y), p))
                    .collect::<HashMap<(u8, u8), &Piece>>();
                println!("Selected piece {:?}", piece);
                highlighted_squares.entities = piece.possible_moves(pieces_map);
                // piece_entity is now the entity in the same square
                selected_piece.entity = Some(piece_entity);
                break;
            }
        }
    }
}

fn move_piece(
    mut commands: Commands,
    selected_square: Res<SelectedSquare>,
    selected_piece: Res<SelectedPiece>,
    highlighted_squares: ResMut<HighlightedSquares>,
    mut turn: ResMut<PlayerTurn>,
    squares_query: Query<&Square>,
    mut pieces_query: Query<(Entity, &mut Piece)>,
    mut reset_selected_event: EventWriter<ResetSelectedEvent>,
) {
    if !selected_square.is_changed() {
        return;
    }

    let square_entity = if let Some(entity) = selected_square.entity {
        entity
    } else {
        return;
    };

    let square = if let Ok(square) = squares_query.get(square_entity) {
        square
    } else {
        return;
    };

    if let Some(selected_piece_entity) = selected_piece.entity {
        let pieces_entity_vec = pieces_query
            .iter_mut()
            .map(|(entity, piece)| (entity, *piece))
            .collect::<Vec<(Entity, Piece)>>();
        // Move the selected piece to the selected square
        let mut piece =
            if let Ok((_piece_entity, piece)) = pieces_query.get_mut(selected_piece_entity) {
                piece
            } else {
                return;
            };

        if highlighted_squares.entities.contains(&(square.x, square.y)) {
            // Check if a piece of the opposite color exists in this square and despawn it
            for (other_entity, other_piece) in pieces_entity_vec {
                if other_piece.x == square.x
                    && other_piece.y == square.y
                    && other_piece.color != piece.color
                {
                    println!("taking piece {:?}", other_piece);
                    // Mark the piece as taken
                    commands.entity(other_entity).insert(Taken);
                }
            }

            // Move piece
            piece.x = square.x;
            piece.y = square.y;

            println!("moving piece {:?}", piece);
            // Change turn
            turn.change();
        }

        reset_selected_event.send(ResetSelectedEvent);
    }
}

struct ResetSelectedEvent;

fn reset_selected(
    mut event_reader: EventReader<ResetSelectedEvent>,
    mut selected_square: ResMut<SelectedSquare>,
    mut selected_piece: ResMut<SelectedPiece>,
) {
    for _event in event_reader.iter() {
        selected_square.entity = None;
        selected_piece.entity = None;
    }
}

#[derive(Component)]
struct Taken;
fn despawn_taken_pieces(
    mut commands: Commands,
    mut app_exit_events: EventWriter<AppExit>,
    query: Query<(Entity, &Piece, &Taken)>,
) {
    for (entity, piece, _taken) in query.iter() {
        // If the king is taken, we should exit
        if piece.piece_type == PieceType::King {
            println!(
                "{} won! Thanks for playing!",
                match piece.color {
                    PieceColor::White => "Black",
                    PieceColor::Black => "White",
                }
            );
            app_exit_events.send(AppExit);
        }

        // Despawn piece and children
        commands.entity(entity).despawn_recursive();
    }
}

pub struct BoardPlugin;

impl Plugin for BoardPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SelectedSquare>()
            .init_resource::<HighlightedSquares>()
            .init_resource::<SelectedPiece>()
            .init_resource::<SquareMaterials>()
            .init_resource::<PlayerTurn>()
            .add_event::<ResetSelectedEvent>()
            .add_startup_system(create_board)
            .add_system_to_stage(CoreStage::PostUpdate, color_squares)
            .add_system_to_stage(CoreStage::PostUpdate, select_square)
            .add_system_to_stage(
                CoreStage::PostUpdate,
                move_piece.after(select_square).before(select_piece),
            )
            .add_system_to_stage(CoreStage::PostUpdate, select_piece.after(select_square))
            .add_system_to_stage(
                CoreStage::PostUpdate,
                despawn_taken_pieces.after(move_piece),
            )
            .add_system_to_stage(CoreStage::PostUpdate, reset_selected.after(select_square));
    }
}
