use bevy::prelude::*;
use bevy_physimple::prelude::*;
use bevy_inspector_egui::Inspectable;

pub struct PlayerPlugin;

#[derive(Component, Inspectable)]
pub struct PlayerMovement {
    on_wall: Option<Vec2>,
    on_floor: bool,
    speed: f32,
    looking_right: bool,
    jump_height: f32,
    acceleration: f32,
    deceleration: f32,
    air_control: f32,
    min_jump_height: f32,
    jump_time_to_peak: f32,
    jump_time_to_descent: f32,
    jump_velocity: f32,
    min_jump_velocity: f32,
    jump_gravity: f32,
    fall_gravity: f32,
}

impl Default for PlayerMovement {
    fn default() -> Self {
        PlayerMovement {
            on_wall: None,
            on_floor: false,
            speed: 350.0,
            looking_right: true,
            jump_height: 120.0,
            acceleration: 10.0,
            deceleration: 12.0,
            air_control: 0.9,
            min_jump_height: 60.0,
            jump_time_to_peak: 0.4,
            jump_time_to_descent: 0.4,
            jump_velocity: 0.0,
            min_jump_velocity: 0.0,
            jump_gravity: 0.0,
            fall_gravity: 0.0,
        }
    }
}

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app
        .add_startup_system(setup)
        .add_system(controller_on_stuff)
        .add_system(process);
    }
}

// TODO: coyote time, jump buffer, player with animations, organize the code a bit better

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let player_size = Vec2::new(54.0, 54.0);

    // Math explained in https://www.youtube.com/watch?v=IOe1aGY6hXA
    let mut pm = PlayerMovement::default();
    pm.jump_velocity = (2.0 * pm.jump_height) / pm.jump_time_to_peak;
    pm.min_jump_velocity = (2.0 * pm.min_jump_height) / pm.jump_time_to_peak;
    pm.jump_gravity = (-2.0 * pm.jump_height) / pm.jump_time_to_peak.powi(2);
    pm.fall_gravity = (-2.0 * pm.jump_height) / pm.jump_time_to_descent.powi(2);

    commands.spawn_bundle(SpriteBundle {
        texture: asset_server.load("bevy.png"),
        sprite: Sprite {
            custom_size: Some(player_size),
            ..Default::default()
        },
        ..Default::default()
    })
    .insert(pm)
    .insert(Name::new("Player"))
    .insert_bundle(KinematicBundle {
        shape: CollisionShape::Square(Square::size(player_size)),
        ..Default::default()
    });
}

fn process(
    input: Res<Input<KeyCode>>, 
    time: Res<Time>, 
    mut query: Query<(&mut PlayerMovement, &mut Vel, &mut Sprite)>
) {
    let mut move_axis = Vec2::new(0.0, 0.0);
    let delta = time.delta_seconds();

    for (mut player, mut velocity, mut sprite) in query.iter_mut() {
        velocity.0.y -= -get_gravity(velocity.as_ref(), player.as_ref()) * delta;

        controller_input(input.as_ref(), &mut move_axis, velocity.as_mut(), player.as_ref());

        accelerate(&move_axis, velocity.as_mut(), player.as_ref(), &delta);

        look_at(&move_axis, player.as_mut(), sprite.as_mut());
    }
}

fn get_gravity(velocity: &Vel, player: &PlayerMovement) -> f32 {
    if velocity.0.x > 0.0 {
        return player.jump_gravity;
    }
    else
    {
        return player.fall_gravity;
    }
}

fn controller_input(
    input: &Input<KeyCode>, 
    move_axis: &mut Vec2, 
    velocity: &mut Vel, 
    player: &PlayerMovement
) {
    if input.pressed(KeyCode::W) {
        move_axis.y += 1.0;
    }
    if input.pressed(KeyCode::S) {
        move_axis.y -= 1.0;
    }
    if input.pressed(KeyCode::D) {
        move_axis.x += 1.0;
    }
    if input.pressed(KeyCode::A) {
        move_axis.x -= 1.0;
    }

    if input.just_pressed(KeyCode::Space) {
        if player.on_floor {
            velocity.0.y = player.jump_velocity;
        }
    }

    if input.just_released(KeyCode::Space) && velocity.0.y > player.min_jump_velocity {
        velocity.0.y = player.min_jump_velocity;
    }
}

fn accelerate(move_axis: &Vec2, velocity: &mut Vel, player: &PlayerMovement, delta: &f32) {
    let mut temp_vel = velocity.0;
    let mut temp_accel: f32;
    let target = Vec2::new(move_axis.x * player.speed, 0.0);

    temp_vel.y = 0.0;
    if move_axis.x != 0.0 {
        temp_accel = player.acceleration;
    }
    else {
        temp_accel = player.deceleration;
    }

    if !player.on_floor {
        temp_accel *= player.air_control;
    }

    temp_vel = temp_vel.lerp(target, temp_accel * delta);

    velocity.0.x = temp_vel.x;

    if move_axis.x == 0.0 {
        let vel_clamp = 0.01;

        if velocity.0.x.abs() < vel_clamp {
            velocity.0.x = 0.0;
        }
    }
}

fn look_at(move_axis: &Vec2, player: &mut PlayerMovement, sprite: &mut Sprite) {
    if move_axis.x >= 0.5 {
        player.looking_right = true;
    }
    if move_axis.x <= -0.5 {
        player.looking_right = false;
    }

    if player.looking_right {
        sprite.flip_x = false;
    }
    else {
        sprite.flip_x = true;
    }
}

// https://github.com/RustyStriker/bevy_physimple/blob/3ba638c99e1a693d99548f0a71bee9e6297de326/examples/platformer.rs#L223
fn controller_on_stuff(
    mut query: Query<(Entity, &mut PlayerMovement)>,
    mut colls: EventReader<CollisionEvent>,
) {
    // Iterate over the collisions and check if the player is on a wall/floor
    let (e, mut c) = query.single_mut();

    // clear the current data on c
    c.on_floor = false;
    c.on_wall = None;

    for coll in colls.iter().filter(|&c| c.is_b_static) {
        if coll.entity_a == e {
            let n = coll.normal.dot(Vec2::Y);

            if n > 0.7 {
                c.on_floor = true;
            }
            else if n.abs() <= 0.7 {
                c.on_wall = Some(coll.normal);
            }
        }
    }
}
