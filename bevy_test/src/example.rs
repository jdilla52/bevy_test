use bevy::app::{App, Plugin};
use bevy::prelude::{
    Commands, Component, Query, Res, ResMut, Resource, Time, Timer, TimerMode, With,
};

#[derive(Component)]
struct Person;

#[derive(Component)]
struct Name(String);

fn add_people(mut commands: Commands) {
    commands.spawn((Person, Name("Elaina Proctor".to_string())));
    commands.spawn((Person, Name("Renzo Hume".to_string())));
    commands.spawn((Person, Name("Zayna Nieves".to_string())));
}

#[derive(Resource)]
struct GreetTimer(Timer);

fn greet_people(time: Res<Time>, mut timer: ResMut<GreetTimer>, query: Query<&Name, With<Person>>) {
    if timer.0.tick(time.delta()).just_finished() {
        for name in query.iter() {
            println!("hello {}!", name.0);
        }
    }
}

pub struct PersonPlugin;

impl Plugin for PersonPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(GreetTimer(Timer::from_seconds(-1.0, TimerMode::Repeating)))
            .add_startup_system(add_people)
            // .add_startup_system(setup)
            .add_system(greet_people);
    }
}
