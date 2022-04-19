use bevy::{
    core::FixedTimestep,
    prelude::*,
    sprite::collide_aabb::{collide, Collision},
};
use glam::Vec2;
use std::process::exit;
use std::time::Duration;

const WINDOW_WIDTH: f32 = 480.;
const WINDOW_HEIGHT: f32 = WINDOW_WIDTH;
const CELL_PER_ROW_COUNT: u8 = 11;
const CELL_SIZE: f32 = WINDOW_WIDTH / CELL_PER_ROW_COUNT as f32;
const BOMB_COUNTDOWN: f32 = 3.;
const PLAYER_SIZE: f32 = 0.85 * CELL_SIZE;
const DELTA: f32 = 1.;
const FRAME_DURATION: f64 = 1. / 60.;

#[derive(Component)]
struct Player {
    bomb_range: u8,
    bomb_set: bool,
}

impl Default for Player {
    fn default() -> Self {
        Player {
            bomb_range: 1,
            bomb_set: false,
        }
    }
}

#[derive(Component)]
struct Explosion(Timer);

#[derive(Component)]
struct Bomb {
    explosion_range: u8,
    countdown: Timer,
}

#[derive(Component)]
struct Wall;

#[derive(Component)]
struct NotTraversable;

#[derive(Component)]
struct Size(f32);

#[derive(Component, Default)]
struct Deltas {
    x: f32,
    y: f32,
}

#[derive(Component)]
struct Keys {
    up: KeyCode,
    down: KeyCode,
    right: KeyCode,
    left: KeyCode,
    bomb: KeyCode,
}

impl Deltas {
    fn norm(&mut self) {
        if self.x != 0. && self.y != 0. {
            let n = (self.x.powi(2) + self.y.powi(2)).sqrt();
            self.x /= n;
            self.y /= n;
        }
    }

    fn reset(&mut self) {
        self.x = 0.;
        self.y = 0.;
    }
}

#[derive(SystemLabel, Debug, Hash, PartialEq, Eq, Clone)]
enum Step {
    Input,
    Movement,
    Death,
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
        .add_system(register_input.label(Step::Input).before(Step::Movement))
        .add_system(check_collision.label(Step::Input).before(Step::Movement))
        .add_system_set(
            SystemSet::new()
                .with_run_criteria(FixedTimestep::step(FRAME_DURATION))
                .with_system(update_player_position.label(Step::Movement))
                .with_system(spawn_bombs.label(Step::Movement))
                .with_system(explode.label(Step::Movement))
                .with_system(put_down_explosion.label(Step::Movement)),
        )
        .add_system_set(
            SystemSet::new()
                .with_run_criteria(FixedTimestep::step(FRAME_DURATION))
                .with_system(check_death.label(Step::Death).after(Step::Movement))
                .with_system(check_game_over.label(Step::Death).after(Step::Movement)),
        )
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
            transform: Transform::from_translation(Vec3::new(
                cell_to_world(1),
                cell_to_world(1),
                0.0,
            )),
            ..Default::default()
        })
        .insert(Player::default())
        .insert(Deltas::default())
        .insert(NotTraversable)
        .insert(Size(PLAYER_SIZE))
        .insert(Keys {
            up: KeyCode::Up,
            down: KeyCode::Down,
            right: KeyCode::Right,
            left: KeyCode::Left,
            bomb: KeyCode::Space,
        });

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
                        cell_to_world(i),
                        cell_to_world(j),
                        1.0,
                    )),
                    ..Default::default()
                })
                .insert(Wall)
                .insert(NotTraversable)
                .insert(Size(CELL_SIZE));
        });
}

fn align_on_grid(x: f32) -> f32 {
    let cell = ((x + WINDOW_WIDTH / 2.) / CELL_SIZE).floor();
    cell * CELL_SIZE - WINDOW_WIDTH / 2. + CELL_SIZE / 2.
}

fn cell_to_world(x: u8) -> f32 {
    x as f32 * CELL_SIZE - WINDOW_WIDTH / 2. + CELL_SIZE / 2.
}

/// take input into consideration
/// todo: maybe spawn bomb
fn register_input(
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<(&mut Player, &mut Deltas, &Keys)>,
) {
    for (mut p, mut delta, k) in query.iter_mut() {
        if keyboard_input.pressed(k.left) {
            delta.x -= DELTA;
        }
        if keyboard_input.pressed(k.right) {
            delta.x += DELTA;
        }

        if keyboard_input.pressed(k.up) {
            delta.y += DELTA;
        }

        if keyboard_input.pressed(k.down) {
            delta.y -= DELTA;
        }

        if keyboard_input.just_pressed(k.bomb) {
            p.bomb_set = true;
        }

        delta.norm();
    }
}

/// called at fixed time ticks
/// update the transforms
/// reset deltas
/// maybe spawn bombs
fn update_player_position(mut query: Query<(With<Player>, &mut Deltas, &mut Transform)>) {
    for (_, mut delta, mut trans) in query.iter_mut() {
        trans.translation.x += delta.x;
        trans.translation.y += delta.y;

        trans.translation.x = trans.translation.x.clamp(
            (-WINDOW_WIDTH + PLAYER_SIZE) / 2.,
            (WINDOW_WIDTH - PLAYER_SIZE) / 2.,
        );

        trans.translation.y = trans.translation.y.clamp(
            (-WINDOW_HEIGHT + PLAYER_SIZE) / 2.,
            (WINDOW_HEIGHT - PLAYER_SIZE) / 2.,
        );
        delta.reset()
    }
}

fn spawn_bombs(mut query: Query<(&mut Player, &Transform)>, mut commands: Commands) {
    for (mut p, trans) in query.iter_mut() {
        if p.bomb_set {
            commands
                .spawn()
                .insert_bundle(SpriteBundle {
                    transform: Transform::from_translation(Vec3::new(
                        align_on_grid(trans.translation.x),
                        align_on_grid(trans.translation.y),
                        0.0,
                    )),
                    sprite: Sprite {
                        color: Color::rgb(0., 0., 0.),
                        custom_size: Some(Vec2::new(PLAYER_SIZE, PLAYER_SIZE)),
                        ..Default::default()
                    },
                    ..Default::default()
                })
                .insert(Bomb {
                    explosion_range: p.bomb_range,
                    countdown: Timer::from_seconds(BOMB_COUNTDOWN, false),
                })
                .insert(NotTraversable)
                .insert(Size(CELL_SIZE));
            p.bomb_set = false;
        }
    }
}

/// called before update_player_position
/// clamp deltas
/// maybe call at fixed time ticks
/// check collision with wall and breakable and bomb
fn check_collision(
    mut players: Query<(&mut Deltas, &Transform, With<Player>, &Size)>,
    obstacles: Query<(&Transform, With<NotTraversable>, &Size)>,
) {
    for (mut d, tp, _, ps) in players.iter_mut() {
        for (to, _, os) in obstacles.iter() {
            if let Some(collision) = collide(
                tp.translation,
                Vec2::new(ps.0, ps.0),
                to.translation,
                Vec2::new(os.0, os.0),
            ) {
                match collision {
                    Collision::Left => d.x = d.x.min(0.),
                    Collision::Right => d.x = d.x.max(0.),
                    Collision::Top => d.y = d.y.max(0.),
                    Collision::Bottom => d.y = d.y.min(0.),
                }
            }
        }
    }
}

/// check if explosion happened
/// called at fixed timestamp
/// if it exploded, spawn explosion
/// despawn bomb
fn explode(
    mut query: Query<(Entity, &mut Bomb, &Transform)>,
    // obstacles: Query<(&Wall, &Transform)>,
    mut commands: Commands,
) {
    let spawn = |x, y, com: &mut Commands| {
        com.spawn()
            .insert_bundle(SpriteBundle {
                transform: Transform::from_translation(Vec3::new(x, y, 0.0)),
                sprite: Sprite {
                    color: Color::rgb(1., 0., 0.),
                    custom_size: Some(Vec2::new(CELL_SIZE, CELL_SIZE)),
                    ..Default::default()
                },
                ..Default::default()
            })
            .insert(Explosion(Timer::from_seconds(BOMB_COUNTDOWN, false)))
            .insert(Size(CELL_SIZE));
    };
    for (e, mut b, trans) in query.iter_mut() {
        b.countdown.tick(Duration::from_millis(
            (FRAME_DURATION * 1000.).floor() as u64
        ));
        if b.countdown.finished() {
            spawn(
                align_on_grid(trans.translation.x),
                align_on_grid(trans.translation.y),
                &mut commands,
            );
            for i in 1..=b.explosion_range {
                spawn(
                    align_on_grid(trans.translation.x + i as f32 * CELL_SIZE),
                    align_on_grid(trans.translation.y),
                    &mut commands,
                );
                spawn(
                    align_on_grid(trans.translation.x),
                    align_on_grid(trans.translation.y - i as f32 * CELL_SIZE),
                    &mut commands,
                );
                spawn(
                    align_on_grid(trans.translation.x - i as f32 * CELL_SIZE),
                    align_on_grid(trans.translation.y),
                    &mut commands,
                );
                spawn(
                    align_on_grid(trans.translation.x),
                    align_on_grid(trans.translation.y + i as f32 * CELL_SIZE),
                    &mut commands,
                );
            }
            commands.entity(e).despawn();
        }
    }
}

/// called at fixed timestamp
/// check if explosion has elapsed
/// despawn explosion
fn put_down_explosion(mut query: Query<(Entity, &mut Explosion)>, mut commands: Commands) {
    for (et, mut ex) in query.iter_mut() {
        ex.0.tick(Duration::from_millis(
            (FRAME_DURATION * 1000.).floor() as u64
        ));
        if ex.0.finished() {
            commands.entity(et).despawn();
        }
    }
}

/// check collision with explosion
fn check_death(
    mut players: Query<(Entity, &Transform, With<Player>, &Size)>,
    mut commands: Commands,
    obstacles: Query<(&Transform, With<Explosion>, &Size)>,
) {
    for (e, tp, _, ps) in players.iter_mut() {
        for (to, _, os) in obstacles.iter() {
            if let Some(_) = collide(
                tp.translation,
                Vec2::new(ps.0, ps.0),
                to.translation,
                Vec2::new(os.0, os.0),
            ) {
                commands.entity(e).despawn();
            }
        }
    }
}

fn check_game_over(mut query: Query<(Entity, With<Player>)>) {
    if query.iter().next().is_none() {
        exit(1)
    }
}
