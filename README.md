# Bevy Transform Gizmo

This Bevy plugin adds a transform gizmo to entities in the scene, allowing you to drag and rotate meshes with your mouse.

![demo](https://user-images.githubusercontent.com/2632925/119207591-7c931d00-ba53-11eb-93f1-795064089ac3.gif)

# Demo

Run a minimal implementation of the gizmo by cloning this repository and running:

```shell
cargo run --example minimal
```

# Features

* Translation handles
* Rotation handles
* Gizmo always renders on top of the main render pass
* Gizmo scales at it moves closer/further from the camera

# Usage

This plugin is built on and relies on [`bevy_mod_picking`](https://github.com/aevyrie/bevy_mod_picking) for 3d mouse interaction with the scene. 

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