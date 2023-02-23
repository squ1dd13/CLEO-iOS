# Used in the settings menu to show the name of the language.
language-name = slovenčina

# Shown when this language has been selected automatically.
language-auto-name = Automatický ({ language-name })

# The name of the language setting.
language-opt-title = Jazyk

# The language setting description.
language-opt-desc = Jazyk, ktorý sa používa CLEO. Automatický režim použije nastavenie vášho systému. Pridajte prosím svoj jazyk na Discord!

### ==== Splash screen ====

# First line at the bottom of the screen.
splash-legal = Autorské právo © 2020-2023 { $copyright_names }. Licencované podľa licencie MIT.

# Second line.
splash-fun = Ahojte! Slovenskí kamaráti! CLEO sa preložilo do slovenčiny tharryzom. Vyrobené s láskou vo Veľkej Británii. Bavte sa s hrou! Príjemnú zábavu!

### ==== Updates ====

# PLEASE NOTE: The update menu is currently made from a game menu, and the game fonts do not
# support non-ASCII characters. Please write with ASCII only here. If you can't do that, leave it
# in English for now. I plan to add Unicode support here in the future.

# Displayed at the top of the update screen.
update-prompt-title = Aktualizácia je k dispozícii

# Message shown on the update screen. { $new_version } will be replaced with the update's version number.
update-prompt-message = CLEO verzia { $new_version } je práve k dispozícii. Chceš si to stiahnuť z GitHubu?

# todo: Add "Yes" and "No" for update menu to localisation files.
# The yes/no options are part of the game, so they're not directly in CLEO's control (yet).

## Release channel settings

update-release-channel-opt-title = Uvolňovací kanál
update-release-channel-opt-desc = Na ktoré aktualizácie CLEO dostávate oznámenie. Alpha poskytuje novšie funkcie skôr, ale môže mať viac chýb. Zakázanie aktualizácií sa neodporúča.

update-release-channel-opt-disabled = Zakázané
update-release-channel-opt-stable = Stabilné
update-release-channel-opt-alpha = Alpha

### ==== Menu ====

# Title for the button at the bottom of the screen that closes the CLEO menu.
menu-close = Zavrieť

# Title for the options tab.
menu-options-tab-title = Možnosti

### ==== Script menu ====

## Script warning overview

# Shown at the top of the script menus when at least one error has been found in any script. This
# is not shown at all if there are zero scripts with errors in them.
menu-script-warning-overview =
    { $num_scripts_with_errors ->
        [one] Chyby sa zistili v jednom skripte. Tento skript je zvýraznený oranžovou farbou.
        *[other] Chyby sa zistili v { $num_scripts_with_errors } skriptoch. Tieto skripty sú zvýraznené oranžovou farbou.
    }

# The second line of the warning.
menu-script-see-below = Ďalšie podrobnosti nájdete nižšie.

menu-script-csa-tab-title = CSA
menu-script-csi-tab-title = CSI

## Specific script warnings

# The script does things that CLEO doesn't support yet.
script-unimplemented-in-cleo = Používajúce funkcie sa aktuálne nepodporujú CLEO IOS

# The script does things that are not possible on iOS (for system reasons).
script-impossible-on-ios = Nejaký používajúci kód sa nebude fungovať v systéme IOS

# The script is identical to another script. { $original_script } will be replaced with the name of
# the script that this one is a duplicate of.
script-duplicate = Duplikát { $original_script }.

# There was an error when checking the script code for problems.
script-check-failed = Nie je možné skenovať skript. Nahláste to prosím ako chybu na GitHube, alebo na Discorde.

# No problems were found when scanning the script. This is a safe script!
script-no-problems = Neboli zistené žiadne chyby.

# Formats for script names in the menu.
script-csa-row-title = { $script_name }
script-csi-row-title = { $script_name }

## Script status messages

# The script is running normally.
script-running = Beží

# The script is not running.
script-not-running = Nebeží

# The script has been forced to run by the user, even though there are problems with it. This only
# applies to CSA scripts.
script-csa-forced-running = Prinútiť bežiť

## Script settings

script-mode-opt-title = Režim spracovania skriptu
script-mode-opt-desc = Meniť režim, akým CLEO spracováva kód skriptu. Skúste to zmeniť, pokiaľ skript nefunguje normálne.

# In "don't break" mode, CLEO won't try to reduce script lag. This might be more stable sometimes.
script-mode-opt-dont-break = Pomaly

# In "break" mode, CLEO will try to reduce script lag caused by long loops in script code.
script-mode-opt-break = Rýchlo

### ==== FPS ====

## FPS lock option

fps-lock-opt-title = Obmedzenie FPS
fps-lock-opt-desc = Maximálna snímková frekvencia, pri ktorej hra pobeží. 30 FPS vyzerá horšie, ale šetrí batériu.

fps-lock-opt-30 = 30 FPS
fps-lock-opt-60 = 60 FPS

## FPS counter option

fps-counter-opt-title = Indikátor FPS
fps-counter-opt-desc = Povoliť alebo zakázať indikátor FPS na obrazovke.

fps-counter-opt-hidden = Zakázané
fps-counter-opt-enabled = Povolené

### ==== Cheat system ====

## Menu

cheat-tab-title = Cheaty

# Two lines of text shown at the top of the cheats menu.
cheat-menu-warning = Používanie cheatov môže viesť k pádom a prípadne k strate postupu v hre.
  Pokiaľ nechcete riskovať prelomenie vášho uloženia, zálohujte si svoj postup najskôr do iného slotu.

## Status messages for cheats

cheat-on = Zapnúť
cheat-off = Vypnúť

# Cheat will be turned on when the menu is closed.
cheat-queued-on = Vo fronte

# Cheat will be turned off when the menu is closed.
cheat-queued-off = Odstrániť z frontu

# Formats for cheat codes in the menu.
cheat-code-row-title = { $cheat_code }
cheat-no-code-title = ???

## Cheat saving option

cheat-transience-opt-title = Uložený režim cheatov
cheat-transience-opt-desc = Ovládať, ako sú spravované cheaty pri opätovnom načítaní/reštartovaní hry.

cheat-transience-opt-transient = Resetovať všetko
cheat-transience-opt-persistent = Uložiť nastavenia

### ==== Cheat descriptions ====

## Weapon sets
cheat-thugs-armoury = Súprava zbraní 1
cheat-professionals-kit = Súprava zbraní 2
cheat-nutters-toys = Súprava zbraní 3
cheat-weapons-4 = Dostať robertko, minigun a termálne okuliare/okuliare na nočné videnie

## Debug cheats
cheat-debug-mappings = ladenie (zobraziť mapovania)
cheat-debug-tap-to-target = ladenie (zobraziť kliknutím na cieľ)
cheat-debug-targeting = ladenie (zobraziť zacielenie)

## Properly cheating
cheat-i-need-some-help = Byť zdravý, dostať brnenie a $250,000
cheat-skip-mission = Preskočiť na dokončenie niektorých misií

## Superpowers
cheat-full-invincibility = Plná neporaziteľnosť
cheat-sting-like-a-bee = Super údery
cheat-i-am-never-hungry = Hráč nikdy nemá hlad.
cheat-kangaroo = 10x výška skoku.
cheat-noone-can-hurt-me = Neobmedzené zdravie
cheat-man-from-atlantis = Nekonečná kapacita pľúc

## Social player attributes
cheat-worship-me = Maximálny rešpekt
cheat-hello-ladies = Maximálna sexuálna atrakcia

## Physical player attributes
cheat-who-ate-all-the-pies = Maximálny tuk
cheat-buff-me-up = Maximálny sval
cheat-max-gambling = Maximálne hráčske zručnosti
cheat-lean-and-mean = Maximálny tuk a sval
cheat-i-can-go-all-night = Maximálna výdrž

## Player skills
cheat-professional-killer = Úroveň vraha pre všetky zbrane
cheat-natural-talent = Maximálne zručnosti vozidla

## Wanted level
cheat-turn-up-the-heat = Zvýšiť úroveň hľadanosti o dve hviezdičky
cheat-turn-down-the-heat = Vyčistiť úroveň hľadanosti
cheat-i-do-as-i-please = Uzamknúť úroveň hľadanosti na aktuálnu hodnotu
cheat-bring-it-on = Šesťhviezdičková úroveň hľadanosti

## Weather
cheat-pleasantly-warm = Slnečno
cheat-too-damn-hot = Horúci deň
cheat-dull-dull-day = Zamračené
cheat-stay-in-and-watch-tv = Dážď
cheat-cant-see-where-im-going = Hmla
cheat-scottish-summer = Búrka
cheat-sand-in-my-ears = Piesočná búrka
## Time
cheat-clock-forward = Čas o 4 hodiny zrýchľuje
cheat-time-just-flies-by = Rýchlejšie ubiehanie času
cheat-speed-it-up = Rýchlejšia hra
cheat-slow-it-down = Pomalšia hra
cheat-night-prowler = Vždy polnoc
cheat-dont-bring-on-the-night = Vždy je 9 hodín večer

## Spawning wearables
cheat-lets-go-base-jumping = Klásť padák
cheat-rocketman = Klásť raketový batoh

## Spawning vehicles

# The vehicle names (in capitals) should not be changed, as they are part of the game. The
# descriptions (in brackets) do need to be translated.
cheat-time-to-kick-ass = Klásť Rhino (tank armády)
cheat-old-speed-demon = Klásť Bloodring Banger (voz demolačných derby)
cheat-tinted-rancher = Klásť Rancher s dvoma tónovanými oknami (dvojdverové SUV)
cheat-not-for-public-roads = Klásť Hotring Racer A (závodné auto)
cheat-just-try-and-stop-me = Klásť Hotring Racer B (závodné auto)
cheat-wheres-the-funeral = Klásť Romero (pohrebné vozidlo)
cheat-celebrity-status = Klásť Stretch Limousine (limuzína)
cheat-true-grime = Klásť Trashmaster (smetiarske auto/nákladný sklápač)
cheat-18-holes = Klásť Caddy (golfový vozík)
cheat-jump-jet = Klásť Hydra (útočné lietadlo VTOL)
cheat-i-want-to-hover = Klásť Vortex (vznášadlo)
cheat-oh-dude = Klásť Hunter (vojenský útočný vrtuľník)
cheat-four-wheel-fun = Klásť Quad (štvorkolka)
cheat-hit-the-road-jack = Klásť Tanker s cisternovým prívesom (cisternový voz)
cheat-its-all-bull = Klásť Dozér (zhrňovač)
cheat-flying-to-stunt = Klásť Stunt Plane (akrobatické lietadlo)
cheat-monster-mash = Klásť Monster Truck (monster truck)

## Gang recruitment
cheat-wanna-be-in-my-gang = Naverbujte kohokoľvek do svojho gangu a dajte mu pištoľ tým, že naň namierite pištoľ
cheat-noone-can-stop-us = Naverbujte kohokoľvek do svojho gangu a dajte mu AK-47 tým, že naň zameriate AK-47
cheat-rocket-mayhem = Naverbujte kohokoľvek do svojho gangu a dajte mu RPG tým, že naň zameriate RPG

## Traffic
cheat-all-drivers-are-criminals = Všetci NPC vodiči jazdia agresívne a majú úroveň hľadanosti
cheat-pink-is-the-new-cool = Ružová prevádzka
cheat-so-long-as-its-black = Čierna prevádzka
cheat-everyone-is-poor = Vidiecka prevádzka
cheat-everyone-is-rich = Prevádzka športových áut

## Pedestrians
cheat-rough-neighbourhood = Dať hráči golfovú palicu a prinútiť chodcov k výtržnostiam
cheat-stop-picking-on-me = Chodci útočia na hráča
cheat-surrounded-by-nutters = Dať chodcom zbrane
cheat-blue-suede-shoes = Všetci chodci sú Elvis Presley
cheat-attack-of-the-village-people = Chodci útočia na hráča zbraňami a raketami
cheat-only-homies-allowed = Členovia gangu všade
cheat-better-stay-indoors = Gangy ovládajú ulice
cheat-state-of-emergency = Chodci sa búria
cheat-ghost-town = Obmedzená prevádzka a žiadni chodci

## Themes
cheat-ninja-town = Téma triády
cheat-love-conquers-all = Kupliarska téma
cheat-lifes-a-beach = Téma plážovej párty
cheat-hicksville = Vidiecka téma
cheat-crazy-town = Karnevalová téma

## General vehicle cheats
cheat-all-cars-go-boom = Všetky autá vybuchnú
cheat-wheels-only-please = Neviditeľné autá
cheat-sideways-wheels = Autá majú bočné kolesá
cheat-speed-freak = Všetky autá majú vnútro
cheat-cool-taxis = Taxíky majú hydrauliku a vnútro

## Vehicle cheats for the player
cheat-chitty-chitty-bang-bang = Lietajúce autá
cheat-cj-phone-home = Veľmi vysoký Bunny Hop
cheat-touch-my-car-you-die = Pri zrážke zničiť ostatné vozidlá
cheat-bubble-cars = Autá odlietajú po zásahu
cheat-stick-like-glue = Vylepšené odpruženie a ovládanie
cheat-dont-try-and-stop-me = Na semaforoch je vždy zelená
cheat-flying-fish = Lietajúce člny

## Weapon usage
cheat-full-clip = Každý má neobmedzenú muníciu
cheat-i-wanna-driveby = Všetky zbrane sú použiteľné vo vozidlách

## Player effects
cheat-goodbye-cruel-world = Samovražda
cheat-take-a-chill-pill = Účinky adrenalínu
cheat-prostitutes-pay = Prostitútky vám platia

## Miscellaneous
cheat-xbox-helper = Upravte štatistiky tak, aby ste boli blízko k získaniu úspechov Xboxu

## Pointless cheats

# Tells the user that a cheat will ALWAYS crash their game.
cheat-crash-warning = ZHRÚTENIA!

cheat-slot-melee = { cheat-crash-warning } Slot zbraní na blízko
cheat-slot-handgun = { cheat-crash-warning } Slot pištoľou
cheat-slot-smg = { cheat-crash-warning } Slot samopalov
cheat-slot-shotgun = { cheat-crash-warning } Slot brokovnicou
cheat-slot-assault-rifle = { cheat-crash-warning }Slot útočných pušiek
cheat-slot-long-rifle = { cheat-crash-warning } Slot dlhých pušiek
cheat-slot-thrown = { cheat-crash-warning } Slot vrhacích zbraní
cheat-slot-heavy = { cheat-crash-warning } Slot ťažká delostrelectva
cheat-slot-equipment = { cheat-crash-warning } Slot vybavenie
cheat-slot-other = { cheat-crash-warning } Iný slot

cheat-predator = Nič nerobí
