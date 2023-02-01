mod board;
mod example;
mod pieces;
mod ui;
mod scorer;

use crate::pieces::PiecesPlugin;
use crate::ui::ChessUIPlugin;
use bevy::prelude::*;
use bevy_mod_picking::*;
use board::BoardPlugin;

fn setup(mut commands: Commands) {
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_matrix(Mat4::from_rotation_translation(
                Quat::from_xyzw(-0.3, -0.5, -0.3, 0.5).normalize(),
                Vec3::new(-7.0, 20.0, 4.0),
            )),
            ..Default::default()
        },
        PickingCameraBundle::default(),
    ));
    commands.spawn(PointLightBundle {
        transform: Transform::from_translation(Vec3::new(4.0, 8.0, 4.0)),
        ..Default::default()
    });
}

fn main() {
    App::new()
        .insert_resource(Msaa {
            samples: 4,
            ..default()
        })
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            window: WindowDescriptor {
                width: 1000.0,
                height: 1000.0,
                ..default()
            },
            ..default()
        }))
        .add_plugin(PickingPlugin)
        .add_plugin(InteractablePickingPlugin)
        .add_plugin(BoardPlugin)
        .add_plugin(PiecesPlugin)
        .add_plugin(ChessUIPlugin)
        .add_startup_system(setup)
        .run();
}
