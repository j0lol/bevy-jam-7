use bevy::{asset::AssetMetaCheck, prelude::*, scene::SceneInstanceReady};
use bevy_skein::SkeinPlugin;

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
        ))
        .add_observer(
            // log the component from the gltf spawn
            |ready: On<SceneInstanceReady>,
             children: Query<&Children>,
             characters: Query<&Character>| {
                for entity in children.iter_descendants(ready.entity) {
                    let Ok(character) = characters.get(entity) else {
                        continue;
                    };
                    info!(?character);
                }
            },
        )
        .add_systems(Startup, startup)
        .run();
}

#[derive(Component, Default, Reflect, Debug)]
#[reflect(Component, Default)]
#[type_path = "api"]
struct Character {
    name: String,
}

fn startup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(SceneRoot(asset_server.load(
        // Change this to your exported gltf file
        GltfAssetLabel::Scene(0).from_asset("demo.gltf"),
    )));
}
