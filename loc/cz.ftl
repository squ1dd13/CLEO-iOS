# Used in the settings menu to show the name of the language.
language-name = čeština

### ==== Splash screen ====

# First line at the bottom of the screen.
splash-legal = Autorské právo © 2020-2023 squ1dd13, AYZM, ODIN, RAiZOK, tharryz, wewewer1. Licencováno pod licencí MIT.

# Second line.
splash-fun = Ahoj, čeští kamarádi! CLEO se přeložilo do češtiny tharryzem. Bavte se s hrou! Příjemnou zábavu! Vyrobeno s láskou ve Velké Británii.

### ==== Updates ====

# PLEASE NOTE: The update menu is currently made from a game menu, and the game fonts do not
# support non-ASCII characters. Please write with ASCII only here. If you can't do that, leave it
# in English for now. I plan to add Unicode support here in the future.

# Displayed at the top of the update screen.
update-prompt-title = Aktualizace je k dispozici

# Message shown on the update screen. { $new_version } will be replaced with the update's version number.
update-prompt-message = CLEO verze { $new_version } je právě k dispozici. Chceš si to stáhnout z GitHubu?
# todo: Add "Yes" and "No" for update menu to localisation files.
# The yes/no options are part of the game, so they're not directly in CLEO's control (yet).

## Release channel settings

update-release-channel-opt-title = Uvolňovací kanál
update-release-channel-opt-desc = Na které aktualizace dostáváte oznámení. Alpha poskytuje novější funkce dříve, ale může mít více chyb. Zakázat aktualizace se nedoporučuje.

update-release-channel-opt-disabled = Zakázáno
update-release-channel-opt-stable = Stabilní
update-release-channel-opt-alpha = Alpha

### ==== Menu ====

# Title for the button at the bottom of the screen that closes the CLEO menu.
menu-close = Zavřít

# Title for the options tab.
menu-options-tab-title = Možnosti

### ==== Script menu ====

## Script warning overview

# Shown at the top of the script menus when at least one error has been found in any script. This
# is not shown at all if there are zero scripts with errors in them.
menu-script-warning-overview =
    { $num_scripts_with_errors ->
        [one] Chyby se zjistily v jednom skriptu. Tento skript je zvýrazněn oranžově.
        *[other] Chyby se zjistily v { $num_scripts_with_errors } skriptech. Tyto skripty jsou zvýrazněny oranžově.
    }

# The second line of the warning.
menu-script-see-below = Další podrobnosti naleznete níže.

menu-script-csa-tab-title = CSA
menu-script-csi-tab-title = CSI

## Specific script warnings

# The script does things that CLEO doesn't support yet.
script-unimplemented-in-cleo = Používající funkce se aktuálně nepodporují CLEO IOS
# The script does things that are not possible on iOS (for system reasons).
script-impossible-on-ios = Nějaký používající kód se nebude fungovat ve systému IOS
# The script is identical to another script. { $original_script } will be replaced with the name of
# the script that this one is a duplicate of.
script-duplicate = Duplikát { $original_script }.

# There was an error when checking the script code for problems.
script-check-failed = Nelze skenovat skript. Nahlaste to prosím jako chybu na GitHubu, nebo na Discordu.

# No problems were found when scanning the script. This is a safe script!
script-no-problems = Nebyly zjištěny žádné chyby.

# Formats for script names in the menu.
script-csa-row-title = { $script_name }
script-csi-row-title = { $script_name }

## Script status messages

# The script is running normally.
script-running = Běží

# The script is not running.
script-not-running = Neběží

# The script has been forced to run by the user, even though there are problems with it. This only
# applies to CSA scripts.
script-csa-forced-running = Přinutit běžit

## Script settings

script-mode-opt-title = Režim zpracování skriptu
script-mode-opt-desc = Měnit režim, jakým CLEO zpracovává kód skriptu. Zkuste to změnit, pokud skript nefunguje normálně.

# In "don't break" mode, CLEO won't try to reduce script lag. This might be more stable sometimes.
script-mode-opt-dont-break = Pomalu

# In "break" mode, CLEO will try to reduce script lag caused by long loops in script code.
script-mode-opt-break = Rychle

### ==== FPS ====

## FPS lock option

fps-lock-opt-title = Omezení FPS
fps-lock-opt-desc = Maximální snímková frekvence, při které hra poběží. 30 FPS vypadá hůř, ale šetří baterii.

fps-lock-opt-30 = 30 FPS
fps-lock-opt-60 = 60 FPS

## FPS counter option

fps-counter-opt-title = Indikátor FPS
fps-counter-opt-desc = Povolit nebo zakázat indikátor FPS na obrazovce.

fps-counter-opt-hidden = Zakázáno
fps-counter-opt-enabled = Povoleno

### ==== Cheat system ====

## Menu

cheat-tab-title = Cheaty

# Two lines of text shown at the top of the cheats menu.
cheat-menu-warning = Používání cheatů může vést k pádům a případně ke ztrátě postupu ve hře.
  Pokud nechcete riskovat prolomení vašeho uložení, zálohujte si svůj postup nejprve do jiného slotu.

## Status messages for cheats

cheat-on = Zapnout
cheat-off = Vypnout

# Cheat will be turned on when the menu is closed.
cheat-queued-on = Ve frontě

# Cheat will be turned off when the menu is closed.
cheat-queued-off = Odstranit z fronty

# Formats for cheat codes in the menu.
cheat-code-row-title = { $cheat_code }
cheat-no-code-title = ???

## Cheat saving option

cheat-transience-opt-title = Uložený režim cheatů
cheat-transience-opt-desc = Ovládat, jak jsou spravovány cheaty při opětovném načítání/restartování hry.
cheat-transience-opt-transient = Resetovat vše
cheat-transience-opt-persistent = Uložit nastavení

### ==== Cheat descriptions ====

## Weapon sets
cheat-thugs-armoury = Sada zbraní 1
cheat-professionals-kit = Sada zbraní 2
cheat-nutters-toys = Sada zbraní 3
cheat-weapons-4 = Dostat robertek, minigun a termální brýle/brýle pro noční vidění

## Debug cheats
cheat-debug-mappings = Ladění (zobrazit mapování)
cheat-debug-tap-to-target = Ladění (zobrazit klepnutím na cíl)
cheat-debug-targeting = Ladění (zobrazit cílení)

## Properly cheating
cheat-i-need-some-help = Být zdravý, dostat brnění a $250,000
cheat-skip-mission = Přeskočit na dokončení některých misí

## Superpowers
cheat-full-invincibility = Plná neporazitelnost
cheat-sting-like-a-bee = Super údery
cheat-i-am-never-hungry = Hráč nikdy nemá hlad.
cheat-kangaroo = 10x výška skoku.
cheat-noone-can-hurt-me = Neomezené zdraví
cheat-man-from-atlantis = Nekonečná kapacita plic

## Social player attributes
cheat-worship-me = Maximální respekt
cheat-hello-ladies = Maximální sexuální atrakce

## Physical player attributes
cheat-who-ate-all-the-pies = Maximální tuk
cheat-buff-me-up = Maximální sval
cheat-max-gambling = Maximální hráčské dovednosti
cheat-lean-and-mean = Maximální tuk a sval
cheat-i-can-go-all-night = Maximální výdrž

## Player skills
cheat-professional-killer = Úroveň vraha pro všechny zbraně
cheat-natural-talent = Maximální dovednosti vozidla

## Wanted level
cheat-turn-up-the-heat = Zvýšit úroveň hledanosti o dvě hvězdičky
cheat-turn-down-the-heat = Vyčištit úroveň hledanosti
cheat-i-do-as-i-please = Uzamknout úroveň hledanosti na aktuální hodnotu.
cheat-bring-it-on = Šesthvězdičková úroveň hledanosti

## Weather
cheat-pleasantly-warm = Slunečno
cheat-too-damn-hot = Horký den
cheat-dull-dull-day = Zataženo
cheat-stay-in-and-watch-tv = Déšť
cheat-cant-see-where-im-going = Mlha
cheat-scottish-summer = Bouřka
cheat-sand-in-my-ears = Písečná bouře

## Time
cheat-clock-forward = Čas o 4 hodiny zrychluje
cheat-time-just-flies-by = Rychlejší ubíhání času
cheat-speed-it-up = Rychlejší hra
cheat-slow-it-down = Pomalejší hra
cheat-night-prowler = Vždy půlnoc
cheat-dont-bring-on-the-night = Vždy je 9 hodin večer

## Spawning wearables
cheat-lets-go-base-jumping = Klást padák
cheat-rocketman = Klást raketový batoh

## Spawning vehicles

# The vehicle names (in capitals) should not be changed, as they are part of the game. The
# descriptions (in brackets) do need to be translated.
cheat-time-to-kick-ass = Klást Rhino (tank armády)
cheat-old-speed-demon = Klást Bloodring Banger (vůz demoličních derby)
cheat-tinted-rancher = Klást Rancher s dvěma tónovanými okny (dvoudveřové SUV)
cheat-not-for-public-roads = Klást Hotring Racer A (závodní auto)
cheat-just-try-and-stop-me = Klást Hotring Racer B (závodní auto)
cheat-wheres-the-funeral = Klást Romero (pohřební vůz)
cheat-celebrity-status = Klást Stretch Limousine (limuzína)
cheat-true-grime = Klást Trashmaster (popelářské auto/nákladní sklápěč)
cheat-18-holes = Klást Caddy (golfový vozík)
cheat-jump-jet = Klást Hydra (útočný letoun VTOL)
cheat-i-want-to-hover = Klást Vortex (vznášedlo)
cheat-oh-dude = Klást Hunter (vojenský útočný vrtulník)
cheat-four-wheel-fun = Klást Quad (čtyřkolka)
cheat-hit-the-road-jack = Klást Tanker s cisternovým přívěsem (cisternový vůz)
cheat-its-all-bull = Klást Dozer (shrnovač)
cheat-flying-to-stunt = Klást Stunt Plane (akrobatické letadlo)
cheat-monster-mash = Klást Monster Truck (monster truck)

## Gang recruitment
cheat-wanna-be-in-my-gang = Naverbujte kohokoli do svého gangu a dejte mu pistoli tím, že na něj namíříte pistoli
cheat-noone-can-stop-us = Naverbujte kohokoli do svého gangu a dejte mu AK-47 tím, že na něj zaměříte AK-47
cheat-rocket-mayhem = Naverbujte kohokoli do svého gangu a dejte mu RPG tím, že na něj zaměříte RPG

## Traffic
cheat-all-drivers-are-criminals = Všichni NPC řidiči jezdí agresivně a mají úroveň hledanosti
cheat-pink-is-the-new-cool = Růžový provoz
cheat-so-long-as-its-black = Černý provoz
cheat-everyone-is-poor = Venkovský provoz
cheat-everyone-is-rich = Provoz sportovních aut

## Pedestrians
cheat-rough-neighbourhood = Dát hráči golfovou hůl a přimět chodce k výtržnostem
cheat-stop-picking-on-me = Chodci útočí na hráče
cheat-surrounded-by-nutters = Dát chodcům zbraně
cheat-blue-suede-shoes = Všichni chodci jsou Elvis Presley
cheat-attack-of-the-village-people = Chodci útočí na hráče zbraněmi a raketami
cheat-only-homies-allowed = Členové gangu všude
cheat-better-stay-indoors = Gangy ovládají ulice
cheat-state-of-emergency = Chodci se bouří
cheat-ghost-town = Omezený provoz a žádní chodci

## Themes
cheat-ninja-town = Téma triády
cheat-love-conquers-all = Kuplířská téma
cheat-lifes-a-beach = Téma plážové párty
cheat-hicksville = Venkovská téma
cheat-crazy-town = Karnevalová téma

## General vehicle cheats
cheat-all-cars-go-boom = Všechna auta vybuchnou
cheat-wheels-only-please = Neviditelná auta
cheat-sideways-wheels = Auta mají boční kola
cheat-speed-freak = Všechna auta mají nitro
cheat-cool-taxis = Taxíky mají hydrauliku a nitro

## Vehicle cheats for the player
cheat-chitty-chitty-bang-bang = Létající auta
cheat-cj-phone-home = Velmi vysoký Bunny Hop
cheat-touch-my-car-you-die = Při srážce zničit ostatní vozidla
cheat-bubble-cars = Auta odlétají po zásahu
cheat-stick-like-glue = Vylepšené odpružení a ovládání
cheat-dont-try-and-stop-me = Na semaforech je vždy zelená
cheat-flying-fish = Létající čluny

## Weapon usage
cheat-full-clip = Každý má neomezenou munici
cheat-i-wanna-driveby = Všechny zbraně jsou použitelné ve vozidlech

## Player effects
cheat-goodbye-cruel-world = Sebevražda
cheat-take-a-chill-pill = Účinky adrenalinu
cheat-prostitutes-pay = Prostitutky vám platí

## Miscellaneous
cheat-xbox-helper = Upravte statistiky tak, abyste byli blízko k získání úspěchů Xboxu

## Pointless cheats

# Tells the user that a cheat will ALWAYS crash their game.
-cheat-crash-warning = ZHROUCENÍ!

cheat-slot-melee = { -cheat-crash-warning } Slot zbraní na blízko
cheat-slot-handgun = { -cheat-crash-warning } Slot pistolí
cheat-slot-smg = { -cheat-crash-warning } Slot samopalů
cheat-slot-shotgun = { -cheat-crash-warning } Slot brokovnicí
cheat-slot-assault-rifle = { -cheat-crash-warning } Slot útočných pušek
cheat-slot-long-rifle = { -cheat-crash-warning } Slot dlouhých pušek
cheat-slot-thrown = { -cheat-crash-warning } Slot vrhacích zbraní
cheat-slot-heavy = { -cheat-crash-warning } Slot těžká dělostřelectva
cheat-slot-equipment = { -cheat-crash-warning } Slot vybavení
cheat-slot-other = { -cheat-crash-warning } Jiný slot

cheat-predator = Nic nedělá
