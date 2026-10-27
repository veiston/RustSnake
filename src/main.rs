use bevy::app::AppExit;
use bevy::prelude::*;

const MULTIPLIER: f32 = 2.0; /* Multiplier for all object sizes and game speed */
const PLAYER_SPEED: f32 = 500.0 * MULTIPLIER;
const FOOD_COLLISION_DISTANCE: f32 = 18.0 * MULTIPLIER;
const SNAKE_COLLISION_DISTANCE: f32 = 12.0 * MULTIPLIER;
const HEAD_FONT_SIZE: f32 = 18.0 * MULTIPLIER;
const BODY_FONT_SIZE: f32 = 16.0 * MULTIPLIER;
/* let mut moi = "moi"; */

#[derive(Component)]
struct Player;

#[derive(Component)]
struct Food;

#[derive(Component)]
struct Body;

#[derive(Resource, Default)]
struct GameState {
    score: u32,
}

#[derive(Resource, Default)]
struct SnakeBody {
    segments: Vec<Entity>,
}

#[derive(Resource)]
struct PreviousHeadPosition(Vec3);

/* Main game loop */
fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(GameState::default())
        .insert_resource(SnakeBody::default())
        .insert_resource(PreviousHeadPosition(Vec3::ZERO))
        .add_systems(Startup, setup)
        .add_systems(Update, (move_player, follow_body, eat_food).chain())
        .add_systems(Update, check_snake_colission)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);

    spawn_text_entity(
        &mut commands,
        "x",
        HEAD_FONT_SIZE,
        Color::WHITE,
        Vec3::ZERO,
        Player,
    );

    spawn_text_entity(
        &mut commands,
        "0",
        HEAD_FONT_SIZE,
        Color::WHITE,
        new_coordinate(),
        Food,
    );
}

fn move_player(
    input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut prev_head: ResMut<PreviousHeadPosition>,
    mut head: Single<&mut Transform, With<Player>>,
) {
    let old_pos = head.translation;
    let dir = input_direction(&input);

    if dir != Vec2::ZERO {
        head.translation += (dir.normalize() * PLAYER_SPEED * time.delta_secs()).extend(0.0);
    }

    prev_head.0 = old_pos;
}

/* Close the application */
fn exit_app(mut writer: AppExit) {
    AppExit::Success;
}

/* Check if death is inevitable */
fn check_snake_colission(
    snake_body: Res<SnakeBody>,
    head_query: Query<&Transform, With<Player>>,
    body_transforms: Query<&Transform, With<Body>>,
) {
    let head = match head_query.single() {
        Ok(h) => h,
        Err(_) => return,
    };

    let head_pos = head.translation.truncate();

    for &segment in &snake_body.segments {
        if let Ok(seg_t) = body_transforms.get(segment) {
            let seg_pos = seg_t.translation.truncate();
            if head_pos.distance(seg_pos) < SNAKE_COLLISION_DISTANCE {
                exit_app(AppExit::Success);
                AppExit::Success;
                return;
            }
        }
    }
}

fn follow_body(
    prev_head: Res<PreviousHeadPosition>,
    snake_body: Res<SnakeBody>,
    mut transforms: Query<&mut Transform, With<Body>>,
) {
    let mut next_pos = prev_head.0;

    for &segment_entity in &snake_body.segments {
        if let Ok(mut t) = transforms.get_mut(segment_entity) {
            let old_pos = t.translation;
            t.translation = next_pos;
            next_pos = old_pos;
        }
    }
}

fn eat_food(
    mut commands: Commands,
    mut game: ResMut<GameState>,
    mut snake_body: ResMut<SnakeBody>,
    head: Single<&Transform, (With<Player>, Without<Food>)>,
    mut food: Single<&mut Transform, (With<Food>, Without<Player>)>,
    body_transforms: Query<&Transform, (With<Body>, Without<Food>, Without<Player>)>,
) {
    let head_pos = head.translation.truncate();
    let food_pos = food.translation.truncate();

    if head_pos.distance(food_pos) < FOOD_COLLISION_DISTANCE {
        game.score += 1;

        let spawn_pos = if let Some(&tail_entity) = snake_body.segments.last() {
            body_transforms
                .get(tail_entity)
                .map(|t| t.translation)
                .unwrap_or(head.translation)
        } else {
            head.translation
        };

        let new_segment = commands
            .spawn((
                Text2d::new("0"),
                TextFont {
                    font_size: BODY_FONT_SIZE,
                    font: default(),
                    ..default()
                },
                TextColor(Color::srgb(0.2, 1.0, 0.2)),
                Transform::from_translation(spawn_pos),
                Body,
            ))
            .id();

        snake_body.segments.push(new_segment);

        food.translation = new_coordinate();
        info!("Score: {}", game.score);
    }
}

fn input_direction(input: &ButtonInput<KeyCode>) -> Vec2 {
    Vec2::new(
        axis(input, KeyCode::ArrowRight, KeyCode::ArrowLeft),
        axis(input, KeyCode::ArrowUp, KeyCode::ArrowDown),
    )
}

fn new_coordinate() -> Vec3 {
    Vec3::new(
        rand::random_range(-450.0..=450.0),
        rand::random_range(-450.0..=450.0),
        0.0,
    )
}

fn axis(input: &ButtonInput<KeyCode>, positive: KeyCode, negative: KeyCode) -> f32 {
    (input.pressed(positive) as i8 - input.pressed(negative) as i8) as f32
}

fn spawn_text_entity<T: Component>(
    commands: &mut Commands,
    text: &str,
    font_size: f32,
    color: Color,
    position: Vec3,
    marker: T,
) {
    commands.spawn((
        Text2d::new(text),
        TextFont {
            font_size,
            font: default(),
            ..default()
        },
        TextColor(color),
        Transform::from_translation(position),
        marker,
    ));
}
