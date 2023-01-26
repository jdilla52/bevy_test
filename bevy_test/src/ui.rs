use crate::board::PlayerTurn;
use crate::pieces::PieceColor;
use bevy::prelude::*;

#[derive(Component)]
struct NextMove;

fn init_next_move(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut color_materials: ResMut<Assets<ColorMaterial>>,
) {
    let font = asset_server.load("fonts/FiraSans-Bold.ttf");

    commands
        .spawn(NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                position: UiRect {
                    left: Val::Px(10.),
                    top: Val::Px(10.),
                    ..Default::default()
                },
                ..Default::default()
            },
            background_color: Color::rgb(0.1, 0.1, 0.1).into(),
            ..Default::default()
        })
        .with_children(|parent| {
            parent
                .spawn(TextBundle {
                    text: Text::from_section(
                        "Next Move: White".to_string(),
                        TextStyle {
                            font,
                            font_size: 40.0,
                            color: Color::rgb(0.8, 0.8, 0.8),
                        },
                    ),
                    ..Default::default()
                })
                .insert(NextMove);
        });
}

fn next_move_text(turn: Res<PlayerTurn>, mut query: Query<&mut Text, With<NextMove>>) {
    if !turn.is_changed() {
        return;
    }
    for mut text in query.iter_mut() {
        text.sections[0].value = format!(
            "Next move: {}",
            match turn.0 {
                PieceColor::White => "White",
                PieceColor::Black => "Black",
            }
        );
    }
}

pub struct ChessUIPlugin;
impl Plugin for ChessUIPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(init_next_move)
            .add_system(next_move_text);
    }
}
