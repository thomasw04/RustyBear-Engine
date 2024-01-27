# RustyBear-Engine

[![Verify and Tests](https://github.com/thomasw04/RustyBear-Engine/actions/workflows/verify.yml/badge.svg)](https://github.com/thomasw04/RustyBear-Engine/actions)
[![Release](https://github.com/thomasw04/RustyBear-Engine/actions/workflows/release.yml/badge.svg)](https://github.com/thomasw04/RustyBear-Engine/actions)

My very first Game-Engine written in Rust + wgpu. The plan is to use it at Ludum Dare 54.
It is more or less an eductational project. I just like building Game-Engines.

For the asset management of the engine see [thomasw04/what](https://github.com/thomasw04/what)

## Featues (Current)
- Native Metal (macOS), Vulkan (Linux + Windows) and DirectX 12 (Windows) support. Thanks to wgpu.
- Thus runs out of the box on all three platforms.
- Easy to use event/input handling system + game controller support.
- 2D renderering, supporting animations and ldtk world loading.
- Rust scripting + hecs integration.
- 3d skybox renderer.
- Pipeline hashing.
- Egui for simple gui creation.
- WASM support. (Currently broken)

## Main Features (Planned: Sorted from highest priority)
- 2D/3D renderer.
- 3D audio engine.
- LuaJit/Squirrel scripting language.
- Physics engine.

## Build from source

1. Clone the repo 
2. Install rustup, gcc and (libudev-dev, libasound2-dev only on linux) 
3. Run ```cargo run --release --example hello_world```
4. Profit :)

## OR use the prebuilt binaries
Note: The macOS binary is for x86-64 only. For M1/M2 please build from source.

## Contribute
- I recommend using vscode + (rust-analyzer, CodeLLDB) extensions.
- Feel free to open a PR or Issue if you have something to contribute.


