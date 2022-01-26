# Bevy Transform Gizmo

This Bevy plugin adds a transform gizmo to entities in the scene, allowing you to drag and rotate meshes with your mouse.

https://user-images.githubusercontent.com/2632925/119217311-33ac8a00-ba8e-11eb-9dfb-db0b9c13cd84.mp4

# Demo

Run a minimal implementation of the gizmo by cloning this repository and running:

```shell
cargo run --example minimal
```

# Features

* Prebuilt transform gizmo appears when you select a designated mesh
* Translation handles
* Rotation handles
* Gizmo always renders on top of the main render pass
* Gizmo scales at it moves closer/further from the camera

# Usage

This plugin is built on and relies on [`bevy_mod_picking`](https://github.com/aevyrie/bevy_mod_picking) for 3d mouse interaction with the scene.

Add the plugin to the `[dependencies]` in `Cargo.toml`

```toml
bevy_transform_gizmo = { git = "https://github.com/ForesightMiningSoftwareCorporation/bevy_transform_gizmo", branch = "main" }
```

You will need to add the transform gizmo plugin, as well as make sure you have also brought in the picking plugin.

```rust
.add_plugin(bevy_mod_picking::DefaultPickingPlugins)
.add_plugin(bevy_transform_gizmo::TransformGizmoPlugin)
```

Next, you will need to mark your picking camera as your gizmo camera:

```rust
.insert_bundle(bevy_mod_picking::PickingCameraBundle::default())
.insert(bevy_transform_gizmo::GizmoPickSource::default());
```

Finally, mark any meshes you want to be transformed with the gizmo; note they must also be selectable in the picking plugin:

```rust
.insert_bundle(bevy_mod_picking::PickableBundle::default())
.insert(bevy_transform_gizmo::GizmoTransformable);
```

See the [minimal](examples/minimal.rs) demo for an example of a minimal implementation.

# License

bevy_transform_gizmo is free and open source! All code in this repository is dual-licensed under either:

* MIT License (LICENSE-MIT or http://opensource.org/licenses/MIT)
* Apache License, Version 2.0 (LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0)

at your option. This means you can select the license you prefer! This dual-licensing approach is the de-facto standard in the Rust ecosystem and there are very good reasons to include both.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.

## Sponsors
The creation and maintenance of Bevy Polyline is sponsored by Foresight Mining Software Corporation.

<img src="fse.png" alt="Foresight Mining Software Corporation" width="480">
