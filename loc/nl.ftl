# Used in the settings menu to show the name of the language.
language-name = Nederlands

### ==== Splash screen ====

# First line at the bottom of the screen.
splash-legal = Copyright © Copyright :copyright: 2020-2022 squ1dd13, AYZM, ODIN, RAiZOK, tharryz, wewewer1. Dit programma valt onder de MIT licentie.

# Second line.
splash-fun = Met plezier gemaakt in engeland. Veel plezier!

# {""} is just an empty string. Leave this empty if you don't want your name shown. Alternatively,
# you can use this to say you made the translation. It will show up on the splash screen after the
# `splash-fun` message.
#
# For example:
#  translator-tag = Translated into English by squ1dd13.
translator-tag = Nederlandse vertaaling door wewewer1#1427

### ==== Updates ====

# PLEASE NOTE: The update menu is currently made from a game menu, and the game fonts do not
# support non-ASCII characters. Please write with ASCII only here. If you can't do that, leave it
# in English for now. I plan to add Unicode support here in the future.

# Displayed at the top of the update screen.
update-prompt-title = Er is een nieuwe versie beschikbaar.

# Message shown on the update screen. { $new_version } will be replaced with the update's version number.
update-prompt-message = CLEO versie { $new_version } is verkrijgbaar. Wil je naar Github gaan om de update te downloaden?

# todo: Add "Yes" and "No" for update menu to localisation files.
# The yes/no options are part of the game, so they're not directly in CLEO's control (yet).

## Release channel settings

update-release-channel-opt-title = Versie Kanaal
update-release-channel-opt-desc = Welke Cleo updates je notificaties voor krijgt. met Beta kan je nieuwe functies eerder gebruiken maar zijn er mogelijk bugs. Updates uitzetten wordt niet aanbevolen.

update-release-channel-opt-disabled = Uit
update-release-channel-opt-stable = Standaard
update-release-channel-opt-disabled = Beta

### ==== Menu ====

# Title for the button at the bottom of the screen that closes the CLEO menu.
menu-close = Sluit

# Title for the options tab.
options-tab-title = Instellingen

### ==== Script menu ====

## Script warning overview

# Shown at the top of the script menus when at least one error has been found in any script. This
# is not shown at all if there are zero scripts with errors in them.
menu-script-warning-overview =
    { $numberOfScriptsWithErrors ->
        [one] Problemen gevonden in één script. Dit script is uitgelicht in oranje.
        *[other] Problemen gevonden in { $numberOfScriptsWithErrors } scripts. Deze scripts zijn uitgelicht in oranje.
    }

# The second line of the warning.
menu-script-see-below = zie hieronder uitgebreidere details.

## Specific script warnings

# The script does things that CLEO doesn't support yet.
script-unimplemented-in-cleo = Dit script probeert functies te gebruiken die nog niet in Cleo zitten.

# The script does things that are not possible on iOS (for system reasons).
script-impossible-on-ios = Dit script gebruikt fincties die niet werken op iOS.

# The script is identical to another script. { $originalScript } will be replaced with the name of
# the script that this one is a duplicate of.
script-duplicate = Duplicaat van { $originalScript }.

# There was an error when checking the script code for problems.
script-check-failed = Script kan niet gescand worden. Geef dit probleem A.U.B. aan in de discord of op Github.

# No problems were found when scanning the script. This is a safe script!
script-no-problems = Geen problemen gevonden met dit script.

## Script status messages

# The script is running normally.
script-running = Aan

# The script is not running.
script-not-running = Niet aan

# The script has been forced to run by the user, even though there are problems with it. This only
# applies to CSA scripts.
script-csa-forced-running = Geforceerd

## Script settings

script-mode-opt-title = Script Modus
script-mode-opt-desc = Veranderd hoe CLEO de scripts laat functioneren, als je problemen hebt met scripts probeer dan deze instellingen te veranderen.

# In "don't break" mode, CLEO won't try to reduce script lag. This might be more stable sometimes.
script-mode-opt-dont-break = Langzaam

# In "break" mode, CLEO will try to reduce script lag caused by long loops in script code.
script-mode-opt-break = Snel

### ==== FPS ====

## FPS lock option

fps-lock-opt-title = PPS Limiet
fps-lock-opt-desc = Maximale plaatjes per seconde in het spel. 30 Plaatjes per seconde is een minder vloeiend beeld, maar het gebruikt minder batterij percentage.

fps-lock-opt-30 = 30 PPS
fps-lock-opt-60 = 60 PPS

## FPS counter option

fps-counter-opt-title = PPS Teller
fps-counter-opt-desc = Zet de plaayjes per seconde teller aan of uit.

fps-counter-opt-hidden = Uit
fps-counter-opt-enabled = Aan

### ==== Cheat system ====

## Menu

cheat-tab-title = Cheats

# Two lines of text shown at the top of the cheats menu.
cheat-menu-warning = Cheats gebruiken kan voor bugs zorgen of zelfs verlies van spel progressie.
cheat-menu-advice = Als je toch cheats wil gebruiken maar ook bang bent om je progressie te verliezen kan je een reservekopie maken van je progressie door in een ander save-slot op te slaan.

## Status messages for cheats

cheat-on = Aan
cheat-off = Uit

# Cheat will be turned on when the menu is closed.
cheat-queued-on = Wachtrij aan

# Cheat will be turned off when the menu is closed.
cheat-queued-off = Wachtrij uit

## Cheat saving option

cheat-transience-opt-title = Cheat onthoudings-systeem
cheat-transience-opt-desc = Veranderd wat het spel doet met de actieve cheats als je het spel afsluit/opnieuw opstart.

cheat-transience-opt-transient = Zet alles uit
cheat-transience-opt-persistent = Onthoud welke aan staan

### ==== Cheat descriptions ====

## Weapon sets
cheat-thugs-armoury = Wapen set 1
cheat-professionals-kit = Wapen set 2
cheat-nutters-toys = Wapen set 3
cheat-weapons-4 = Geef een dildo, minigun en een hitte/nacht-visie bril

## Debug cheats
cheat-debug-mappings = Debug (show mappings)
cheat-debug-tap-to-target = Debug (show tap to target)
cheat-debug-targeting = Debug (show targeting)

## Properly cheating
cheat-i-need-some-help = Geef leven, schild en $250,000
cheat-skip-mission = Sla sommige missies automatisch over

## Superpowers
cheat-full-invincibility = Compleete onschaadbaar
cheat-sting-like-a-bee = Super sterk slaan
cheat-i-am-never-hungry = Nooit hongerig worden
cheat-kangaroo = 10 keer zo hoog springen
cheat-noone-can-hurt-me = Oneindig levens
cheat-man-from-atlantis = Oneindige longcapaciteit

## Social player attributes
cheat-worship-me = Maximaal respect
cheat-hello-ladies = Maximale sexuele attractie

## Physical player attributes
cheat-who-ate-all-the-pies = Maximale dikheid
cheat-buff-me-up = Maximale spiermassa
cheat-max-gambling = Maximale gok vaardigheid
cheat-lean-and-mean = Minimale dikheid en spiermassa
cheat-i-can-go-all-night = Maximaal uithoudingsvermogen

## Player skills
cheat-professional-killer = Hitman vaardigheden op alle wapens
cheat-natural-talent = Maximale voortuig vaardigheden

## Wanted level
cheat-turn-up-the-heat = Sterren gaan met twee omhoog
cheat-turn-down-the-heat = Haal alle sterren weg
cheat-i-do-as-i-please = Zet sterren aantal vast op huidig aantal
cheat-bring-it-on = Zet hoeveeheid sterren naar zes

## Weather
cheat-pleasantly-warm = Zonnig weer
cheat-too-damn-hot = Heel zonnig weer
cheat-dull-dull-day = Bewolkt weer
cheat-stay-in-and-watch-tv = Regenachtig weer
cheat-cant-see-where-im-going = Mistig weer
cheat-scottish-summer = Stormachtig weer
cheat-sand-in-my-ears = Zandstorm

## Time
cheat-clock-forward = Zet de tijd vier uur vooruit
cheat-time-just-flies-by = Snelere tijd
cheat-speed-it-up = Sneler spel
cheat-slow-it-down = Langzamer spel
cheat-night-prowler = Altijd middernacht
cheat-dont-bring-on-the-night = altijd 21:00

## Spawning wearables
cheat-lets-go-base-jumping = Breng een parachute de wereld in
cheat-rocketman = Breng een jetpack de wereld in

## Spawning vehicles

# The vehicle names (in capitals) should not be changed, as they are part of the game. The
# descriptions (in brackets) do need to be translated.
cheat-time-to-kick-ass = breng een Rihno (leger tank) de wereld in
cheat-old-speed-demon = breng een Bloodring Banker (demolition derby car) de wereld in
cheat-tinted-rancher = breng een Rancher met getint glas (twee-deur SUV) de wereld in
cheat-not-for-public-roads = breng een Hotring Racer A (race auto) de wereld in
cheat-just-try-and-stop-me = breng een Hotring Racer B (race auto) de wereld in
cheat-wheres-the-funeral = breng een Romero (lijkwagen) de wereld in
cheat-celebrity-status = breng een Stretch Limousine (limousine) de wereld in
cheat-true-grime = breng een Spawn Trashmaster (vuilniswagen) de wereld in
cheat-18-holes = breng een Caddy (golf karretje) de wereld in
cheat-jump-jet = breng een Hydra (VTOL gevechtsstraaljager) de wereld in
cheat-i-want-to-hover = breng een Vortex (zweefwagen) de wereld in
cheat-oh-dude = breng een Hunter (military attack helicopter) de wereld in
cheat-four-wheel-fun = breng een Quad (quad) de wereld in
cheat-hit-the-road-jack = breng een Tanker met aanhangwagen (tanker truck) de wereld in
cheat-its-all-bull = breng een Dozer (bulldozer) de wereld in
cheat-flying-to-stunt = breng een Stunt Plane (stunt vliegtuig) de wereld in
cheat-monster-mash = breng een Monster Truck (monster truck) de wereld in

## Gang recruitment
cheat-wanna-be-in-my-gang = werf iedereen je gang in en geef ze een pistoon door op ze te richten met een pistool
cheat-noone-can-stop-us = werf iedereen je gang in en geef ze een AK-47 door op ze te richten met een AK-47
cheat-rocket-mayhem = werf iedereen je gang in en geef ze een RPG door op ze te richten met een RPG

## Traffic
cheat-all-drivers-are-criminals = All NPC drivers drive aggressively and have a wanted level
cheat-pink-is-the-new-cool = Roze verkeer
cheat-so-long-as-its-black = Rwart verkeer
cheat-everyone-is-poor = Plattelands verkeer
cheat-everyone-is-rich = Sport verkeer

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
