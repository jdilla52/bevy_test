mod board;
mod example;
mod pieces;
mod ui;

use bevy::prelude::*;
use bevy::ui::UiPlugin;
use bevy_mod_picking::*;

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

use board::BoardPlugin;
use crate::pieces::PiecesPlugin;

fn main() {
    App::new()
        .insert_resource(Msaa {
            samples: 4,
            ..default()
        })
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            window: WindowDescriptor {
                width: 1600.0,
                height: 1600.0,
                ..default()
            },
            ..default()
        }))
        .add_plugin(PickingPlugin)
        .add_plugin(InteractablePickingPlugin)
        .add_plugin(BoardPlugin)
        .add_plugin(PiecesPlugin)
        // .add_plugin(UiPlugin)
        .add_startup_system(setup)
        .add_startup_system(pieces::create_pieces)
        // .add_plugin(PersonPlugin)
        .run();
}
