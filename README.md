# CLEO iOS
<!-- I love badges :D -->
[![Discord](https://img.shields.io/discord/767478053139775528?color=7289DA&label=DISCORD&style=for-the-badge)](https://discord.gg/cXwkTUasJU) ![Downloads](https://img.shields.io/github/downloads/squ1dd13/CLEO-iOS/total?style=for-the-badge) ![Licence](https://img.shields.io/github/license/squ1dd13/CLEO-iOS?style=for-the-badge)

_**Note:** This branch (`main`) is for the current version of CLEO iOS, which is written in Rust. For the original C++ version, see the [`c++` branch](https://github.com/Squ1dd13/CLEO-iOS/tree/c+%2B)._

**[Join the Discord server for support, info and script suggestions.](https://discord.gg/cXwkTUasJU)**

## Completed Features
* File support
  * CSA scripts
  * CSI scripts
  * FXT language extensions
* Mod loading
  * Archive (`.img`) file modding
  * Automatic file replacement
* Cheat menu
* 60 FPS

## Planned Features
* Sound modding
* Texture modding
* Support for other iOS games (III and VC)

## Installation
1. Download the .deb file from the [latest release](https://github.com/squ1dd13/CLEO-iOS/releases/latest).
2. Install the .deb using a tool like `dpkg` or through an app like Filza.

When the game is opened, if it has been at least five hours since the last check, CLEO will check to see if there is
a new release available. If there is one, a message will be displayed with the option to go to the [release](https://github.com/squ1dd13/CLEO-iOS/releases/latest).

To update CLEO, simply follow the above steps with the newer .deb. The package manager will handle the update,
and your mods will remain in place.

## Scripts
### Installation
1. Find a script that you want to use.
2. Navigate to the GTA app's data folder.
This will be a folder in `/var/mobile/Containers/Data/Application`. If you are using Filza, you are looking for a folder displayed as `GTA: SA`. If not, the folder name will be the app's UUID. (Therefore it's easiest to use Filza for this.)
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

## File Modding
On top of script modding, CLEO is also capable of various mod loading tasks.

### File Swapping
While it is possible to modify game files (inside `gta3sa.app`) directly, this is not the best
way to change files. This is because you have to keep track of what you have changed and make 
sure you have backups to avoid having to reinstall the game due to broken files. CLEO allows
different files to be loaded while leaving the game's files unchanged.

1. If you do not have it already, make a folder named "Replace" inside your CLEO directory.
2. Find a mod which replaces game files (such as `handling.cfg`, `gta3.img`, or any other files
that are found inside `gta3sa.app`).
3. Put the replacement files inside the "Replace" folder. To replace `handling.cfg`, the files 
would look like this:
    ```
    CLEO
      ...
      Replace
        handling.cfg
      ...
    ```
4. Launch the game. Whenever the game tries to load the files from `gta3sa.app`, it will instead
load from the "Replace" folder.

If you want to remove a file swap mod, just delete the files inside "Replace". The game will load
from `gta3sa.app` instead.

### IMG Archives
To replace files inside an IMG archive:
1. Make a folder in your CLEO directory with the same name as the archive (for example, `gta3.img`).
2. Find a mod with replacement files that need to go into the archive (often, these are `.dff` files,
but there are others too).
3. Get the files from the mod and put them inside the folder you made in step 1. For example, a mod
that replaces the `clover.dff` file inside `gta3.img` would need a layout like this:
    ```
    CLEO
      ...
      gta3.img
        clover.dff
      ...
    ```
4. Launch the game. The replacement files will take effect.

To reverse the changes, just delete the files from the `.img` folder
you made (or delete the entire `.img` folder if you want to remove
all replacements for that archive).

### Combined
With both of the examples from above, your CLEO directory would look like this:
```
CLEO
  ...
  Replace
    handling.cfg
  ...
  gta3.img
    clover.dff
  ...
```

Please note that you do _**not**_ need a folder named "Replace" for replacing files _**inside**_ the archive.
The "Replace" folder is only for replacing entire files that you see inside `gta3sa.app`.

## Cheats
* A list of cheats can be found in the CLEO menu. It can be accessed by selecting the "Cheats" tab
at the top of the menu, to the right of the "Scripts" tab.
* **Cheats can break save files and/or crash the game**, so be careful when using them (especially
if using cheats that do not have cheat codes). **It is strongly recommended that you back up your save in a different slot** 
before using cheats.
* Some cheats can be enabled or disabled (such as causing all new cars to be a certain colour), while
others are single events (such as giving the player money). Both types of cheat can be activated by
tapping on them in the menu.

## What do I do if I need help/found a bug/have a suggestion?
Please don't keep quiet about it! 

If you have a suggestion or you think you've found a bug, either 
create an issue on GitHub or join the [Discord server](https://discord.gg/cXwkTUasJU) to
discuss it there. The more bugs that get fixed, the better CLEO will
become for you and everybody else!

If you need help with something, please join the Discord server so
we can assist you there. Also, don't forget that you can read
through the step-by-step instructions on this page if you can't
remember how to do something.

## Thanks to...
* [Seemann](https://github.com/x87) for offering support and info, and for letting this project officially 
be a part of [CLEO](http://cleo.li/).
* [Alexander Blade](http://www.dev-c.com/) for creating CLEO Android, and for publishing information on 
his Android-specific opcodes [here](https://gtaforums.com/topic/663125-android-cleo-android/).
* [DK22Pac](https://github.com/DK22Pac) and all the others who have contributed to 
[plugin-sdk](https://github.com/DK22Pac/plugin-sdk), which has been very helpful for creating game structures.
* All those who have contributed to the [gtasa-reversed](https://github.com/codenulls/gta-reversed) project, which has been a valuable
resource for building my understanding of some of the more complex systems that are in both the PC and iOS versions.
* oliver#1219 for gifting me the other GTA games to help with getting CLEO working on those too.
* Members of the CLEO iOS [Discord server](https://discord.gg/cXwkTUasJU) for reporting bugs, helping investigate them and testing fixes.
* The GTA modding community in general for doing so much of the research which made CLEO iOS possible!
