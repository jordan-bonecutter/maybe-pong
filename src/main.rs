use bevy::{
    prelude::*,
    sprite::{collide_aabb::{collide, Collision}, MaterialMesh2dBundle},
    time::FixedTimestep,
};

use std::cmp::Ord;
use std::ops::MulAssign;

fn main() {
    let cfg = Config{
        physics_frames: 60.0,
        arena_size: Vec3::new(900.0, 700.0, 0.0),
        paddle_size: Vec3::new(5.0, 100.0, 0.0),
        paddle_padding: 100.0,
        paddle_color: Color::rgb(1.0, 1.0, 1.0),
        ball_color: Color::rgb(1.0, 0.0, 0.0),
        ball_size: Vec3::new(20.0, 20.0, 0.0),
        inital_ball_speed: Vec3::new(-10.0, 0.0, 0.0),
        ai_paddle_speed: 5.0,
        angle_aggressiveness: 2.0,
        wall_thickness: 5.0,
    };

    let timestep = FixedTimestep::step((1.0 / cfg.physics_frames) as f64);

    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(cfg)
        .add_startup_system(setup)
        .add_event::<CollisionEvent>()
        .add_system_set(
            SystemSet::new()
                .with_run_criteria(timestep)
                .with_system(move_left_paddle)
                .with_system(move_ball.before(bounce_walls))
                .with_system(move_ai_paddle.before(move_ball))
                .with_system(bounce_walls.before(move_left_paddle))
        )
        .add_system(bevy::window::close_on_esc)
        .run();
}

#[derive(Resource)]
struct Config {
    physics_frames: f64,
    arena_size: Vec3,
    paddle_size: Vec3,
    paddle_padding: f32,
    paddle_color: Color,
    ball_color: Color,
    ball_size: Vec3,
    inital_ball_speed: Vec3,
    ai_paddle_speed: f32,
    angle_aggressiveness: f32,
    wall_thickness: f32,
}

impl Config {
    fn left_paddle_starting_position(&self) -> Vec3 {
        Vec3::new(
            self.paddle_padding - (self.arena_size.x / 2.0) + (self.paddle_size.x / 2.0),
            0.0,
            1.0,
        )
    }

    fn right_paddle_starting_position(&self) -> Vec3 {
        Vec3::new(
            (self.arena_size.x / 2.0) - self.paddle_padding - (self.paddle_size.x / 2.0),
            0.0,
            1.0,
        )
    }

    fn top_wall_position(&self) -> Vec3 {
        Vec3::new(
            0.0,
            (self.arena_size.y / 2.0) + (self.wall_thickness / 2.0),
            1.0,
        )
    }

    fn top_wall_scale(&self) -> Vec3 {
        Vec3::new(
            self.arena_size.x + (self.wall_thickness * 2.0),
            self.wall_thickness,
            1.0,
        )
    }

    fn bottom_wall_position(&self) -> Vec3 {
        Vec3::new(
            0.0,
            (-self.arena_size.y / 2.0) - (self.wall_thickness / 2.0),
            1.0,
        )
    }

    fn bottom_wall_scale(&self) -> Vec3 {
        Vec3::new(
            self.arena_size.x + (self.wall_thickness * 2.0),
            self.wall_thickness,
            1.0,
        )
    }

    fn left_wall_position(&self) -> Vec3 {
        Vec3::new(
            (-self.arena_size.x / 2.0) - (self.wall_thickness / 2.0),
            0.0,
            1.0,
        )
    }

    fn left_wall_scale(&self) -> Vec3 {
        Vec3::new(
            self.wall_thickness,
            self.arena_size.y + (2.0 * self.wall_thickness),
            1.0,
        )
    }

    fn right_wall_position(&self) -> Vec3 {
        Vec3::new(
            (self.arena_size.x / 2.0) + (self.wall_thickness / 2.0),
            0.0,
            1.0,
        )
    }

    fn right_wall_scale(&self) -> Vec3 {
        Vec3::new(
            self.wall_thickness,
            self.arena_size.y + (2.0 * self.wall_thickness),
            1.0,
        )
    }

    fn ball_starting_position() -> Vec3 {
        Vec3::new(0.0, 0.0, 1.0)
    }
}

#[derive(Component)]
struct LeftPaddle;

#[derive(Component)]
struct RightPaddle;

#[derive(Component)]
struct Paddle;

#[derive(Component)]
struct Ball;

#[derive(Component)]
struct Wall(Vec3);

#[derive(Component, Deref, DerefMut, Debug)]
struct Velocity(Vec3);

impl MulAssign<f32> for Velocity {
    fn mul_assign(&mut self, rhs: f32) {
        self.0 *= rhs;
    }
}

#[derive(Default)]
struct CollisionEvent;

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    cfg: Res<Config>,
) {

    commands.spawn(Camera2dBundle::default());

    // Spawn left paddle
    commands.spawn((
       SpriteBundle {
            transform: Transform{
                translation: cfg.left_paddle_starting_position(),
                scale: cfg.paddle_size,
                ..default()
            },
            sprite: Sprite{
                color: cfg.paddle_color,
                ..default()
            },
            ..default()
       },
       Paddle,
       LeftPaddle,
    ));

    // Spawn right paddle
    commands.spawn((
       SpriteBundle {
            transform: Transform{
                translation: cfg.right_paddle_starting_position(),
                scale: cfg.paddle_size,
                ..default()
            },
            sprite: Sprite{
                color: cfg.paddle_color,
                ..default()
            },
            ..default()
       },
       Paddle,
       RightPaddle,
    ));

    // Spawn ball
    commands.spawn((
        MaterialMesh2dBundle{
            mesh: meshes.add(shape::Circle::default().into()).into(),
            material: materials.add(ColorMaterial::from(cfg.ball_color)),
            transform: Transform::from_translation(Config::ball_starting_position()).with_scale(cfg.ball_size),
            ..default()
        },
        Ball,
        Velocity(cfg.inital_ball_speed),
    ));

    // Spawn top wall
    commands.spawn((
       SpriteBundle {
            transform: Transform{
                translation: cfg.top_wall_position(),
                scale: cfg.top_wall_scale(),
                ..default()
            },
            sprite: Sprite{
                color: cfg.paddle_color,
                ..default()
            },
            ..default()
       },
       Wall(Vec3::new(1.0, -1.0, 1.0)),
    ));

    // Spawn top wall
    commands.spawn((
       SpriteBundle {
            transform: Transform{
                translation: cfg.bottom_wall_position(),
                scale: cfg.bottom_wall_scale(),
                ..default()
            },
            sprite: Sprite{
                color: cfg.paddle_color,
                ..default()
            },
            ..default()
       },
       Wall(Vec3::new(1.0, -1.0, 1.0)),
    ));

    // Spawn right wall
    commands.spawn((
       SpriteBundle {
            transform: Transform{
                translation: cfg.right_wall_position(),
                scale: cfg.right_wall_scale(),
                ..default()
            },
            sprite: Sprite{
                color: cfg.paddle_color,
                ..default()
            },
            ..default()
       },
       Wall(Vec3::new(-1.0, 1.0, 1.0)),
    ));

    // Spawn right wall
    commands.spawn((
       SpriteBundle {
            transform: Transform{
                translation: cfg.left_wall_position(),
                scale: cfg.left_wall_scale(),
                ..default()
            },
            sprite: Sprite{
                color: cfg.paddle_color,
                ..default()
            },
            ..default()
       },
       Wall(Vec3::new(-1.0, 1.0, 1.0)),
    ));
}

fn move_left_paddle(
    keyboard: Res<Input<KeyCode>>,
    mut query: Query<&mut Transform, With<LeftPaddle>>,
    cfg: Res<Config>,
) {
    let mut paddle_transform = query.single_mut();
    let mut velocity = Vec3::new(0.0, 0.0, 0.0);

    if keyboard.pressed(KeyCode::K) {
        velocity.y += 1.0;
    }

    if keyboard.pressed(KeyCode::J) {
        velocity.y -= 1.0;
    }

    paddle_transform.translation += velocity * 10.0;
    paddle_transform.translation.y = paddle_transform.translation.y.clamp(
        (-cfg.arena_size.y / 2.0) + (cfg.paddle_size.y / 2.0),
        cfg.arena_size.y / 2.0 - (cfg.paddle_size.y / 2.0),
    );
}

fn overlaps(a0: f32, a1: f32, b0: f32, b1: f32) -> bool {
    contains(a0, a1, b0) || contains(a0, a1, b1) || contains(b0, b1, a0) || contains(b0, b1, a0)
}

fn contains(lo: f32, hi: f32, x: f32) -> bool {
    lo <= x && hi >= x
}

fn bounds(transform: &Transform) -> (f32, f32, f32, f32) {
    let (width, height) = (transform.scale.x, transform.scale.y);
    let (xcenter, ycenter) = (transform.translation.x, transform.translation.y);
    let (x0, x1) = (xcenter - (width / 2.0), xcenter + (width / 2.0));
    let (y0, y1) = (ycenter - (height / 2.0), ycenter + (height / 2.0));
    (x0, x1, y0, y1)
}

fn move_ball(
    mut ball: Query<(&mut Transform, &mut Velocity), With<Ball>>,
    paddles: Query<&Transform, (With<Paddle>, Without<Ball>)>,
    cfg: Res<Config>,
) {
    // First we move the ball
    let (mut ball_transform, mut ball_velocity) = ball.single_mut();
    let old_ball_position = ball_transform.translation;
    ball_transform.translation.x += ball_velocity.x;
    ball_transform.translation.y += ball_velocity.y;

    // Let's check if the ball is hitting a paddle
    paddles.iter().for_each(|paddle| {
        let paddle_bounds = bounds(paddle);
        let ball_bounds = bounds(&ball_transform);

        if !overlaps(paddle_bounds.0, paddle_bounds.1, ball_bounds.0, ball_bounds.1) {
            return;
        }

        if !overlaps(paddle_bounds.2, paddle_bounds.3, ball_bounds.2, ball_bounds.3) {
            return;
        }
        
        // Where on the paddle did it hit?
        let speed = ball_velocity.length();
        let delta_y = (paddle.translation.y - ball_transform.translation.y) / ((paddle.scale.y / cfg.angle_aggressiveness));
        ball_velocity.y = ball_velocity.x * delta_y;
        ball_velocity.x = -ball_velocity.x;
        *ball_velocity = Velocity(ball_velocity.normalize());
        *ball_velocity *= speed;
    })
}

fn move_ai_paddle(
    ball: Query<&Transform, With<Ball>>,
    mut query: Query<&mut Transform, (With<RightPaddle>, Without<Ball>)>,
    cfg: Res<Config>,
) {
    let ball = ball.single(); 
    let mut ai_paddle = query.single_mut();

    let delta_y = ball.translation.y - ai_paddle.translation.y;
    if delta_y >= -cfg.ai_paddle_speed && delta_y <= cfg.ai_paddle_speed {
        ai_paddle.translation.y += delta_y;
    } else if delta_y < 0.0 {
        ai_paddle.translation.y -= cfg.ai_paddle_speed;
    } else if delta_y > 0.0 {
        ai_paddle.translation.y += cfg.ai_paddle_speed;
    }

    ai_paddle.translation.y = ai_paddle.translation.y.clamp(
        (-cfg.arena_size.y / 2.0) + (cfg.paddle_size.y / 2.0),
        cfg.arena_size.y / 2.0 - (cfg.paddle_size.y / 2.0),
    );
}

fn bounce_walls(
    mut ball: Query<(&mut Transform, &mut Velocity), (With<Ball>, Without<Wall>)>,
    walls: Query<(&Transform, &Wall)>,
    cfg: Res<Config>,
) {
    let (ball_tform, mut ball_vel) = ball.single_mut();

    walls.iter().for_each(|wall| {
        let (wall_tform, wall) = wall;
        let wall_bounds = bounds(wall_tform);
        let ball_bounds = bounds(&ball_tform);

        if !overlaps(wall_bounds.0, wall_bounds.1, ball_bounds.0, ball_bounds.1) {
            return;
        }

        if !overlaps(wall_bounds.2, wall_bounds.3, ball_bounds.2, ball_bounds.3) {
            return;
        }
        
        ball_vel.x *= wall.0.x;
        ball_vel.y *= wall.0.y;
    })
}
