# CLEO iOS
<!-- I love badges :D -->
[![Discord](https://img.shields.io/discord/767478053139775528?color=7289DA&label=DISCORD&style=for-the-badge)](https://discord.gg/cXwkTUasJU) ![Downloads](https://img.shields.io/github/downloads/squ1dd13/CLEO-iOS/total?style=for-the-badge) 

_**Note:** This branch (`main`) is for the current version of CLEO iOS, which is written in Rust. For the original C++ version, see the [`c++` branch](https://github.com/Squ1dd13/CLEO-iOS/tree/c+%2B)._

**[Join the Discord server for support, info and script suggestions.](https://discord.gg/cXwkTUasJU)**

## Completed Features
* CSA script loading
* CSI script loading
* FXT loading
* Cheat menu

## Planned Features
* IMG editing/hooking for custom models
* Sound modding
* Texture modding

A possible change would be to integrate with the game's button control
system rather than relying on Android-style swipe gestures for showing menus.

## Installation
1. Download the .deb file from the [latest release](https://github.com/squ1dd13/CLEO-iOS/releases/latest).
2. Install the .deb using a tool like `dpkg` or through an app like Filza.

## Mods
### Installation
1. Find a script that you want to use.
2. Navigate to the GTA app's data folder.
This will be a folder in `/var/mobile/Containers/Data/Application`. If you are using Filza, you are looking for a folder displayed as `GTA: SA`. If not, the folder name will be the app's UUID. (Therefore, it's easiest to use Filza for this.)
3. Open the `Documents` folder inside the app data folder.
4. Create a folder named `CLEO`. It may be a good idea to bookmark
this folder so you can find it again later easily.
5. Put any `.csi`, `.csa` or `.fxt` files from the script into the
`CLEO` folder. Any other files will be ignored, so you can add those
if you don't want to lose them. If you want, you can create more
 folders inside the `CLEO` folder to organise your mods: CLEO will
 look inside these too.

### Use
* Any `.csi` files will be presented in the CLEO menu, which can be invoked by
swiping down on the screen. These scripts can be activated by tapping them, at which
point they will run until they choose to exit.
* `.csa` scripts start as soon as the game loads, and typically do not exit until the
game does. **You should read any "readme" files which come with the script to find out how to interact with the mod.**
* Some scripts require certain touch combinations in "touch zones". These are nine 
areas on the screen which a script can check to see if you are touching. A numbered
diagram of these zones can be found [here](https://3.bp.blogspot.com/--yB8v3cBRyg/U9iO-NyyXPI/AAAAAAAAAJQ/FeGJI47KbYA/s1600/EC3B.jpg).

## Thanks to...
* [Seemann](https://github.com/x87) for offering support and info, and for letting this project officially 
be a part of [CLEO](http://cleo.li/).
* [Alexander Blade](http://www.dev-c.com/) for creating CLEO Android, and for publishing information on 
his Android-specific opcodes ([here](https://gtaforums.com/topic/663125-android-cleo-android/)).
* [DK22Pac](https://github.com/DK22Pac) and all the others who have contributed to 
[plugin-sdk](https://github.com/DK22Pac/plugin-sdk), which was very helpful for creating game structures.
* Members of the CLEO iOS [Discord server](https://discord.gg/cXwkTUasJU) for reporting bugs, helping investigate them and testing fixes.
* The GTA modding community in general for doing so much of the research which made CLEO iOS possible.