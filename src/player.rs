use bevy::prelude::*;
use bevy_physimple::prelude::*;
use bevy_inspector_egui::Inspectable;
use crate::helpers::TimerHelper;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app
        .add_startup_system(setup)
        .add_system(controller_on_stuff.label("stuff"))
        .add_system(was_on_floor.before("stuff"))
        .add_system(controller_input.after("stuff"))
        .add_system(gravity)
        .add_system(accelerate)
        .add_system(look_at);
    }
}

#[derive(Component, Inspectable)]
pub struct PlayerMovement {
    speed: f32,
    #[inspectable(read_only)]
    on_wall: Option<Vec2>,
    #[inspectable(read_only)]
    on_floor: bool,
    #[inspectable(ignore)]
    was_on_floor: bool,
    #[inspectable(read_only)]
    move_axis: Vec2,
    #[inspectable(read_only)]
    looking_right: bool,
    acceleration: f32,
    deceleration: f32,
    #[inspectable(min = 0.0, max = 1.0)]
    air_control: f32,
    #[inspectable(read_only)]
    is_jumping: bool,
    #[inspectable(read_only)]
    jump_height: f32,
    #[inspectable(read_only)]
    min_jump_height: f32,
    #[inspectable(read_only)]
    jump_time_to_peak: f32,
    #[inspectable(read_only)]
    jump_time_to_descent: f32,
    #[inspectable(ignore)]
    jump_velocity: f32,
    #[inspectable(ignore)]
    min_jump_velocity: f32,
    #[inspectable(ignore)]
    jump_gravity: f32,
    #[inspectable(ignore)]
    fall_gravity: f32,
}

impl Default for PlayerMovement {
    fn default() -> Self {
        PlayerMovement {
            on_wall: None,
            on_floor: false,
            was_on_floor: false,
            move_axis: Vec2::new(0.0, 0.0),
            speed: 350.0,
            looking_right: true,
            jump_height: 120.0,
            acceleration: 10.0,
            deceleration: 12.0,
            air_control: 0.9,
            is_jumping: false,
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

#[derive(Bundle)]
struct PlayerBundle {
    name: Name,
    #[bundle]
    sprite_bundle: SpriteBundle,
    player_movement: PlayerMovement,
    #[bundle]
    physics: KinematicBundle,
    coyote_timer: CoyoteTimer,
    jump_buffer: JumpBuffer,
}

impl Default for PlayerBundle {
    fn default() -> Self {
        let player_size = Vec2::new(54.0, 54.0);

        // Math explained in https://www.youtube.com/watch?v=IOe1aGY6hXA
        let mut pm = PlayerMovement::default();
        pm.jump_velocity = (2.0 * pm.jump_height) / pm.jump_time_to_peak;
        pm.min_jump_velocity = (2.0 * pm.min_jump_height) / pm.jump_time_to_peak;
        pm.jump_gravity = (-2.0 * pm.jump_height) / pm.jump_time_to_peak.powi(2);
        pm.fall_gravity = (-2.0 * pm.jump_height) / pm.jump_time_to_descent.powi(2);

        let mut coyote_timer = Timer::from_seconds(0.1, false);
        coyote_timer.pause();

        let mut jump_buffer = Timer::from_seconds(0.1, false);
        jump_buffer.pause();

        PlayerBundle {
            name: Name::new("Player"),
            sprite_bundle: SpriteBundle {
                sprite: Sprite {
                    custom_size: Some(player_size),
                    ..Default::default()
                },
                ..Default::default()
            },
            player_movement: pm,
            physics: KinematicBundle {
                shape: CollisionShape::Square(Square::size(player_size)),
                ..Default::default()
            },
            coyote_timer: CoyoteTimer(coyote_timer),
            jump_buffer: JumpBuffer(jump_buffer),
        }
    }
}

#[derive(Component)]
pub struct CoyoteTimer(pub Timer);

#[derive(Component)]
pub struct JumpBuffer(pub Timer);

// TODO: player with animations, organize the code a bit better

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let mut player_bundle = PlayerBundle::default();
    player_bundle.sprite_bundle.texture = asset_server.load("bevy.png");

    commands.spawn_bundle(player_bundle);
}

fn gravity( 
    time: Res<Time>, 
    mut query: Query<(&PlayerMovement, &mut Vel)>
) {
    let delta = time.delta_seconds();
    for (player, mut velocity) in query.iter_mut() {
        if velocity.0.y > 0.0 {
            velocity.0.y -= -player.jump_gravity * delta;
        }
        else
        {
            velocity.0.y -= -player.fall_gravity * delta;
        }
    }
}

fn was_on_floor(
    mut query: Query<&mut PlayerMovement>
) {
    for mut player in query.iter_mut() {
        player.was_on_floor = player.on_floor;
    }
}

fn controller_input(
    input: Res<Input<KeyCode>>, 
    mut query: Query<(&mut PlayerMovement, &mut Vel, &mut CoyoteTimer, &mut JumpBuffer)>
) {
    for (mut player, mut velocity, mut coyote, mut jump_buffer) in query.iter_mut() {
        player.move_axis = Vec2::new(0.0, 0.0);

        if input.pressed(KeyCode::W) {
            player.move_axis.y += 1.0;
        }
        if input.pressed(KeyCode::S) {
            player.move_axis.y -= 1.0;
        }
        if input.pressed(KeyCode::D) {
            player.move_axis.x += 1.0;
        }
        if input.pressed(KeyCode::A) {
            player.move_axis.x -= 1.0;
        }

        if player.on_floor && !jump_buffer.0.is_stopped() {
            jump_buffer.0.pause();

            velocity.0.y = player.jump_velocity;
            player.is_jumping = true;
        }
    
        if input.just_pressed(KeyCode::Space) {
            if player.on_floor || !coyote.0.is_stopped() {
                coyote.0.pause();

                velocity.0.y = player.jump_velocity;
                player.is_jumping = true;
            }
            else {
                jump_buffer.0.start();
            }
        }

        if player.is_jumping && velocity.0.y <= 0.0 {
            player.is_jumping = false;
        }
    
        if input.just_released(KeyCode::Space) && velocity.0.y > player.min_jump_velocity {
            velocity.0.y = player.min_jump_velocity;
        }

        if !player.on_floor && player.was_on_floor && !player.is_jumping {
            coyote.0.start();
        }
    }
}

fn accelerate(
    time: Res<Time>, 
    mut query: Query<(&PlayerMovement, &mut Vel)>
) {
    let delta = time.delta_seconds();

    for (player, mut velocity) in query.iter_mut() {
        let mut temp_vel = velocity.0;
        let mut temp_accel: f32;
        let target = Vec2::new(player.move_axis.x * player.speed, 0.0);
    
        temp_vel.y = 0.0;
        if player.move_axis.x != 0.0 {
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
    
        if player.move_axis.x == 0.0 {
            let vel_clamp = 0.01;
    
            if velocity.0.x.abs() < vel_clamp {
                velocity.0.x = 0.0;
            }
        }
    }
}

fn look_at(
    mut query: Query<(&mut PlayerMovement, &mut Sprite)>
) {
    for (mut player, mut sprite) in query.iter_mut() {
        if player.move_axis.x >= 0.5 {
            player.looking_right = true;
        }
        else if player.move_axis.x <= -0.5 {
            player.looking_right = false;
        }
    
        if player.looking_right {
            sprite.flip_x = false;
        }
        else {
            sprite.flip_x = true;
        }
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
