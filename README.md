# Magnate
A bevy game for bevy jam 2.

Rotate rubies to light up the runes, but beware that they're inseperarable once touching.

## Controls
Click on the corners of the ruby triangles to rotate them.
Left click rotates counter clockwise, rightclick rotates clockwise.
Once two rubies touch, they will now combine to a single entity that cannot be separated.

The goal is to light up all the runes by moving a ruby onto them.

Press `R` to reload the level or press a number `1`-`9` to load a specific level.

## Level Editor
(This is considered cheating!)
Press `Left Control` + a number `0`-`9` to save the current state as a level.
On PC the levels are saved and loaded from `./levels`. On the web the are stored
in `LocalStorage`. They are somewhat easily editable json files, if you want
to undo or fix a mistake.
Be sure to create and GitHub Issue if you have a good level to share.

Note: If a level is built-in, then loading a level will always load the built-in level
and not the saved one. Built-in Level 0 is garanteed to be empty, the game starts with
level 1.

To place tiles press `A` to select Triangles, `S` for Immovables and `D` for Runes.
Then hold Left Control while clicking on a tile to place it. *Illegal placement is not
restricted, especially placing two tile on the same positon or placing and rotating with
the same click might break things!*
Be sure to save regularly e.g. to save slot 9, because there is no undo.

## Build
Build with `cargo build`.

To build the website, install the wasm toolchain and `wasm-bindgen-cli`.
Remove the `features = ["dynamic"]` from the Cargo.toml for bevy.

```bash
cargo build --release --target wasm32-unknown-unknown
wasm-bindgen --no-typescript --out-dir wasm/ --target web target/wasm32-unknown-unknown/release/game-magnate.wasm
cd wasm
zip -r magnate.zip *
```
