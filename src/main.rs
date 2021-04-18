use bevy::core::FixedTimestep;
use bevy::prelude::*;
use bevy::render::pass::ClearColor;

const TIME_STEP: f32 = 1.0 / 8.0;

const ARENA_WIDTH: i16 = 10;
const ARENA_HEIGHT: i16 = 24;
const SCALE: f32 = 30.0;

fn main() {
    App::build()
        .insert_resource(WindowDescriptor {
            title: "Rustic Tetris!".to_string(),
            width: SCALE * ARENA_WIDTH as f32,
            height: SCALE * ARENA_HEIGHT as f32,
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .insert_resource(ClearColor(Color::rgb(0.3, 0.3, 0.3)))
        .add_startup_system(setup.system())
        .add_system_set(
            SystemSet::new()
                .with_run_criteria(FixedTimestep::step(TIME_STEP as f64))
                .with_system(player_input.system().label(TetriminoMovement::Input))
                .with_system(
                    player_movement
                        .system()
                        .label(TetriminoMovement::Movement)
                        .after(TetriminoMovement::Input),
                ),
        )
        .add_system_set_to_stage(
            CoreStage::PostUpdate,
            SystemSet::new()
                .with_system(position_translation.system())
                .with_system(size_scaling.system()),
        )
        .run();
}

#[derive(SystemLabel, Debug, Hash, PartialEq, Eq, Clone)]
pub enum TetriminoMovement {
    Input,
    Movement,
}

#[derive(Debug)]
struct Position {
    x: i16,
    y: i16,
}

#[derive(Debug)]
enum Action {
    ShiftRight,
    ShiftLeft,
    RotateClockwise,
    RotateCounterClockwise,
    SoftDrop,
    HardDrop,
    Hold,
    None,
}

#[derive(Debug)]
struct Size {
    width: f32,
    height: f32,
}

impl Size {
    pub fn square(x: f32) -> Self {
        Self {
            width: x,
            height: x,
        }
    }
}

#[derive(Debug)]
struct Arena {
    width: i16,
    height: i16,
}

#[derive(Debug)]
struct Tetrimino {
    action: Action,
}

fn setup(mut commands: Commands, mut materials: ResMut<Assets<ColorMaterial>>) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
    commands.spawn_bundle(UiCameraBundle::default());

    // spawn test player brick
    commands
        .spawn_bundle(SpriteBundle {
            material: materials.add(Color::rgb(0.5, 0.5, 1.0).into()),
            sprite: Sprite::new(Vec2::new(30.0, 30.0)),
            ..Default::default()
        })
        .insert(Tetrimino {
            action: Action::None,
        })
        .insert(Position { x: 5, y: 10 })
        .insert(Size::square(0.8))
        .insert(Action::None);

    commands.insert_resource(Arena {
        width: ARENA_WIDTH,
        height: ARENA_HEIGHT,
    });
}

fn player_input(keyboard_input: Res<Input<KeyCode>>, mut teterminos: Query<&mut Tetrimino>) {
    for pressed in keyboard_input.get_pressed() {
        eprintln!("{:?}", pressed);
    }
    if let Some(mut tetermino) = teterminos.iter_mut().next() {
        let action = if keyboard_input.pressed(KeyCode::Up)
            || keyboard_input.pressed(KeyCode::X)
            || keyboard_input.pressed(KeyCode::Numpad1)
            || keyboard_input.pressed(KeyCode::Numpad5)
            || keyboard_input.pressed(KeyCode::Numpad9)
        {
            Action::RotateClockwise
        } else if keyboard_input.pressed(KeyCode::LControl)
            || keyboard_input.pressed(KeyCode::Z)
            || keyboard_input.pressed(KeyCode::Numpad3)
            || keyboard_input.pressed(KeyCode::Numpad7)
        {
            Action::RotateCounterClockwise
        } else if keyboard_input.pressed(KeyCode::Space) || keyboard_input.pressed(KeyCode::Numpad8)
        {
            Action::HardDrop
        } else if keyboard_input.pressed(KeyCode::LShift)
            || keyboard_input.pressed(KeyCode::C)
            || keyboard_input.pressed(KeyCode::Numpad0)
        {
            Action::Hold
        } else if keyboard_input.pressed(KeyCode::Left) || keyboard_input.pressed(KeyCode::Numpad4)
        {
            Action::ShiftLeft
        } else if keyboard_input.pressed(KeyCode::Right) || keyboard_input.pressed(KeyCode::Numpad6)
        {
            Action::ShiftRight
        } else if keyboard_input.pressed(KeyCode::Down) || keyboard_input.pressed(KeyCode::Numpad2)
        {
            Action::SoftDrop
        } else {
            Action::None
        };
        tetermino.action = action;
        // eprintln!("{:?}", tetermino);
    }
}

fn player_movement(mut teterminos: Query<(&mut Tetrimino, &mut Position)>) {
    for (mut tetermino, mut position) in teterminos.iter_mut() {
        match tetermino.action {
            Action::ShiftRight => {
                position.x += 1;
            }
            Action::ShiftLeft => {
                position.x -= 1;
            }
            Action::RotateClockwise => {}
            Action::RotateCounterClockwise => {}
            Action::SoftDrop => {
                position.y -= 1;
            }
            Action::HardDrop => {
                // send event hard drop
            }
            Action::Hold => {
                position.y += 1;
            }
            Action::None => {}
        };
        tetermino.action = Action::None;
        position.x = position.x.max(0).min(ARENA_WIDTH as i16);
        position.y = position.y.max(0).min(ARENA_HEIGHT as i16);
    }
}

fn size_scaling(windows: Res<Windows>, mut q: Query<(&Size, &mut Sprite)>, arena: Res<Arena>) {
    let window = windows.get_primary().unwrap();
    for (sprite_size, mut sprite) in q.iter_mut() {
        sprite.size = Vec2::new(
            sprite_size.width / arena.width as f32 * window.width() as f32,
            sprite_size.height / arena.height as f32 * window.height() as f32,
        );
    }
}

fn position_translation(
    windows: Res<Windows>,
    mut q: Query<(&Position, &mut Transform)>,
    arena: Res<Arena>,
) {
    fn convert(pos: f32, bound_window: f32, bound_game: f32) -> f32 {
        let tile_size = bound_window / bound_game;
        pos / bound_game * bound_window - (bound_window / 2.) + (tile_size / 2.)
    }
    let window = windows.get_primary().unwrap();
    for (pos, mut transform) in q.iter_mut() {
        transform.translation = Vec3::new(
            convert(pos.x as f32, window.width() as f32, arena.width as f32),
            convert(pos.y as f32, window.height() as f32, arena.height as f32),
            0.0,
        );
    }
}
