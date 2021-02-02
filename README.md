# Zinc - Zinc Is Not CLEO
<!-- Badges are fun -->
[![forthebadge](https://forthebadge.com/images/badges/made-with-c-plus-plus.svg)](https://forthebadge.com) [![forthebadge](https://forthebadge.com/images/badges/built-with-love.svg)](https://forthebadge.com)

Jailbreak tweak for injecting CLEO scripts into GTA: SA on
iOS. [Video demonstration](https://www.youtube.com/watch?v=6FTkOEV7qnw)

This is **not** a port of the existing tools (the CLEO family), but is instead a from-scratch implementation for the iOS
platform. It attempts to function similarly to the Android version of CLEO.

## Using

* Install a `.deb` archive for this tweak to the device.
* To install a script, place it in `/var/mobile/Documents/CS`.
* Any supporting files (`.fxt` only at the moment) should also be placed there.
* Expect decent results only with CLEO Android (`.csa` or `.csi`) scripts.
* Custom textures are not yet supported.

Please carefully read any `readme.txt` (or similarly named) files packaged with scripts
in order to find out what you have to do to use the mod. To access the mod menu, you must
swipe down in the middle of the screen. You can dismiss it either by selecting a script
to run, or by tapping outside of the menu.

### What does it do?

* Loads scripts and injects them into the game.
* Loads localisation files (.fxt).
* Mimics CLEO Android's touch zone control system.
* Provide a script menu ("mod menu") similar to CLEO Android and allows scripts to be loaded mid-game.

### What does it *not* do?

* Load PC scripts (some may work, but don't expect most to). 
* Guarantee that all scripts (Android or not) will work.

## Known Issues

* Zinc is **not currently compatible with the Odyssey jailbreak**, as it relies on Saurik's Cydia Substrate.

* Not all instructions are implemented.*

* Any script that relies on custom assembly code *will never work*. This is because the Android version of
  the game is exclusively 32-bit, and while there are both 32 and 64-bit versions of the game for iOS,
  Zinc only supports 64-bit devices.

*There are several instructions added in CLEO Android that cannot be used with Zinc, either because they don't
make sense for iOS (i.e. they relate to Android-only system features) or because the iOS game cannot provide
enough information for them to be implemented - function names are not present in the iOS version, for example,
so instructions that aim to obtain function information by name cannot be implemented (realistically). There
are also some instructions that I don't know enough about to implement (please see **Contributing**).

## Contributing
Please feel free to contribute code, ideas, tutorials or anything else to the Zinc project!

Even if you can't contribute to the code directly, I'm looking specifically for information on
the Android opcodes `0xDD0`-`0xDDD` because they are not implemented in Zinc and thus many scripts
that use them will be broken.

## Building

* Configure with CMake and build with Clang from an iOS toolchain.
* You will need to specify the path to your iOS toolchain's `bin` directory in `CMakeLists.txt`, along with the path to
  your SDK.
* To build a `.deb`, you need to run `meta/package.sh` with the single argument `Zinc`.
    * You will need to change the path to `ldid` to match your toolchain. This does not apply if you have the `codesign`
      command (e.g. if you're on macOS).
    * If you do not use `zsh` you will have to change the first line of `package.sh`:

        ```shell
        #!/usr/bin/zsh
        ```
      to
        ```shell
        #!/bin/bash
        ```
      or whatever the path to your shell is.