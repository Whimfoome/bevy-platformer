use bevy::prelude::*;
use bevy_physimple::prelude::*;
use player::{CoyoteTimer, JumpBuffer};

mod player;
mod helpers;
mod debug;

const CLEAR: Color = Color::rgb(0.1, 0.1, 0.1);

fn main() {
    let mut app = App::new();
    
    app // Setup
        .insert_resource(ClearColor(CLEAR))
        .insert_resource(create_window())
        .add_plugins(DefaultPlugins)
        .add_plugin(Physics2dPlugin)
        .add_plugin(debug::DebugPlugin);

    app // Game
        .add_startup_system(spawn)
        .add_plugin(player::PlayerPlugin)
        .add_system(tick_timers)
        .run();
}

fn create_window() -> WindowDescriptor {
    WindowDescriptor {
        title: "Bevy Platformer".to_owned(),
        vsync: true,
        ..Default::default()
    }
}

fn spawn(mut commands: Commands) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());

    // Spawn Floor
    let floor_size = Vec2::new(600.0, 30.0);
    commands
    .spawn_bundle(SpriteBundle {
        sprite: Sprite {
            custom_size: Some(floor_size),
            color: Color::WHITE,
            ..Default::default()
        },
        transform: Transform::from_xyz(0.0, -200.0, 0.0),
        ..Default::default()
    })
    .insert_bundle(StaticBundle {
        shape: CollisionShape::Square(Square::size(floor_size)),
        ..Default::default()
    });
}

fn tick_timers(
    time: Res<Time>,
    mut query: Query<(&mut CoyoteTimer, &mut JumpBuffer)>
) {
    for (mut coyote, mut jump_buffer) in query.iter_mut() {
        coyote.0.tick(time.delta());
        jump_buffer.0.tick(time.delta());
    }
}
