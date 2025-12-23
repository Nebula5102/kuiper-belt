use bevy::{
    prelude::*,
    camera::visibility::RenderLayers,
    camera::RenderTarget,
    render::render_resource::{
        Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
    },
    window::WindowResized,
};
use rand::Rng;
use rand::rng;
use bevy::input::gamepad::*;
use bevy_kira_audio::{Audio, AudioControl, AudioPlugin, AudioSource};
const RES_WIDTH: u32 = 1200;
const RES_HEIGHT: u32 = 640;
const HIGH_RES_LAYERS: RenderLayers = RenderLayers::layer(1);
const BOUNDS: Vec2 = Vec2::new(1200.0, 640.0);

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugins(AudioPlugin)
        .insert_resource(Time::<Fixed>::from_hz(60.0))
        .init_resource::<AsteroidSpawner>()
        .init_resource::<PlayerControllers>()
        .add_event::<ResetGameEvent>()
        .add_systems(
            Startup, 
            (
                setup_camera, setup, load_audio_assets, setup_asteroid_spawning,
            ).chain()
        )
        .add_systems(
            FixedUpdate,
            (
                rotation,
                thrust,
                fire_laser,
                warp_drive,
                shield_system,
                //shield_system_controller,
            )
        )
        .add_systems(
            Update, 
            ( 
                fit_canvas,
                handle_connection,
                thrust_controller,
                rotation_controller,
                fire_laser_controller,
                warp_drive_controller,
                move_player, 
                projectile_movement, 
                projectile_player_collision,
                player_player_collision,
                update_health_ui,
                update_shield_ui,
                move_asteroids,            
                projectile_shield_collision,
                spawn_asteroid,
                projectile_asteroid_collision,
                ship_asteroid_collision,
                reset_key_system,
                reset_game_system,
                ship_nickel_collision,
            )
        )
        .run();
}

#[derive(Component)]
struct Canvas;

#[derive(Component)]
struct InGameCamera;

#[derive(Component)]
struct OuterCamera;

fn setup_camera(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    let canvas_size = Extent3d {
        width: RES_WIDTH,
        height: RES_HEIGHT,
        ..default()
    };

    let mut canvas = Image {
        texture_descriptor: TextureDescriptor {
            label: None,
            size: canvas_size,
            dimension: TextureDimension::D2,
            format: TextureFormat::Bgra8UnormSrgb,
            mip_level_count: 1,
            sample_count: 1,
            usage: TextureUsages::TEXTURE_BINDING
                | TextureUsages::COPY_DST
                | TextureUsages::RENDER_ATTACHMENT,
                view_formats: &[],
        },
        ..default()
    };
    canvas.resize(canvas_size);

    let image_handle = images.add(canvas);

    commands.spawn((
            Camera2d,
            Camera {
                order: -1,
                target: RenderTarget::Image(image_handle.clone().into()),
                ..default()
            },
            Msaa::Off,
            InGameCamera,
            ));
    commands.spawn((Sprite::from_image(image_handle), Canvas, HIGH_RES_LAYERS));
    commands.spawn((Camera2d, Msaa::Off, OuterCamera, HIGH_RES_LAYERS));
}

fn fit_canvas(
        mut resize_messages: MessageReader<WindowResized>,
        mut projection: Single<&mut Projection, With<OuterCamera>>,
    ) {
    let Projection::Orthographic(projection) = &mut **projection else {
        return;
    };
    for window_resized in resize_messages.read() {
        let h_scale = window_resized.width / RES_WIDTH as f32;
        let v_scale = window_resized.height / RES_HEIGHT as f32;
        projection.scale = 1. / h_scale.min(v_scale).round();
    }
}

#[derive(Message)]
struct ResetGameEvent;

fn reset_game_system(
    mut commands: Commands,
    mut reset_reader: EventReader<ResetGameEvent>,
    game_entities: Query<Entity, With<GameEntity>>,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    query: Query<(&PlayerId, &Health)>
) {
    if reset_reader.is_empty() {
        return;
    }
    reset_reader.clear();

    // Despawn everything that belongs to the game
    for entity in &game_entities {
        commands.entity(entity).despawn();
    }

    // Recreate initial state
    setup(commands, asset_server,meshes,materials); 
}

#[derive(Component)]
struct Nickel {
    radius: f32,
}


fn reset_key_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut reset_writer: MessageWriter<ResetGameEvent>,
) {
    if keyboard.just_pressed(KeyCode::KeyR) {
        reset_writer.write(ResetGameEvent);
    }
}
#[derive(Component)]
struct GameEntity;

#[derive(Component)]
struct HealthPoint {
    red: bool,
    blue: bool,
}

#[derive(Component, Clone, Copy, PartialEq, Eq)]
enum PlayerId {
    One,
    Two,
}

#[derive(Component)]
struct Health {
    hp: i32,
}

#[derive(Component)]
struct Player {
    movement_speed: f32,
    rotation_speed: f32,
    velocity: Vec2,
    radius: f32,
    color: u32,
}

#[derive(Component)]
struct Asteroid {
    velocity: Vec2,
    radius: f32,
}

#[derive(Resource, Default)]
struct AsteroidSpawner {
    timer: Timer,
}

#[derive(Component)]
struct ShieldHealth {
    shp: f32,
}

#[derive(Component)]
struct ShieldPoint {
    red: bool,
    blue: bool,
}

#[derive(Component)]
struct Shield; 

#[derive(Component)]
struct Projectile {
    velocity: Vec2,
    radius: f32,
    owner: PlayerId,
}

#[derive(Resource)]
struct Sounds {
    laser: Handle<AudioSource>,
    damage: Handle<AudioSource>,
}

//implementation 2
#[derive(Component)]
struct AssignedController {
    gp: Option<Entity>,
}

#[derive(Resource, Default)]
struct PlayerControllers {
    p1: Option<Entity>,
    p2: Option<Entity>,
}

#[derive(Component)]
struct WarpCooldown {
    timer: Timer,
}

fn handle_connection(
    mut events: MessageReader<GamepadConnectionEvent>,
    mut controllers: ResMut<PlayerControllers>,
) {
    for event in events.read() {
        match &event.connection {
            GamepadConnection::Connected { name, vendor_id, product_id } => {
                println!("Connected");
                if controllers.p1.is_none() {
                    controllers.p1 = Some(event.gamepad);
                } else if controllers.p2.is_none() {
                    controllers.p2 = Some(event.gamepad);
                }                                
            }
            GamepadConnection::Disconnected => {
                println!("Disconnected");
            }
        }
    }
}

fn load_audio_assets(mut commands: Commands, asset_server: Res<AssetServer>) {
    let laser_sound = asset_server.load("sounds/laser.ogg"); 
    let damage_sounds = asset_server.load("sounds/damage.ogg"); //Hew moaning
    commands.insert_resource(
        Sounds { 
            laser: laser_sound,
            damage: damage_sounds,
        });
}

fn setup(
            mut commands: Commands, 
            asset_server: Res<AssetServer>,
            mut meshes: ResMut<Assets<Mesh>>,
            mut materials: ResMut<Assets<ColorMaterial>>,
         ) {
    let blueship = asset_server.load("starblue.png");
    let redship = asset_server.load("starred.png");

    let max_health = 500;

    // Player 1
    commands.spawn((
        Sprite::from_image(redship),
        Player {
            movement_speed: 500.0,
            rotation_speed: f32::to_radians(300.0),
            velocity: Vec2::ZERO,
            radius: 17.0,
            color: 1,
        },
        PlayerId::One,
        Health { hp: max_health },
        Transform::from_xyz(-300.0, 0.0, 0.0),
        WarpCooldown{
            timer: Timer::from_seconds(5.0, TimerMode::Once),
        },
        GameEntity,
        ShieldHealth {shp: 500.0},
    ));

    // Player 2
    commands.spawn((
        Sprite::from_image(blueship),
        Player {
            movement_speed: 500.0,
            rotation_speed: f32::to_radians(300.0),
            velocity: Vec2::ZERO,
            radius: 17.0,
            color: 2,
        },
        PlayerId::Two,
        Health { hp: max_health },
        Transform::from_xyz(300.0, 0.0, 0.0),
        WarpCooldown{
            timer: Timer::from_seconds(5.0, TimerMode::Once),
        },
        GameEntity,
        ShieldHealth {shp: 500.0},
    ));
    let mut space = 300.0;
    commands.spawn((
        Mesh2d(meshes.add(Rectangle::new(100.,20.0))),
        MeshMaterial2d(materials.add(Color::srgba(0.2,0.2,1.0,0.5))),
        Transform::from_xyz(space+87., 270., 0.0),
        ShieldPoint {
                red: false,
                blue: true,
            },
    ));
    for _ in 0..((max_health+1)/100) {
        space += 25.;
        let shape = meshes.add(Rectangle::new(20.0,20.0));
        commands.spawn((
            Mesh2d(shape),
            MeshMaterial2d(materials.add(Color::srgb(0.2,0.2,1.0))),
            Transform::from_xyz(space, 300., 0.0),
            HealthPoint {
                red: false,
                blue: true,
            },
            GameEntity,
        ));
    }
    space = -500.0;
       commands.spawn((
        Mesh2d(meshes.add(Rectangle::new(100.0,20.0))),
        MeshMaterial2d(materials.add(Color::srgba(1.0,0.2,0.2,0.5))),
        Transform::from_xyz(space+87., 270., 0.0),
        ShieldPoint {
                red: true,
                blue: false,
            },
    ));
    for _ in 0..((max_health+1)/100) {
        space += 25.;
        let shape = meshes.add(Rectangle::new(20.0,20.0));
        commands.spawn((
            Mesh2d(shape),
            MeshMaterial2d(materials.add(Color::srgb(1.0,0.2,0.0))),
            Transform::from_xyz(space, 300., 0.0),
            HealthPoint {
                red: true,
                blue: false,
            },
            GameEntity,
        ));
    }
}

fn update_shield_ui(
    mut commands: Commands,
    shp_ui: Query<(Entity, &ShieldPoint)>,
    qshield: Query<(&ShieldHealth, &Player), Changed<ShieldHealth>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,    
    ) {
    let hp_blocks: Vec<_> = shp_ui.iter().collect();

    for (hp, player) in &qshield {
        let size = 100. - (100. - (hp.shp/5.));
        if player.color == 2 {
            // Blue player
            for (entity, block) in hp_blocks.iter() {
                let space = 300.0;
                if block.blue {
                    commands.entity(*entity).despawn();
                    commands.spawn((
                        Mesh2d(meshes.add(Rectangle::new(size,20.0))),
                        MeshMaterial2d(materials.add(Color::srgba(0.2,0.2,1.0,0.5))),
                        Transform::from_xyz(space+87., 270., 0.0),
                        ShieldPoint {
                            red: false,
                            blue: true,
                        },
                        GameEntity,
                    ));
                    break;
                }
            }
        } 
        if player.color == 1 {
            // Red player
            for (entity, block) in hp_blocks.iter() {
                let space = -500.0;
                if block.red {
                    commands.entity(*entity).despawn();
                    commands.spawn((
                        Mesh2d(meshes.add(Rectangle::new(size,20.0))),
                        MeshMaterial2d(materials.add(Color::srgba(1.0,0.2,0.2,0.5))),
                        Transform::from_xyz(space+87., 270., 0.0),
                        ShieldPoint {
                            red: true,
                            blue: false,
                        },
                        GameEntity,
                    ));
                    break;
                }
            }
        }
    }
}

fn update_health_ui(
    mut commands: Commands,
    hp_ui: Query<(Entity, &HealthPoint)>,
    qhealth: Query<(&Health, &Player), Changed<Health>>,
    audio: Res<Audio>,
    sounds: Res<Sounds>,
) {
    let hp_blocks: Vec<_> = hp_ui.iter().collect();

    for (hp, player) in &qhealth {
        if player.color == 2 {
            // Blue player
            if (hp.hp as f32 / 100.) - (hp.hp as f32 / 100.) != (hp.hp % 100) as f32 {
                continue
            }
            for (entity, block) in hp_blocks.iter() {
                if block.blue {
                    commands.entity(*entity).despawn();
                    audio.play(sounds.damage.clone());
                    break;
                }
            }
        } 
        if player.color == 1{
            // Red player
            if (hp.hp as f32 / 100.) - (hp.hp as f32 / 100.) != (hp.hp % 100) as f32 {
                continue
            }
            for (entity, block) in hp_blocks.iter() {
                if block.red {
                    commands.entity(*entity).despawn();
                    audio.play(sounds.damage.clone());
                    break;
                }
            }
        }
    }
}


fn gamepad_for_player(
    controllers: &PlayerControllers,
    id: PlayerId,
) -> Option<Entity> {
    match id {
        PlayerId::One => controllers.p1,
        PlayerId::Two => controllers.p2,
    }
}

fn rotation_controller(
    time: Res<Time>,
    controllers: Res<PlayerControllers>,
    gamepads: Query<(Entity, &Gamepad)>,
    mut query: Query<(&PlayerId, &Player, &mut Transform)>,
) {
    for (id, ship, mut transform) in &mut query {
        // Skip if no gamepad assigned
        
        let Some(gamepad) = gamepad_for_player(&controllers, *id) else {
            continue;
        };

        let mut rotation_factor = 0.0;
        for (e, gp) in &gamepads {
            if Some(e) == Some(gamepad) {
                if gp.pressed(GamepadButton::DPadLeft) {
                    rotation_factor += 1.0;
                }
                if gp.pressed(GamepadButton::DPadRight) {
                    rotation_factor -= 1.0;
                }                
            }
                        
        }
        // Apply rotation
        transform.rotate_z(rotation_factor * ship.rotation_speed * time.delta_secs());
    }
}




fn rotation(
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&PlayerId, &Player, &mut Transform)>,
) {
    for (id, ship, mut transform) in &mut query {
        let mut rotation_factor = 0.0;

        match id {
            PlayerId::One => {
                if keyboard.pressed(KeyCode::KeyA) {
                    rotation_factor += 1.0;
                }
                if keyboard.pressed(KeyCode::KeyD) {
                    rotation_factor -= 1.0;
                }
            }
            PlayerId::Two => {
                if keyboard.pressed(KeyCode::ArrowLeft) {
                    rotation_factor += 1.0;
                }
                if keyboard.pressed(KeyCode::ArrowRight) {
                    rotation_factor -= 1.0;
                }
            }
        }

        transform.rotate_z(rotation_factor * ship.rotation_speed * time.delta_secs());
    }
}


fn thrust_controller(
    time: Res<Time>,
    controllers: Res<PlayerControllers>,
    gamepads: Query<(Entity, &Gamepad)>,
    mut controller_query: Query<(&mut Player, &PlayerId, &Transform)>,
) {
    for (mut ship, id, transform) in &mut controller_query {
        // Get assigned gamepad for this PlayerId
        let Some(gamepad) = gamepad_for_player(&controllers, *id) else {
            continue;
        };

        let mut pressed = false;
        for (e, gp) in gamepads {
            if Some(e) == Some(gamepad) {
                if gp.pressed(GamepadButton::DPadUp) {
                    pressed = true;
                    break;
                }
            }
        }

        // If the button is pressed
        if pressed {
            let speed = ship.movement_speed;
            let forward = (transform.rotation * Vec3::Y).truncate();
            ship.velocity += forward * speed * time.delta_secs();
        }
    }
}


fn thrust(
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&PlayerId, &mut Player, &Transform)>,
) {
    for (id, mut ship, transform) in &mut query {
        let forward = (transform.rotation * Vec3::Y).truncate();
        let speed = ship.movement_speed;

        let pressing = match id {
            PlayerId::One => keyboard.pressed(KeyCode::KeyW),
            PlayerId::Two => keyboard.pressed(KeyCode::ArrowUp),
        };

        if pressing {
            ship.velocity += forward * speed * time.delta_secs();
        }
    }
}

fn move_player(
    time: Res<Time>,
    mut query: Query<(&mut Player, &mut Transform)>,
) {
    for (mut ship, mut transform) in &mut query {
        let friction = 0.8;
        let vel = ship.velocity;
        ship.velocity = vel - vel * friction * time.delta_secs();

        let delta = ship.velocity * time.delta_secs();
        transform.translation.x += delta.x;
        transform.translation.y += delta.y;

        let extents = Vec3::from((BOUNDS / 2.0, 0.0));
        transform.translation = transform.translation.min(extents).max(-extents);
    }
}


fn shield_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut player_query: Query<(Entity, &PlayerId, &mut ShieldHealth, Option<&Children>), Without<Shield>>,
    shielded_query: Query<&ChildOf, With<Shield>>,
) {
    for (entity, id, mut shield, children) in &mut player_query {
        let pressed = match id {
            PlayerId::One => keyboard.pressed(KeyCode::KeyQ),
            PlayerId::Two => keyboard.pressed(KeyCode::ShiftRight),
        };

        let has_shield = children.map_or(false, |children| {
            children.iter().any(|child| shielded_query.get(child).is_ok())
        });

        if shield.shp <= 0. {
            if let Some(children) = children {
                for child in children.iter() {
                    if shielded_query.get(child).is_ok() {
                        commands.entity(child).despawn();
                    }
                }
            }
            continue;           
        }
        // PRESSING — ensure shield exists
        if pressed {
            // Skip if shield already exists
            if has_shield {
                if shield.shp >= 0. {
                    shield.shp -= 1.0;               
                }
                continue;
            }
                       
            let shield_mesh = meshes.add(Circle::new(40.0).mesh());
            let shield_material = materials.add(Color::srgba(0.3, 0.7, 1.0, 0.4));

            // Spawn shield as child of player
            commands.entity(entity).with_children(|parent| {
                parent.spawn((
                    Shield,
                    Mesh2d(shield_mesh),
                    MeshMaterial2d(shield_material),
                    Transform::default(),
                    GlobalTransform::default(),
                ));
            });
        } else {
            // NOT PRESSING — remove shield child if exists
            if let Some(children) = children {
                for child in children.iter() {
                    if shielded_query.get(child).is_ok() {
                        commands.entity(child).despawn();
                    }
                }
            }
        }
    }
}


fn shield_system_controller(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    gamepads: Query<(Entity, &Gamepad)>,
    controllers: Res<PlayerControllers>,     
    mut player_query: Query<(Entity, &PlayerId, &mut ShieldHealth, Option<&Children>), Without<Shield>>,
    shielded_query: Query<&ChildOf, With<Shield>>,
) {
    for (entity, id, mut shield, children) in &mut player_query {
        let mut pressed = false;
        let Some(gamepad) = gamepad_for_player(&controllers, *id) else {
            continue;
        };
        // Match assigned controller
        for (e, gp) in &gamepads {
            if Some(e) == Some(gamepad) {
                // NORTH = shield button
                if gp.pressed(GamepadButton::North) {
                    pressed = true;
                }
            }
        }

        let has_shield = children.map_or(false, |children| {
            children.iter().any(|child| shielded_query.get(child).is_ok())
        });


        if shield.shp <= 0. {
            if let Some(children) = children {
                for child in children.iter() {
                    if shielded_query.get(child).is_ok() {
                        commands.entity(child).despawn();
                    }
                }
            }
            continue;           
        }        
        // PRESSING — ensure shield exists
        if pressed {
                        
            if has_shield {
                if shield.shp >= 0. {
                    shield.shp -= 1.0;               
                }
                continue;
            }
            
            
            let shield_mesh = meshes.add(Circle::new(40.0).mesh());
            let shield_material = materials.add(Color::srgba(0.3, 0.7, 1.0, 0.4));

            commands.entity(entity).with_children(|parent| {
                parent.spawn((
                    Shield,
                    Mesh2d(shield_mesh),
                    MeshMaterial2d(shield_material),
                    Transform::default(),
                    GlobalTransform::default(),
                ));
            });
        }
        // NOT PRESSING — remove shield child if it exists
        else {
            if let Some(children) = children {
                for child in children.iter() {
                    if shielded_query.get(child).is_ok() {
                        commands.entity(child).despawn();
                    }
                }
            }
        }
    }
}


fn projectile_shield_collision(
    mut commands: Commands,
    projectile_query: Query<(Entity, &Transform, &Projectile)>,
    mut player_query: Query<(Entity, &Transform, &Player, &mut ShieldHealth, &PlayerId, Option<&Children>, )>,
    shielded_query: Query<&ChildOf, With<Shield>>,
) {
    for (proj_entity, proj_tf, proj) in &projectile_query {
        let proj_pos = proj_tf.translation.truncate();

        for (player_entity, player_tf, player, mut shield, player_id, children) in &mut player_query {
            if proj.owner == *player_id {
                continue; // don't hit yourself
            }

            // Check if player has a shield child
            let shield_active = children.map_or(false, |children| {
                children.iter().any(|child| {
                    // a shield child exists
                    // safer than querying full shield list
                    true // we already filtered in shield_system
                })
            });

            let player_pos = player_tf.translation.truncate();
            let distance = player_pos.distance(proj_pos);

            if (distance < player.radius + proj.radius + 20.) && shield_active {
                // Shield absorbs but does NOT destroy projectile
                
                if shield.shp - 100. < 0. {
                    shield.shp = 0.;
                } else {
                    shield.shp -= 100.;
                }                
                if shield.shp <= 0. {
                    if let Some(children) = children {
                        for child in children.iter() {
                            if shielded_query.get(child).is_ok() {
                                commands.entity(child).despawn();
                            }
                        }
                    }
                }
                commands.entity(proj_entity).despawn();
                continue;
            }
        }
    }
}

fn warp_drive(
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&PlayerId, &mut Transform, &mut WarpCooldown)>,
) {
    let warp_distance = 200.0;

    for (id, mut transform, mut cooldown) in &mut query {
        cooldown.timer.tick(time.delta());
        if !cooldown.timer.finished() {
            continue;
        }
        let warp_pressed = match id {
            PlayerId::One => keyboard.just_pressed(KeyCode::KeyS),
            PlayerId::Two => keyboard.just_pressed(KeyCode::ArrowDown),
        };

        if warp_pressed {
            let forward = (transform.rotation * Vec3::Y).truncate();
            let new_pos = transform.translation + (forward.extend(0.0) * warp_distance);

            let extents = Vec3::from((BOUNDS / 2.0, 0.0));
            transform.translation = new_pos.min(extents).max(-extents);
            cooldown.timer.reset();
        }
    }
}

fn warp_drive_controller(
    time: Res<Time>,
    controllers: Res<PlayerControllers>,
    gamepads: Query<(Entity, &Gamepad)>,
    mut query: Query<(&PlayerId, &Player, &mut Transform, &mut WarpCooldown)>,
) {
    let warp_distance = 200.0;

    for (id, ship, mut transform, mut cooldown) in &mut query {
        cooldown.timer.tick(time.delta());
        if !cooldown.timer.finished() {
            continue;
        }
        let mut warp_pressed = false;
        let Some(gamepad) = gamepad_for_player(&controllers, *id) else {
            continue;
        };

        // Match controller like the other controller systems
        for (e, gp) in &gamepads {
            if Some(e) == Some(gamepad) {
                // SOUTH = warp button
                if gp.just_pressed(GamepadButton::South) {
                    warp_pressed = true;
                    break;

                }
            }
        }

        if warp_pressed {
            let forward = (transform.rotation * Vec3::Y).truncate();
            let new_pos = transform.translation + (forward.extend(0.0) * warp_distance);

            let extents = Vec3::from((BOUNDS / 2.0, 0.0));
            transform.translation = new_pos.min(extents).max(-extents);
            cooldown.timer.reset();
        }
    }
}


fn projectile_movement(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &Projectile)>,
) {
    for (mut transform, projectile) in &mut query {
        let delta = projectile.velocity * time.delta_secs();
        transform.translation.x += delta.x;
        transform.translation.y += delta.y;
    }
}

fn projectile_color_for(
        id: PlayerId
    ) -> Color {
    match id {
        PlayerId::One => Color::srgb(1.0,0.2,0.2),
        PlayerId::Two => Color::srgb(0.2,0.2,1.0),
    }
}

fn fire_laser(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    query: Query<(&Transform, &Player, &PlayerId)>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    audio: Res<Audio>,
    sounds: Res<Sounds>,
) {
    for (transform, _, id) in &query {

        let shoot = match id {
            PlayerId::One => keyboard_input.just_pressed(KeyCode::Space),
            PlayerId::Two => keyboard_input.just_pressed(KeyCode::Enter),
        };

        if shoot {
            let color = projectile_color_for(*id);

            let forward = (transform.rotation * Vec3::Y).truncate().normalize();

            let mesh = Circle::new(4.0).mesh().build();
            let mesh_handle = meshes.add(mesh);

            commands.spawn((
                Mesh2d(mesh_handle),
                MeshMaterial2d(materials.add(color)),
                Transform::from_translation(transform.translation),
                Projectile {
                    velocity: forward * 400.0,
                    radius: 5.0,
                    owner: *id,
                },
            ));
            audio.play(sounds.laser.clone());
        }
    }
}




fn fire_laser_controller(
    gamepads: Query<(Entity, &Gamepad)>,
    controllers: Res<PlayerControllers>,
    query: Query<(&Transform, &Player, &PlayerId)>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    audio: Res<Audio>,
    sounds: Res<Sounds>,
) {
    for (transform, _player, id) in &query {
        let mut shoot = false;

        let Some(gamepad) = gamepad_for_player(&controllers, *id) else {
            continue;
        };        // Loop over every connected controller (same pattern as rotation + thrust controller)
        for (e, gp) in &gamepads {
            // Compare controller to assigned controller
            if Some(e) == Some(gamepad) {
                // West button = Fire
                if gp.just_pressed(GamepadButton::West) {
                    shoot = true;
                    break;
                }
            }
        }

        if shoot {
            let color = projectile_color_for(*id);

            let forward = (transform.rotation * Vec3::Y).truncate().normalize();

            let mesh = Circle::new(4.0).mesh().build();
            let mesh_handle = meshes.add(mesh);

            commands.spawn((
                Mesh2d(mesh_handle),
                MeshMaterial2d(materials.add(color)),
                Transform::from_translation(transform.translation),
                Projectile {
                    velocity: forward * 400.0,
                    radius: 5.0,
                    owner: *id,
                },
            ));

            audio.play(sounds.laser.clone());
        }
    }
}


fn projectile_player_collision(
    mut commands: Commands,
    projectile_query: Query<(Entity, &Transform, &Projectile)>,
    mut player_query: Query<(Entity, &Transform, &Player, &PlayerId, &mut Health)>,
    audio: Res<Audio>,
    sounds: Res<Sounds>,
) {
    for (proj_entity, proj_tf, proj) in &projectile_query {
        let proj_pos = proj_tf.translation.truncate();

        for (player_entity, player_tf, player, player_id, mut health) in &mut player_query {
            if proj.owner == *player_id {
                continue; // don't hit yourself
            }

            let player_pos = player_tf.translation.truncate();
            let distance = player_pos.distance(proj_pos);

            if distance < player.radius + proj.radius {
                // hit detected
                commands.entity(proj_entity).despawn();

                if (health.hp as f32 / 100.) - (health.hp as f32 /100.) != (health.hp % 100) as f32 {
                    let last_num = health.hp % 100;
                    health.hp -= last_num;
                } else {
                    health.hp -= 100;
                }

                if health.hp <= 0 {
                    commands.entity(player_entity).despawn();
                }
                break;
            }
        }
    }
}

fn player_player_collision(
    mut commands: Commands,
    query: Query<(Entity, &Transform, &Player, &PlayerId)>,
    mut hq: Query<&mut Health>,
) {
    let players: Vec<_> = query.iter().collect();

    if players.len() < 2 { return; }

    let (e1, tf1, p1, id1) = players[0];
    let (e2, tf2, p2, id2) = players[1];

    let pos1 = tf1.translation.truncate();
    let pos2 = tf2.translation.truncate();

    let dist = pos1.distance(pos2);
    if dist < p1.radius + p2.radius {
        let v1 = p1.velocity.length();
        let v2 = p2.velocity.length();
        if v1 > v2 {
            if let Ok(mut h2) = hq.get_mut(e2) {
                h2.hp -= 5;
                if h2.hp <= 0 {
                    commands.entity(e2).despawn();
                }
            }
        } else {
            if let Ok(mut h1) = hq.get_mut(e1) {
                h1.hp -= 5;
                if h1.hp <= 0 {
                    commands.entity(e1).despawn();
                }
            }
        }
    }
}

fn spawn_asteroid(
    time: Res<Time>,
    mut spawner: ResMut<AsteroidSpawner>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    spawner.timer.tick(time.delta());
    if !spawner.timer.finished() {
        return;
    }

    // ----- Generate random spawn position along screen edges -----
    let mut rng = rand::thread_rng();

    // Randomly pick which edge to spawn on
    let side = rng.gen_range(0..4);
    let x;
    let y;

    // Screen bounds (same as yours)
    let half = BOUNDS / 2.0;

    match side {
        0 => { // Top
            x = rng.gen_range(-half.x..half.x);
            y = half.y;
        }
        1 => { // Bottom
            x = rng.gen_range(-half.x..half.x);
            y = -half.y;
        }
        2 => { // Left
            x = -half.x;
            y = rng.gen_range(-half.y..half.y);
        }
        _ => { // Right
            x = half.x;
            y = rng.gen_range(-half.y..half.y);
        }
    }

    let spawn_pos = Vec3::new(x, y, 0.0);
    
    let center = Vec2::ZERO;
    let direction_to_center = (center - Vec2::new(x, y)).normalize();
    let mut direction = direction_to_center;

    /*
    let angle_offset = rng.gen_range(-0.5..0.5); // ± ~30 degrees
    let offset = Vec2::new(angle_offset,angle_offset);
    direction = direction.rotate(offset);
    */

    

    // ----- Give asteroid a random velocity toward center-ish -----
    let speed = rng.gen_range(20.0..70.0);

    let velocity = direction * speed;

    // ----- Create asteroid mesh -----
    let mesh = meshes.add(Circle::new(70.0).mesh());
    let material = materials.add(Color::srgb(0.8, 0.8, 0.8));

    // ----- Spawn asteroid entity -----
    commands.spawn((
        Mesh2d(mesh.into()),
        MeshMaterial2d(material),
        Transform::from_translation(spawn_pos),
        Asteroid { 
            velocity,
            radius: 70.0,
        },
        GameEntity,
    ));
}

fn move_asteroids(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &Asteroid)>
) {
    for (mut transform, asteroid) in &mut query {
        let delta = asteroid.velocity * time.delta_secs();
        transform.translation.x += delta.x;
        transform.translation.y += delta.y;
    }
}


fn setup_asteroid_spawning (
        mut commands: Commands,
    ) {
    commands.insert_resource(AsteroidSpawner {
        timer: Timer::from_seconds(5.0,TimerMode::Repeating),
    });
}

fn ship_asteroid_collision(
    mut commands: Commands,
    mut players: Query<(Entity, &Transform, &Player, &mut Health)>,
    asteroids: Query<(Entity, &Transform, &Asteroid), Without<Player>>,
) {
    for (player_entity, player_transform, player, mut health) in players.iter_mut() {
        let player_pos = player_transform.translation.truncate();
        let player_radius = player.radius;

        for (asteroid_entity, asteroid_transform, asteroid) in asteroids.iter() {
            let asteroid_pos = asteroid_transform.translation.truncate();
            let asteroid_radius = asteroid.radius;

            let distance = player_pos.distance(asteroid_pos);

            if distance < player_radius + asteroid_radius {
                // Damage player

                let even = health.hp % 100;
                if even == 0 {
                    health.hp -= 100;
                } else {
                    health.hp = health.hp - even;
                }
                if health.hp<= 0 {
                    commands.entity(player_entity).despawn();
                }

                // Destroy asteroid
                commands.entity(asteroid_entity).despawn();

                // Optional: break so one asteroid only hits once
                break;
            }
        }
    }
}


fn projectile_asteroid_collision(
    mut commands: Commands,
    projectile_query: Query<(Entity, &Transform, &Projectile), Without<Asteroid>>,
    asteroid_query: Query<(Entity, &Transform, &Asteroid), Without<Projectile>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>
) {
    // Check each projectile
    for (projectile_entity, projectile_transform, projectile) in projectile_query.iter() {
        let projectile_pos = projectile_transform.translation.truncate();

        // Check each asteroid
        for (asteroid_entity, asteroid_transform, asteroid) in asteroid_query.iter() {
            let asteroid_pos = asteroid_transform.translation.truncate();

            let distance = projectile_pos.distance(asteroid_pos);

            if distance < projectile.radius + asteroid.radius {
                // Hit!
                commands.entity(asteroid_entity).despawn();
                commands.entity(projectile_entity).despawn();

                if asteroid.radius > 30.0 {
                    let new_radius = asteroid.radius * 0.5;

                    let mut rng = rand::thread_rng();

                    let num_children = rng.gen_range(2..4);

                    for _ in 0..num_children {
                        let angle = rng.gen_range(0.0..std::f32::consts::TAU);
                        let direction = Vec2::from_angle(angle);

                        // child velocity
                        let speed = rng.gen_range(60.0..160.0);
                        let velocity = direction * speed;

                        // Build mesh for child
                        let mesh = meshes.add(Circle::new(new_radius).mesh());
                        let material = materials.add(Color::srgb(0.8, 0.8, 0.8));
                        commands.spawn((
                                Mesh2d(mesh),
                                MeshMaterial2d(material),
                                Transform::from_translation(
                                    asteroid_transform.translation,
                                ),
                            Asteroid {
                                radius: new_radius,
                                velocity,
                            },
                            GameEntity,
                        ));
                    }
                } else {
                        let angle = rng().gen_range(0.0..std::f32::consts::TAU);

                        // child velocity
                        let speed = rng().gen_range(10.0..60.0);
                        let mesh = meshes.add(Circle::new(7.0).mesh());
                        let material = materials.add(Color::srgb(0.2, 0.8, 0.8));
                        commands.spawn((
                                Mesh2d(mesh),
                                MeshMaterial2d(material),
                                Transform::from_translation(
                                    asteroid_transform.translation,
                                ),
                            Nickel {
                                radius: 7.0,
                            },
                            GameEntity,
                        ));
                }
                break; // stop checking after hit
            }
        }
    }
}

fn ship_nickel_collision(
    mut commands: Commands,
    mut players: Query<(Entity, &Transform, &mut ShieldHealth, &Player)>,
    asteroids: Query<(Entity, &Transform, &Nickel), Without<Player>>,
) {

    for (player_entity, player_transform, mut shp, player ) in players.iter_mut() {
        let player_pos = player_transform.translation.truncate();
        let player_radius = player.radius;

        for (nickel_entity, nickel_transform, nickel) in asteroids.iter() {
            let nickel_pos = nickel_transform.translation.truncate();
            let nickel_radius = nickel.radius;

            let distance = player_pos.distance(nickel_pos);

            if distance < player_radius + nickel_radius {
                // Damage player
                shp.shp += 100.0;

                // Destroy asteroid
                commands.entity(nickel_entity).despawn();

                // Optional: break so one asteroid only hits once
                break;
            }
        }
    }
}
