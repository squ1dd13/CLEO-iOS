### ==== Language settings ====

# Used in the settings menu to show the name of the language.
language-name = English

# Shown when this language has been selected automatically.
language-auto-name = Automatic (English)

# The name of the language setting.
language-setting-title = Language

# The language setting description.
language-setting-desc = The language to use for CLEO. Automatic mode will use your device/game settings. Join the Discord server to add your own language!

### ==== Splash screen ====

# First line at the bottom of the screen.
splash-legal = Copyright © 2020-2023 squ1dd13, AYZM, ODIN, RAiZOK, tharryz, wewewer1. Licenced under the MIT License.

# Second line.
splash-fun = Made with love in the United Kingdom. Have fun!

### ==== Updates ====

# PLEASE NOTE: The update menu is currently made from a game menu, and the game fonts do not
# support non-ASCII characters. Please write with ASCII only here. If you can't do that, leave it
# in English for now. I plan to add Unicode support here in the future.

# Displayed at the top of the update screen.
update-prompt-title = Update Available

# Message shown on the update screen. { $new_version } will be replaced with the update's version number.
update-prompt-message = CLEO version { $new_version } is available. Do you want to go to GitHub to download it?

# todo: Add "Yes" and "No" for update menu to localisation files.
# The yes/no options are part of the game, so they're not directly in CLEO's control (yet).

## Release channel settings

update-release-channel-opt-title = Release Channel
update-release-channel-opt-desc = Which CLEO updates you get notifications for. Alpha gives newer features sooner but might have more bugs. Disabling updates is not recommended.

update-release-channel-opt-disabled = Disabled
update-release-channel-opt-stable = Stable
update-release-channel-opt-alpha = Alpha

### ==== Menu ====

# Title for the button at the bottom of the screen that closes the CLEO menu.
menu-close = Close

# Title for the options tab.
menu-options-tab-title = Options

### ==== Script menu ====

## Script warning overview

# Shown at the top of the script menus when at least one error has been found in any script. This
# is not shown at all if there are zero scripts with errors in them.
menu-script-warning-overview =
    { $num_scripts_with_errors ->
        [one] Found problems in one script. This script is highlighted in orange.
        *[other] Found problems in { $num_scripts_with_errors } scripts. These scripts are highlighted in orange.
    }

# The second line of the warning.
menu-script-see-below = See below for further details.

## Specific script warnings

# The script does things that CLEO doesn't support yet.
script-unimplemented-in-cleo = Uses features currently unsupported by CLEO iOS.

# The script does things that are not possible on iOS (for system reasons).
script-impossible-on-ios = Uses some code that won't work on iOS.

# The script is identical to another script. { $original_script } will be replaced with the name of
# the script that this one is a duplicate of.
script-duplicate = Duplicate of { $original_script }.

# There was an error when checking the script code for problems.
script-check-failed = Unable to scan script. Please report this as a bug on GitHub or Discord.

# No problems were found when scanning the script. This is a safe script!
script-no-problems = No problems detected.

# Formats for script names in the menu.
script-csa-row-title = { $script_name }
script-csi-row-title = { $script_name }

## Script status messages

# The script is running normally.
script-running = Running

# The script is not running.
script-not-running = Not running

# The script has been forced to run by the user, even though there are problems with it. This only
# applies to CSA scripts.
script-csa-forced-running = Forced

## Script settings

script-mode-opt-title = Script Processing Mode
script-mode-opt-desc = Changes how CLEO processes script code. Try changing this if you're experiencing issues with scripts.

# In "don't break" mode, CLEO won't try to reduce script lag. This might be more stable sometimes.
script-mode-opt-dont-break = Slow

# In "break" mode, CLEO will try to reduce script lag caused by long loops in script code.
script-mode-opt-break = Fast

### ==== FPS ====

## FPS lock option

fps-lock-opt-title = FPS Limit
fps-lock-opt-desc = The maximum framerate the game will run at. 30 FPS looks worse but saves battery.

fps-lock-opt-30 = 30 FPS
fps-lock-opt-60 = 60 FPS

## FPS counter option

fps-counter-opt-title = FPS Counter
fps-counter-opt-desc = Enables or disables the on-screen FPS counter.

fps-counter-opt-hidden = Disabled
fps-counter-opt-enabled = Enabled

### ==== Cheat system ====

## Menu

cheat-tab-title = Cheats

# Two lines of text shown at the top of the cheats menu.
cheat-menu-warning = Using cheats can lead to crashes and possibly a loss of game progress.
cheat-menu-advice = If you don't want to risk breaking your save, back up your progress to a different slot first.

## Status messages for cheats

cheat-on = On
cheat-off = Off

# Cheat will be turned on when the menu is closed.
cheat-queued-on = Queued on

# Cheat will be turned off when the menu is closed.
cheat-queued-off = Queued off

# Formats for cheat codes in the menu.
cheat-code-row-title = { $cheat_code }
cheat-no-code-title = ???

## Cheat saving option

cheat-transience-opt-title = Cheat Saving Mode
cheat-transience-opt-desc = Controls how cheats are managed when reloading/restarting the game.

cheat-transience-opt-transient = Reset all
cheat-transience-opt-persistent = Save states

### ==== Cheat descriptions ====

## Weapon sets
cheat-thugs-armoury = Weapon set 1
cheat-professionals-kit = Weapon set 2
cheat-nutters-toys = Weapon set 3
cheat-weapons-4 = Give dildo, minigun and thermal/night-vision goggles

## Debug cheats
cheat-debug-mappings = Debug (show mappings)
cheat-debug-tap-to-target = Debug (show tap to target)
cheat-debug-targeting = Debug (show targeting)

## Properly cheating
cheat-i-need-some-help = Give health, armour and $250,000
cheat-skip-mission = Skip to completion on some missions

## Superpowers
cheat-full-invincibility = Full invincibility
cheat-sting-like-a-bee = Super punches
cheat-i-am-never-hungry = Player never gets hungry
cheat-kangaroo = 10x jump height
cheat-noone-can-hurt-me = Infinite health
cheat-man-from-atlantis = Infinite lung capacity

## Social player attributes
cheat-worship-me = Maximum respect
cheat-hello-ladies = Maximum sex appeal

## Physical player attributes
cheat-who-ate-all-the-pies = Maximum fat
cheat-buff-me-up = Maximum muscle
cheat-max-gambling = Maximum gambling skill
cheat-lean-and-mean = Minimum fat and muscle
cheat-i-can-go-all-night = Maximum stamina

## Player skills
cheat-professional-killer = Hitman level for all weapons
cheat-natural-talent = Maximum vehicle skills

## Wanted level
cheat-turn-up-the-heat = Increase wanted level by two stars
cheat-turn-down-the-heat = Clear wanted level
cheat-i-do-as-i-please = Lock wanted level to current value
cheat-bring-it-on = Six-star wanted level

## Weather
cheat-pleasantly-warm = Sunny weather
cheat-too-damn-hot = Very sunny weather
cheat-dull-dull-day = Overcast weather
cheat-stay-in-and-watch-tv = Rainy weather
cheat-cant-see-where-im-going = Foggy weather
cheat-scottish-summer = Stormy weather
cheat-sand-in-my-ears = Sandstorm

## Time
cheat-clock-forward = Advance clock by 4 hours
cheat-time-just-flies-by = Faster time
cheat-speed-it-up = Faster gameplay
cheat-slow-it-down = Slower gameplay
cheat-night-prowler = Always midnight
cheat-dont-bring-on-the-night = Always 9 p.m.

## Spawning wearables
cheat-lets-go-base-jumping = Spawn parachute
cheat-rocketman = Spawn jetpack

## Spawning vehicles

# The vehicle names (in capitals) should not be changed, as they are part of the game. The
# descriptions (in brackets) do need to be translated.
cheat-time-to-kick-ass = Spawn Rhino (army tank)
cheat-old-speed-demon = Spawn Bloodring Banger (demolition derby car)
cheat-tinted-rancher = Spawn Rancher with tinted windows (two-door SUV)
cheat-not-for-public-roads = Spawn Hotring Racer A (racing car)
cheat-just-try-and-stop-me = Spawn Hotring Racer B (racing car)
cheat-wheres-the-funeral = Spawn Romero (hearse)
cheat-celebrity-status = Spawn Stretch Limousine (limousine)
cheat-true-grime = Spawn Trashmaster (garbage truck/bin lorry)
cheat-18-holes = Spawn Caddy (golf cart)
cheat-jump-jet = Spawn Hydra (VTOL attack jet)
cheat-i-want-to-hover = Spawn Vortex (hovercraft)
cheat-oh-dude = Spawn Hunter (military attack helicopter)
cheat-four-wheel-fun = Spawn Quad (quadbike/ATV/four-wheeler)
cheat-hit-the-road-jack = Spawn Tanker and trailer (tanker truck)
cheat-its-all-bull = Spawn Dozer (bulldozer)
cheat-flying-to-stunt = Spawn Stunt Plane (stunt plane)
cheat-monster-mash = Spawn Monster Truck (monster truck)

## Gang recruitment
cheat-wanna-be-in-my-gang = Recruit anyone into your gang and give them a pistol by aiming a pistol at them
cheat-noone-can-stop-us = Recruit anyone into your gang and give them an AK-47 by aiming an AK-47 at them
cheat-rocket-mayhem = Recruit anyone into your gang and give them an RPG by aiming an RPG at them

## Traffic
cheat-all-drivers-are-criminals = All NPC drivers drive aggressively and have a wanted level
cheat-pink-is-the-new-cool = Pink traffic
cheat-so-long-as-its-black = Black traffic
cheat-everyone-is-poor = Rural traffic
cheat-everyone-is-rich = Sports car traffic

## Pedestrians
cheat-rough-neighbourhood = Give player golf club and make pedestrians riot
cheat-stop-picking-on-me = Pedestrians attack the player
cheat-surrounded-by-nutters = Give pedestrians weapons
cheat-blue-suede-shoes = All pedestrians are Elvis Presley
cheat-attack-of-the-village-people = Pedestrians attack the player with guns and rockets
cheat-only-homies-allowed = Gang members everywhere
cheat-better-stay-indoors = Gangs control the streets
cheat-state-of-emergency = Pedestrians riot
cheat-ghost-town = Reduced live traffic and no pedestrians

## Themes
cheat-ninja-town = Triad theme
cheat-love-conquers-all = Pimp theme
cheat-lifes-a-beach = Beach party theme
cheat-hicksville = Rural theme
cheat-crazy-town = Carnival theme

## General vehicle cheats
cheat-all-cars-go-boom = Explode all vehicles
cheat-wheels-only-please = Invisible vehicles
cheat-sideways-wheels = Cars have sideways wheels
cheat-speed-freak = All cars have nitro
cheat-cool-taxis = Taxis have hydraulics and nitro

## Vehicle cheats for the player
cheat-chitty-chitty-bang-bang = Flying cars
cheat-cj-phone-home = Very high bunny hops
cheat-touch-my-car-you-die = Destroy other vehicles on collision
cheat-bubble-cars = Cars float away when hit
cheat-stick-like-glue = Improved suspension and handling
cheat-dont-try-and-stop-me = Traffic lights are always green
cheat-flying-fish = Flying boats

## Weapon usage
cheat-full-clip = Everyone has unlimited ammunition
cheat-i-wanna-driveby = Full weapon control in vehicles

## Player effects
cheat-goodbye-cruel-world = Suicide
cheat-take-a-chill-pill = Adrenaline effects
cheat-prostitutes-pay = Prostitutes pay you

## Miscellaneous
cheat-xbox-helper = Adjust stats to be close to getting Xbox achievements

## Pointless cheats

# Tells the user that a cheat will ALWAYS crash their game.
-cheat-crash-warning = CRASHES!

cheat-slot-melee = { -cheat-crash-warning } Melee slot
cheat-slot-handgun = { -cheat-crash-warning } Handgun slot
cheat-slot-smg = { -cheat-crash-warning } SMG slot
cheat-slot-shotgun = { -cheat-crash-warning } Shotgun slot
cheat-slot-assault-rifle = { -cheat-crash-warning } Assault rifle slot
cheat-slot-long-rifle = { -cheat-crash-warning } Long rifle slot
cheat-slot-thrown = { -cheat-crash-warning } Thrown weapon slot
cheat-slot-heavy = { -cheat-crash-warning } Heavy artillery slot
cheat-slot-equipment = { -cheat-crash-warning } Equipment slot
cheat-slot-other = { -cheat-crash-warning } Other slot

cheat-predator = Does nothing
