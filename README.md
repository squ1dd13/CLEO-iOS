# CLEO iOS (Rust)
This is an attempt at a rewrite of CLEO in Rust.

CLEO v2 is (hopefully) going to be a more general-purpose modding solution for the iOS platform, with planned features
that will try to eliminate the need for a computer when installing various types of mods.

## Planned Features
* Basic CLEO features
    * CSA/CSI script loading
    * Android-style mod menu
    * Support for FXT localisations
* IMG editing/hooking for custom models
* Sound modding
* Texture modding
* Cheat menu (for using cheats without a keyboard)

A possible change would be to integrate with the game's button control
system rather than relying on Android-style swipe gestures for showing menus.

## Thanks to...
* [Seemann](https://github.com/x87) for offering support and info, and for letting this project officially 
be a part of [CLEO](http://cleo.li/).
* [Alexander Blade](http://www.dev-c.com/) for creating CLEO Android, and for publishing information on 
his Android-specific opcodes ([here](https://gtaforums.com/topic/663125-android-cleo-android/)).
* [DK22Pac](https://github.com/DK22Pac) and all the others who have contributed to 
[plugin-sdk](https://github.com/DK22Pac/plugin-sdk), which was very helpful for creating game structures.
* The GTA modding community in general for doing so much of the research which made CLEO iOS possible.