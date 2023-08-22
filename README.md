<div align="center">
    
# Bevy Transform Gizmo

**Simple 3D transform gizmo for bevy**

https://user-images.githubusercontent.com/2632925/227469248-726b21c8-5308-49f0-9b04-e567833774e1.mp4

[![crates.io](https://img.shields.io/crates/v/bevy_transform_gizmo)](https://crates.io/crates/bevy_transform_gizmo)
[![docs.rs](https://docs.rs/bevy_transform_gizmo/badge.svg)](https://docs.rs/bevy_transform_gizmo)
[![CI](https://github.com/ForesightMiningSoftwareCorporation/bevy_transform_gizmo/workflows/CI/badge.svg?branch=main)](https://github.com/ForesightMiningSoftwareCorporation/bevy_transform_gizmo/actions?query=workflow%3A%22CI%22+branch%3Amain)
[![Bevy tracking](https://img.shields.io/badge/Bevy%20tracking-main-lightblue)](https://github.com/bevyengine/bevy/blob/main/docs/plugins_guidelines.md#main-branch-tracking)
    
</div>

# Demo

Run a minimal implementation of the gizmo by cloning this repository and running:

```shell
cargo run --example minimal
```

# Features

* Prebuilt transform gizmo appears when you select a designated mesh
* Translation handles (axis, plane, and normal to camera)
* Rotation handles
* Gizmo always renders on top of the main render pass
* Gizmo is always the same size at it moves closer/further from the camera

# Usage

This plugin is built on and relies on [`bevy_mod_picking`](https://github.com/aevyrie/bevy_mod_picking) for 3d mouse interaction with the scene.

See the [minimal](examples/minimal.rs) demo for an example of a minimal implementation.

# License

bevy_transform_gizmo is free and open source! All code in this repository is dual-licensed under either:

* MIT License ([LICENSE-MIT](LICENSE-MIT) or [http://opensource.org/licenses/MIT](http://opensource.org/licenses/MIT))
* Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or [http://www.apache.org/licenses/LICENSE-2.0](http://www.apache.org/licenses/LICENSE-2.0))

at your option. This means you can select the license you prefer! This dual-licensing approach is the de-facto standard in the Rust ecosystem and there are very good reasons to include both.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.

## Sponsors
The creation and maintenance of Bevy Transform Gizmo is sponsored by Foresight Mining Software Corporation.

<img src="https://user-images.githubusercontent.com/2632925/151242316-db3455d1-4934-4374-8369-1818daf512dd.png" alt="Foresight Mining Software Corporation" width="480">

