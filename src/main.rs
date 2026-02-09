use std::f32::consts::PI;

use crate::player::input::PlayerInput;
use crate::player::spawn_player;
use avian3d::math::Vector3;
use avian3d::prelude::*;
use bevy::input::common_conditions::input_just_pressed;
use bevy::window::{CursorGrabMode, CursorOptions};
use bevy::{
    asset::AssetMetaCheck, light::CascadeShadowConfigBuilder, prelude::*, scene::SceneInstanceReady,
};
use bevy_ahoy::prelude::*;
use bevy_enhanced_input::prelude::*;
use bevy_skein::SkeinPlugin;

#[derive(PhysicsLayer, Default)]
enum CollisionLayer {
    #[default]
    Default,
    Player,
    Sensor,
}

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct Gun;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(AssetPlugin {
                // Wasm builds will check for meta files (that don't exist) if this isn't set.
                // This causes errors and even panics in web builds on itch.
                // See https://github.com/bevyengine/bevy_github_ci_template/issues/48.
                meta_check: AssetMetaCheck::Never,
                ..default()
            }),
            SkeinPlugin::default(),
            PhysicsPlugins::default(),
            EnhancedInputPlugin,
            AhoyPlugins::default(),
        ))
        .add_input_context::<PlayerInput>()
        // .insert_skein_preset("DefaultCharacterController", CharacterController::default())
        .add_observer(spawn_player)
        //.add_observer(gun_reparent)
        .add_systems(Startup, startup)
        .add_systems(
            Update,
            (
                capture_cursor.run_if(input_just_pressed(MouseButton::Left)),
                release_cursor.run_if(input_just_pressed(KeyCode::Escape)),
                move_gun,
            ),
        )
        .run();
}

#[derive(Component)]
struct ToReparent {
    new_parent: Entity,
}

pub fn move_gun(
    gun: Single<&mut Transform, (With<Gun>, Without<Camera3d>)>,
    camera: Single<&Transform, (With<Camera3d>, Without<Gun>)>,
) {
    let mut gun_transform = gun.into_inner();
    let cam_vec: &Transform = camera.into_inner();

    let offset = vec3(-2.0, -1.0, -4.0);

    let xx = cam_vec.local_x().as_vec3();
    let yy = cam_vec.local_y().as_vec3();
    let zz = cam_vec.local_z().as_vec3();
    gun_transform.translation =
        cam_vec.translation + (offset.x * xx) + (offset.y * yy) + (offset.z * zz);
    gun_transform.rotation = cam_vec.rotation;
    gun_transform.rotate_local_y(PI);
}

pub fn gun_reparent(
    add: On<Add, Gun>,
    // mut spawner: Query<&mut Transform>,
    camera: Single<Entity, With<Camera3d>>,
    mut commands: Commands,
) {
    // commands
    //     .entity(camera.into_inner())
    //     .insert_child(0, add.entity);

    commands
        .entity(add.entity)
        .remove::<Transform>()
        .insert(ChildOf(camera.into_inner()));
    //.insert(Transform::default());

    // let Ok(mut transform) = spawner.get_mut(add.entity) else {
    //     println!("Wah!");
    //     return;
    // };

    // transform.translation = vec3(-10.0, 0.0, 0.0);
}

pub mod player {
    use crate::CollisionLayer;
    use crate::player::input::PlayerInput;
    use avian3d::prelude::*;
    use bevy::prelude::*;
    use bevy_ahoy::prelude::*;

    pub fn spawn_player(
        add: On<Add, PlayerSpawner>,
        players: Query<Entity, With<Player>>,
        spawner: Query<&Transform>,
        camera: Single<Entity, With<Camera3d>>,
        mut commands: Commands,
    ) {
        println!("are you even working??");
        for player in players {
            // Respawn the player on hot-reloads
            commands.entity(player).despawn();
        }
        let Ok(transform) = spawner.get(add.entity).copied() else {
            println!("Wah!");
            return;
        };
        let player = commands.spawn((Player, transform)).id();
        commands
            .entity(camera.into_inner())
            .insert(CharacterControllerCameraOf {
                yank_speed: 80.0_f32.to_radians(),
                ..CharacterControllerCameraOf::new(player)
            });
    }

    #[derive(Component, Reflect)]
    #[reflect(Component)]
    #[require(
    PlayerInput,
    CharacterController {
        acceleration_hz: 10.0,
        air_acceleration_hz: 150.0,
        speed: 6.0,
        gravity: 23.0,
        friction_hz: 4.0,
        ..default()
    },
    RigidBody::Kinematic,
    Collider::cylinder(0.7, 1.8),
    CollisionLayers::new(
        [CollisionLayer::Player],
        LayerMask::ALL,
    ),
)]
    pub struct Player;

    #[derive(Component, Reflect)]
    #[reflect(Component)]
    pub struct PlayerSpawner;

    pub mod input {
        use bevy::ecs::lifecycle::HookContext;
        use bevy::ecs::world::DeferredWorld;
        use bevy::prelude::*;
        use bevy_ahoy::prelude::*;
        use bevy_enhanced_input::actions;
        use bevy_enhanced_input::prelude::*;

        #[derive(Component, Default)]
        #[component(on_add = PlayerInput::on_add)]
        pub struct PlayerInput;

        impl PlayerInput {
            pub fn on_add(mut world: DeferredWorld, ctx: HookContext) {
                world
                    .commands()
                    .entity(ctx.entity)
                    .insert(actions!(PlayerInput[

                        (
                            Action::<Movement>::new(),
                            DeadZone::default(),
                            Bindings::spawn((
                                Cardinal::wasd_keys(),
                                Axial::left_stick()
                            ))
                        ),
                        (
                            Action::<Jump>::new(),
                            bindings![KeyCode::Space,  GamepadButton::South],
                        ),
                        (
                            Action::<Crouch>::new(),
                            bindings![KeyCode::ControlLeft, GamepadButton::LeftTrigger],
                        ),
                        (
                            Action::<RotateCamera>::new(),
                            Scale::splat(0.04),
                            Bindings::spawn((
                                Spawn(Binding::mouse_motion()),
                                Axial::right_stick()
                            ))
                        ),
                    ]));
            }
        }
    }
}

#[derive(Component, Default, Reflect, Debug)]
#[reflect(Component, Default)]
#[type_path = "api"]
struct Character {
    name: String,
}

fn startup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        DirectionalLight {
            shadows_enabled: true,
            ..default()
        },
        // This is a relatively small scene, so use tighter shadow
        // cascade bounds than the default for better quality.
        // We also adjusted the shadow map to be larger since we're
        // only using a single cascade.
        CascadeShadowConfigBuilder {
            num_cascades: 1,
            maximum_distance: 1.6,
            ..default()
        }
        .build(),
    ));

    commands.spawn(Camera3d::default());
    commands.spawn((SceneRoot(asset_server.load(
        // Change this to your exported gltf file
        GltfAssetLabel::Scene(0).from_asset("World.glb"),
    )),));
    commands.spawn((SceneRoot(asset_server.load(
        // Change this to your exported gltf file
        GltfAssetLabel::Scene(0).from_asset("Character.glb"),
    )),));
}

fn capture_cursor(mut cursor: Single<&mut CursorOptions>) {
    cursor.grab_mode = CursorGrabMode::Locked;
    cursor.visible = false;
}

fn release_cursor(mut cursor: Single<&mut CursorOptions>) {
    cursor.visible = true;
    cursor.grab_mode = CursorGrabMode::None;
}
