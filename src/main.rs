use glam::Vec2;

use bevy::prelude::*;

const WINDOW_WIDTH: f32 = 640.;
const WINDOW_HEIGHT: f32 = 480.;

const PLAYER_SIZE: f32 = 50.;
struct Entity(u64);

#[derive(Component)]
struct Player;

#[derive(Component)]
struct Range;

#[derive(Component)]
struct Bomb;

#[derive(Component)]
struct Wall;

#[derive(Component, Default)]
struct Deltas {
    x: f32,
    y: f32,
}
fn main() {
    App::new()
        .insert_resource(WindowDescriptor {
            title: "A really fun Bomberman".to_string(),
            width: WINDOW_WIDTH,
            height: WINDOW_HEIGHT,
            vsync: true,
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_startup_system(setup)
        .add_system(player.system())
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
    commands
        .spawn_bundle(SpriteBundle {
            sprite: Sprite {
                color: Color::rgb(0.25, 0.25, 0.75),
                custom_size: Some(Vec2::new(PLAYER_SIZE, PLAYER_SIZE)),
                ..Default::default()
            },
            transform: Transform::from_translation(Vec3::new(0.0, -220.0, 0.0)),
            ..Default::default()
        })
        .insert(Player);
}

fn player(
    keyboard_input: Res<Input<KeyCode>>,
    mut commands: Commands,
    mut query: Query<(With<Player>, &mut Transform)>,
) {
    const DELTA: f32 = 1.;

    for (_, mut trans) in query.iter_mut() {
        let mut bomb = false;

        if keyboard_input.pressed(KeyCode::Left) {
            trans.translation.x -= DELTA;
        }
        if keyboard_input.pressed(KeyCode::Right) {
            trans.translation.x += DELTA;
        }

        if keyboard_input.pressed(KeyCode::Up) {
            trans.translation.y += DELTA;
        }

        if keyboard_input.pressed(KeyCode::Down) {
            trans.translation.y -= DELTA;
        }
        if keyboard_input.just_pressed(KeyCode::Space) {
            bomb = true;
        }

        // Apply movement deltas

        trans.translation.x = trans
            .translation
            .x
            .clamp(-WINDOW_WIDTH / 2., WINDOW_WIDTH / 2.);

        trans.translation.y = trans
            .translation
            .y
            .clamp(-WINDOW_HEIGHT / 2., WINDOW_HEIGHT / 2.);
    }
}
