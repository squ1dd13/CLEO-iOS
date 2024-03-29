# Used in the settings menu to show the name of the language.
language-name = Nederlands

# Shown when this language has been selected automatically.
language-auto-name = Automatisch ({ language-name })

# The name of the language setting.
language-opt-title = Taal

# The language setting description.
language-opt-desc = De taal waar in je CLEO wil gebruiken. Automatische modus gaat uit van je systeem taal. Voeg je eige taal toe in de Discord!

### ==== Splash screen ====

# First line at the bottom of the screen.
splash-legal = Copyright © 2020-2023 { $copyright_names }. Dit programma valt onder de MIT licentie.

# Second line.
splash-fun = Met plezier gemaakt in engeland. Nederlandse vertaaling door wewewer1#1427. Veel plezier!

### ==== Updates ====

# PLEASE NOTE: The update menu is currently made from a game menu, and the game fonts do not
# support non-ASCII characters. Please write with ASCII only here. If you can't do that, leave it
# in English for now. I plan to add Unicode support here in the future.

# Displayed at the top of the update screen.
update-prompt-title = Er is een nieuwe CLEO versie beschikbaar.

# Message shown on the update screen. { $new_version } will be replaced with the update's version number.
update-prompt-message = CLEO versie { $new_version } is verkrijgbaar. Wil je naar Github gaan om de update te downloaden?

# todo: Add "Yes" and "No" for update menu to localisation files.
# The yes/no options are part of the game, so they're not directly in CLEO's control (yet).

## Release channel settings

update-release-channel-opt-title = Versie Kanaal
update-release-channel-opt-desc = Welke CLEO updates je notificaties voor krijgt. met Beta kan je nieuwe functies eerder gebruiken maar zijn er mogelijk bugs. Updates uitzetten wordt niet aanbevolen.

update-release-channel-opt-disabled = Uit
update-release-channel-opt-stable = Standaard
update-release-channel-opt-alpha = Beta

### ==== Menu ====

# Title for the button at the bottom of the screen that closes the CLEO menu.
menu-close = Sluit

# Title for the options tab.
menu-options-tab-title = Instellingen

## Menu gesture settings

menu-gesture-opt-title = Menu Gebaar
menu-gesture-opt-desc = De vinger beweging nodig om het CLEO menu te laten verschijnen.

# A single motion where one finger moves quickly down the screen.
menu-gesture-opt-one-finger-swipe = Een vinger naar beneden vegen

# A single swipe (as above) but with two fingers at the same time instead of just one.
menu-gesture-opt-two-finger-swipe = Twee vingers naar beneden vegen

# A short tap on the screen with two fingers at once.
menu-gesture-opt-two-finger-tap = Twee vingers aanraken

# A short tap on the screen with three fingers at once.
menu-gesture-opt-three-finger-tap = Drie vingers aanraken

### ==== Script menu ====

## Script warning overview

# Shown at the top of the script menus when at least one error has been found in any script. This
# is not shown at all if there are zero scripts with errors in them.
menu-script-warning-overview =
    { $num_scripts_with_errors ->
        [one] Problemen gevonden in één script. Dit script is uitgelicht in oranje.
        *[other] Problemen gevonden in { $num_scripts_with_errors } scripts. Deze scripts zijn uitgelicht in oranje.
    }

# The second line of the warning.
menu-script-see-below = Zie hieronder uitgebreidere details.

menu-script-csa-tab-title = CSA
menu-script-csi-tab-title = CSI

## Specific script warnings

# The script does things that CLEO doesn't support yet.
script-unimplemented-in-cleo = Dit script probeert functies te gebruiken die nog niet in CLEO zitten.

# The script does things that are not possible on iOS (for system reasons).
script-impossible-on-ios = Dit script gebruikt functies die niet werken op iOS.

# The script is identical to another script. { $original_script } will be replaced with the name of
# the script that this one is a duplicate of.
script-duplicate = Duplicaat van { $original_script }.

# There was an error when checking the script code for problems.
script-check-failed = Script kan niet gescand worden. Geef dit probleem A.U.B. aan in de discord server of op Github.

# No problems were found when scanning the script. This is a safe script!
script-no-problems = Geen problemen gevonden met dit script.

# Formats for script names in the menu.
script-csa-row-title = { $script_name }
script-csi-row-title = { $script_name }

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
fps-lock-opt-desc = Maximale plaatjes per seconde in het spel. 30 Plaatjes per seconde is een minder vloeiend beeld, maar het gebruikt minder batterij.

fps-lock-opt-30 = 30 PPS
fps-lock-opt-60 = 60 PPS

## FPS counter option

fps-counter-opt-title = PPS Teller
fps-counter-opt-desc = Zet de plaatjes per seconde teller aan of uit.

fps-counter-opt-hidden = Uit
fps-counter-opt-enabled = Aan

### ==== Cheat system ====

## Menu

cheat-tab-title = Cheats

# Two lines of text shown at the top of the cheats menu.
cheat-menu-warning = Cheats gebruiken kan voor bugs zorgen of zelfs verlies van spel progressie.
  Als je cheats wil gebruiken maar bang bent om je progressie te verliezen sla dan op in een ander save-slot.

## Status messages for cheats

cheat-on = Aan
cheat-off = Uit

# Cheat will be turned on when the menu is closed.
cheat-queued-on = Wachtrij aan

# Cheat will be turned off when the menu is closed.
cheat-queued-off = Wachtrij uit

# Formats for cheat codes in the menu.
cheat-code-row-title = { $cheat_code }
cheat-no-code-title = ???

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
cheat-i-am-never-hungry = Nooit hongerig zijn
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
cheat-i-do-as-i-please = Zet het aantal sterren vast op huidig aantal
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
cheat-dont-bring-on-the-night = Altijd 21:00

## Spawning wearables
cheat-lets-go-base-jumping = Breng een parachute de wereld in
cheat-rocketman = Breng een jetpack de wereld in

## Spawning vehicles

# The vehicle names (in capitals) should not be changed, as they are part of the game. The
# descriptions (in brackets) do need to be translated.
cheat-time-to-kick-ass = Breng een Rihno (leger tank) de wereld in
cheat-old-speed-demon = Breng een Bloodring Banker (demolition derby car) de wereld in
cheat-tinted-rancher = Breng een Rancher met getint glas (twee-deur SUV) de wereld in
cheat-not-for-public-roads = Breng een Hotring Racer A (race auto) de wereld in
cheat-just-try-and-stop-me = Breng een Hotring Racer B (race auto) de wereld in
cheat-wheres-the-funeral = Breng een Romero (lijkwagen) de wereld in
cheat-celebrity-status = Breng een Stretch Limousine (limousine) de wereld in
cheat-true-grime = Breng een Spawn Trashmaster (vuilniswagen) de wereld in
cheat-18-holes = Breng een Caddy (golf karretje) de wereld in
cheat-jump-jet = Breng een Hydra (VTOL gevechtsstraaljager) de wereld in
cheat-i-want-to-hover = Breng een Vortex (zweefwagen) de wereld in
cheat-oh-dude = Breng een Hunter (militaire aanvals helicopter) de wereld in
cheat-four-wheel-fun = Breng een Quad (quad) de wereld in
cheat-hit-the-road-jack = Breng een Tanker met aanhangwagen (tanker truck) de wereld in
cheat-its-all-bull = Breng een Dozer (bulldozer) de wereld in
cheat-flying-to-stunt = Breng een Stunt Plane (stunt vliegtuig) de wereld in
cheat-monster-mash = Breng een Monster Truck (monster truck) de wereld in

## Gang recruitment
cheat-wanna-be-in-my-gang = Werf iedereen je gang in en geef ze een pistoon door op ze te richten met een pistool
cheat-noone-can-stop-us = Werf iedereen je gang in en geef ze een AK-47 door op ze te richten met een AK-47
cheat-rocket-mayhem = Werf iedereen je gang in en geef ze een RPG door op ze te richten met een RPG

## Traffic
cheat-all-drivers-are-criminals = Al het NPC verkeer rijd agressief en heeft de politie achter zich aan
cheat-pink-is-the-new-cool = Roze verkeer
cheat-so-long-as-its-black = Rwart verkeer
cheat-everyone-is-poor = Plattelands verkeer
cheat-everyone-is-rich = Sport verkeer

## Pedestrians
cheat-rough-neighbourhood = Geef de speler een golfclub en zorg ervoor dat voetgangers in opstand komen
cheat-stop-picking-on-me = Voetgangers vallen de speler aan
cheat-surrounded-by-nutters = Geef voetgangers wapens
cheat-blue-suede-shoes = Alle voetgangers zijn Elvis Presley
cheat-attack-of-the-village-people = Voetgangers vallen de speler aan met geweeren en raketten
cheat-only-homies-allowed = Gang leden overal
cheat-better-stay-indoors = Gangs nemen de straten over
cheat-state-of-emergency = Voetgangers komen in opstand
cheat-ghost-town = Minder verkeer en geen voetgangers

## Themes
cheat-ninja-town = Overal zijn er Triad mensen
cheat-love-conquers-all = Overal zijn er Pimp mensen
cheat-lifes-a-beach = Overal zijn er Beach party mensen
cheat-hicksville = Overal zijn er Rural mensen
cheat-crazy-town = Overal zijn er Carnival mensen

## General vehicle cheats
cheat-all-cars-go-boom = Laat alle voertuigen exploderen
cheat-wheels-only-please = Maak voertuigen onzichtbaar
cheat-sideways-wheels = De wielen van autos staan gedraaid
cheat-speed-freak = Alle autos hebben nitro
cheat-cool-taxis = Taxis hebben hydraulica en nitro

## Vehicle cheats for the player
cheat-chitty-chitty-bang-bang = Vliegende autos
cheat-cj-phone-home = Hoge bunny hops op de fiets
cheat-touch-my-car-you-die = Sloop andere voertuigen als je ze aanraakt
cheat-bubble-cars = Autos zweven weg als ze geraakt worden
cheat-stick-like-glue = Betere ophanging en bestuurbaarheid
cheat-dont-try-and-stop-me = Verkeerslichten staan altijd op groen
cheat-flying-fish = Vliegende boten

## Weapon usage
cheat-full-clip = Iedereen heeft oneindige kogels
cheat-i-wanna-driveby = Volledig wapen werking in voortuigen

## Player effects
cheat-goodbye-cruel-world = Zelfmoord
cheat-take-a-chill-pill = Adrenaline effecten
cheat-prostitutes-pay = Prostituees betalen jou

## Miscellaneous
cheat-xbox-helper = Zet je statistieken zodat je bijna de Xbox prestaties hebt

## Pointless cheats

# Tells the user that a cheat will ALWAYS crash their game.
cheat-crash-warning = LAAT HET SPEL VASTLOPEN!

cheat-slot-melee = { cheat-crash-warning } Melee slot
cheat-slot-handgun = { cheat-crash-warning } pistool slot
cheat-slot-smg = { cheat-crash-warning } SMG slot
cheat-slot-shotgun = { cheat-crash-warning } Shotgun slot
cheat-slot-assault-rifle = { cheat-crash-warning } Assault geweer slot
cheat-slot-long-rifle = { cheat-crash-warning } Lang geweer slot
cheat-slot-thrown = { cheat-crash-warning } Gooi wapen slot
cheat-slot-heavy = { cheat-crash-warning } Zware artillerie slot
cheat-slot-equipment = { cheat-crash-warning } Apperatuur slot
cheat-slot-other = { cheat-crash-warning } Rest slot

cheat-predator = Doet niks
