# CSiOS
Jailbreak tweak for injecting CLEO scripts into GTA: SA on iOS. [Video demonstration](https://www.youtube.com/watch?v=6FTkOEV7qnw)

This is **not** a port of the existing tools, but is instead a complete reimplementation for the iOS platform. It attempts to 
function similarly to the Android version of CLEO. 

## Using
This project is in a very early stage of development and is not particularly user-friendly at the moment, but it does work.
* Install a `.deb` archive for this tweak to the device.
* To install a script, place it in `/var/mobile/Documents/CS`.
* Any supporting files (`.fxt` only at the moment) should also be placed there.
* Expect decent results only with CLEO Android (`.csa`) scripts.
* Custom textures are not yet supported.

## Building
Build with CMake. You do not need Theos to build this – Logos is not used anywhere. You do need an iOS SDK though. `CMakeLists.txt` gives the path for Xcode's iOS SDK on macOS, but this may differ from the location of your SDK.

## Known Issues
* CSiOS is **not compatible with the Odyssey jailbreak**, as it uses Saurik's Cydia Substrate. If CoolStar wants to replace 
libraries that work completely fine then that's OK. I will not be adding support for Odyssey.
* Android's `0DD1` opcode (`GetFuncAddrByCStrName`) cannot be implemented because the symbols have been stripped from the iOS version of the game.