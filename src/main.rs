use bevy::prelude::*;
use glam::Vec2;

const WINDOW_WIDTH: f32 = 480.;
const WINDOW_HEIGHT: f32 = WINDOW_WIDTH;
const CELL_PER_ROW_COUNT: u32 = 11;
const CELL_SIZE: f32 = WINDOW_WIDTH / CELL_PER_ROW_COUNT as f32;

const PLAYER_SIZE: f32 = 0.85 * CELL_SIZE;

#[derive(Component)]
struct Player;

#[derive(Component)]
struct Explosion;

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

    // player
    commands
        .spawn_bundle(SpriteBundle {
            sprite: Sprite {
                color: Color::rgb(0.25, 0.25, 0.75),
                custom_size: Some(Vec2::new(PLAYER_SIZE, PLAYER_SIZE)),
                ..Default::default()
            },
            transform: Transform::from_translation(Vec3::new(0., 0., 0.0)),
            ..Default::default()
        })
        .insert(Player);

    // walls
    (0..11)
        .flat_map(|i| {
            [
                (0, i),
                (i, 0),
                (i, CELL_PER_ROW_COUNT - 1),
                (CELL_PER_ROW_COUNT - 1, i),
            ]
        })
        .chain(
            (0..CELL_PER_ROW_COUNT / 2)
                .flat_map(|x| (0..CELL_PER_ROW_COUNT / 2).map(move |y| (2 * x, 2 * y))),
        )
        .for_each(|(i, j)| {
            commands
                .spawn_bundle(SpriteBundle {
                    sprite: Sprite {
                        color: Color::rgb(0.3, 0.2, 0.2),
                        custom_size: Some(Vec2::new(CELL_SIZE, CELL_SIZE)),
                        ..Default::default()
                    },
                    transform: Transform::from_translation(Vec3::new(
                        i as f32 * CELL_SIZE - (WINDOW_WIDTH - CELL_SIZE) / 2.,
                        j as f32 * CELL_SIZE - (WINDOW_HEIGHT - CELL_SIZE) / 2.,
                        0.0,
                    )),
                    ..Default::default()
                })
                .insert(Wall);
        });
}

fn align_on_grid(x: f32, y: f32) -> (f32, f32) {
    (x, y)
}

/// take input into consideration
/// todo: maybe spawn bomb
fn register_input(
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<(With<Player>, &mut Deltas)>,
) {
}

/// called at fixed time ticks
/// update the transforms
/// reset deltas
/// maybe spawn bombs
fn update_player_position(
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<(With<Player>, &mut Deltas, &mut Transform)>,
) {
}

/// called before update_player_position
/// clamp deltas
/// maybe call at fixed time ticks
/// check collision with wall and breakable
fn check_collision(mut query: Query<(With<Player>, &mut Deltas, &mut Transform)>) {}

/// check if explosion happened
/// called at fixed timestamp
/// if it exploded, spawn explosion
/// despawn bomb
fn explode(mut query: Query<(&Bomb)>) {}

/// called at fixed timestamp
/// check if explosion has elapsed
/// despawn explosion
fn put_down_explosion(mut query: Query<(&Explosion)>) {}

/// death
fn check_death(mut query: Query<(With<Player>, &mut Transform)>) {}

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

        trans.translation.x = trans.translation.x.clamp(
            (-WINDOW_WIDTH + PLAYER_SIZE) / 2.,
            (WINDOW_WIDTH - PLAYER_SIZE) / 2.,
        );

        trans.translation.y = trans.translation.y.clamp(
            (-WINDOW_HEIGHT + PLAYER_SIZE) / 2.,
            (WINDOW_HEIGHT - PLAYER_SIZE) / 2.,
        );

        // if bomb {
        //     commands
        //         .spawn()
        //         .insert_bundle(SpriteBundle {
        //             transform: Transform::from_translation(Vec3::new(
        //                 trans.translation.x,
        //                 trans.translation.y,
        //                 0.0,
        //             )),
        //             sprite: Sprite {
        //                 color: Color::rgb(0., 0., 0.),
        //                 custom_size: Some(Vec2::new(PLAYER_SIZE, PLAYER_SIZE)),
        //                 ..Default::default()
        //             },
        //             ..Default::default()
        //         })
        //         .insert(Bomb);
        // }
    }
}
