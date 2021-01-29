# Zinc - Zinc Is Not CLEO

Jailbreak tweak for injecting CLEO scripts into GTA: SA on
iOS. [Video demonstration](https://www.youtube.com/watch?v=6FTkOEV7qnw)

This is **not** a port of the existing tools (the CLEO family), but is instead a complete reimplementation for the iOS
platform. It attempts to function similarly to the Android version of CLEO.

## Using

This project is in a very early stage of development and is not particularly user-friendly at the moment, but it does
work.

* Install a `.deb` archive for this tweak to the device.
* To install a script, place it in `/var/mobile/Documents/CS`.
* Any supporting files (`.fxt` only at the moment) should also be placed there.
* Expect decent results only with CLEO Android (`.csa`) scripts.
* Custom textures are not yet supported.

### What does it do?

* Loads scripts and injects them into the game.
* Loads localisation files (.fxt).
* Mimics CLEO Android's touch zone control system.

### What does it *not* do?

* Provide a script menu ("mod menu") such as that of CLEO Android.
* Load PC scripts (some may work, but don't expect them to).

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

## Known Issues

* Zinc is **not compatible with the Odyssey jailbreak**, as it uses Saurik's Cydia Substrate.
* Android's `0DD1` opcode (`GetFuncAddrByCStrName`) cannot be implemented because the symbols have been stripped from
  the iOS version of the game.