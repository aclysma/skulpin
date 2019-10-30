# skulpin

Skia + Vulkan = Skulpin

This crate provides an easy option for drawing hardware-accelerated 2D by combining vulkan and skia. (And a dash of 
winit!)

TODO: Build Status and crates.io badges

This crate mainly depends on:
 * [ash](https://github.com/MaikKlein/ash) - Vulkan bindings for Rust
 * [skia-safe](https://github.com/rust-skia/rust-skia) - [Skia](https://skia.org) bindings for Rust
 * [winit](https://github.com/rust-windowing/winit) - Cross-platform window handling
 
## Usage

Currently there are two main ways to use this library.
 * [app](blob/master/examples/skuplin_app.rs) - Implement the AppHandler trait and launch the app. It's simple but not as flexible.
 * [renderer_only](blob/master/examples/renderer_only.rs) - You manage the window and event loop yourself. Then add the renderer to draw to it.

## Status

For now this is a proof-of-concept. I think there is desire for a simple entry point to drawing on the screen, and that
this approach can provide a good balance of performance, features, and ease-of-use for many applications.

Flutter, Google's new UI framework that targets 60+ FPS on mobile devices, uses a Skia + Vulkan stack. So I expect this
type of usage to be maintained and improved in the upstream libraries.

## License

Licensed under either of

* Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
* MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally
submitted for inclusion in the work by you, as defined in the Apache-2.0
license, shall be dual licensed as above, without any additional terms or
conditions.

See [LICENSE-APACHE](LICENSE-APACHE) and [LICENSE-MIT](LICENSE-MIT).
