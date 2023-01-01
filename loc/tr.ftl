# Used in the settings menu to show the name of the language.
language-name = Turkce
### ==== Splash screen ====

# First line at the bottom of the screen.
splash-legal = Telif Hakki © 2020-2023 squ1dd13, AYZM, ODIN, RAiZOK, tharryz, wewewer1. MIT Lisansi Altinda Lisanslanmistir.

# Second line.
splash-fun = Turklere Merhaba! Ceviri Berkakahs. Ingiltere'de sevgilerle yapildi. Iyi eglenceler!

### ==== Updates ====

# PLEASE NOTE: The update menu is currently made from a game menu, and the game fonts do not
# support non-ASCII characters. Please write with ASCII only here. If you can't do that, leave it
# in English for now. I plan to add Unicode support here in the future.

# Displayed at the top of the update screen.
update-prompt-title = Guncelleme Mevcut

# Message shown on the update screen. { $new_version } will be replaced with the update's version number.
update-prompt-message = CLEO'nun { $new_version } versiyonu hazir. GitHub'tan indirmek ister misin?

# todo: Add "Yes" and "No" for update menu to localisation files.
# The yes/no options are part of the game, so they're not directly in CLEO's control (yet).

## Release channel settings

update-release-channel-opt-title = Yayinlayis Kanali
update-release-channel-opt-desc = Hangi CLEO guncellemeleri bildirimini alicagini sec. Alpha yeni ozellikleri daha onceden kullanabilmeni saglar ama hatalar olabilir. Guncellemeleri devre disi birakmak onerilmez.
update-release-channel-opt-disabled = Devre disi
update-release-channel-opt-stable = Stabil
update-release-channel-opt-alpha = Alpha

### ==== Menu ====

# Title for the button at the bottom of the screen that closes the CLEO menu.
menu-close = Kapat

# Title for the options tab.
options-tab-title = Ayarlar

### ==== Script menu ====

## Script warning overview

# Shown at the top of the script menus when at least one error has been found in any script. This
# is not shown at all if there are zero scripts with errors in them.
menu-script-warning-overview =
    { $numberOfScriptsWithErrors ->
        [one] Scriptte hata bulundu, hatali script turuncu renk ile belirtildi.
        *[other] Su scriptlerde hata bulundu. { $numberOfScriptsWithErrors } hatali scriptler turuncu renk ile belirtildi.
    }

# The second line of the warning.
menu-script-see-below = Daha fazla detay icin alt kismi kontrol edin.
## Specific script warnings

# The script does things that CLEO doesn't support yet.
script-unimplemented-in-cleo = CLEO iOS destegi olmayan script.

# The script does things that are not possible on iOS (for system reasons).
script-impossible-on-ios = iOS kod uyusmazligindan dolayi bu sistemde calisamayacak script.

# The script is identical to another script. { $originalScript } will be replaced with the name of
# the script that this one is a duplicate of.
script-duplicate = Ayni scriptten zaten var { $originalScript }.

# There was an error when checking the script code for problems.
script-check-failed = Script taranmasinda hata, lutfen GitHub veya Discorddan bildirin.

# No problems were found when scanning the script. This is a safe script!
script-no-problems = Hata Bulunamadi.

## Script status messages

# The script is running normally.
script-running = Calisiyor

# The script is not running.
script-not-running = Calismiyor

# The script has been forced to run by the user, even though there are problems with it. This only
# applies to CSA scripts.
script-csa-forced-running = Zorla Calisiyor

## Script settings

script-mode-opt-title = Script Hazirlana Modu
script-mode-opt-desc = CLEO'nun scriptleri nasi tarayacagini degistirir. Eger scriptiniz calismiyorsa bu secenegi deneyin.

# In "don't break" mode, CLEO won't try to reduce script lag. This might be more stable sometimes.
script-mode-opt-dont-break = Yavas

# In "break" mode, CLEO will try to reduce script lag caused by long loops in script code.
script-mode-opt-break = Hizli

### ==== FPS ====

## FPS lock option

fps-lock-opt-title = FPS Limiti
fps-lock-opt-desc =  Maksimum 30 FPS. Oyun kotu gorunecek fakat sarjiniz daha uzun sure dayanacak.
fps-lock-opt-30 = 30 FPS
fps-lock-opt-60 = 60 FPS

## FPS counter option

fps-counter-opt-title = FPS Sayaci
fps-counter-opt-desc = Ekrandaki FPS sayacini acar veya kapatir.
fps-counter-opt-hidden = Devre disi
fps-counter-opt-enabled = Aktif

### ==== Cheat system ====

## Menu

cheat-tab-title = Hileler

# Two lines of text shown at the top of the cheats menu.
cheat-menu-warning = Hile kullanmak oyununuzu cokertip ilerleme kaybetmenize sebep olabilir.
cheat-menu-advice = Eger ilerleme kaybetmek istemiyorsaniz save dosyanizi yedekleyin.

## Status messages for cheats

cheat-on = Aktif
cheat-off = Devre disi
# Cheat will be turned on when the menu is closed.
cheat-queued-on = Siraya konuldu
# Cheat will be turned off when the menu is closed.
cheat-queued-off = Siradan cikarildi
## Cheat saving option

cheat-transience-opt-title = Hileleri kaydet
cheat-transience-opt-desc = Oyunu yeniden baslattiginizda ayni hileler kayitli kalir.
cheat-transience-opt-transient = Hepsini sifirla
cheat-transience-opt-persistent = Hileleri kaydet

### ==== Cheat descriptions ====

## Weapon sets
cheat-thugs-armoury = Silah seti 1
cheat-professionals-kit = Silah seti 2
cheat-nutters-toys = Silah seti 3
cheat-weapons-4 = Dildo, minigun ve thermal/gece-gorus gozlukleri verir.

## Debug cheats
cheat-debug-mappings = Debug (Karakter haritalanmasini goster)
cheat-debug-tap-to-target = Debug (Tiklama hedefini goster)
cheat-debug-targeting = Debug (Hedeflenmeyi goster)

## Properly cheating
cheat-i-need-some-help = Can, zirh ve $250,000 ver
cheat-skip-mission = Bazi gorevlerin sonuna atla

## Superpowers
cheat-full-invincibility = Tamamen olumsuzluk
cheat-sting-like-a-bee = Super yumruk
cheat-i-am-never-hungry = CJ asla acikmaz
cheat-kangaroo = 10x ziplama mesafesi
cheat-noone-can-hurt-me = Sinirsiz can
cheat-man-from-atlantis = Sinirsiz akciger kapasitesi

## Social player attributes
cheat-worship-me = Maksimum saygi
cheat-hello-ladies = Maksimum cekicilik

## Physical player attributes
cheat-who-ate-all-the-pies = Maksimum yag
cheat-buff-me-up = Maksimum kas
cheat-max-gambling = Maksimum kumar becerisi
cheat-lean-and-mean = Minimum yag ve kas
cheat-i-can-go-all-night = Maksimum stamina

## Player skills
cheat-professional-killer = Butun silahlarda Hitman seviyesi
cheat-natural-talent = Maksimum surus yetenegi

## Wanted level
cheat-turn-up-the-heat = Aranma seviyesini 2 yildiz arttirir
cheat-turn-down-the-heat = Aranma seviyesini siler
cheat-i-do-as-i-please = Aranma seviyesini suanki seviyeye sabitler
cheat-bring-it-on = 6 Yildiz aranma

## Weather
cheat-pleasantly-warm = Gunesli hava
cheat-too-damn-hot = Cok gunesli hava
cheat-dull-dull-day = Kasvetli hava
cheat-stay-in-and-watch-tv = Yagmurlu hava
cheat-cant-see-where-im-going = Sisli hava
cheat-scottish-summer = Firtinali hava
cheat-sand-in-my-ears = Kum firtinasi

## Time
cheat-clock-forward = Saati 4 saat ileri al
cheat-time-just-flies-by = Daha hizli gecen zaman
cheat-speed-it-up = Hizlandirilmis oynanis
cheat-slow-it-down = Yavaslatilmis oynanis
cheat-night-prowler = Her zaman gece yarisi
cheat-dont-bring-on-the-night = Her zaman saat 9

## Spawning wearables
cheat-lets-go-base-jumping = Parasut iste
cheat-rocketman = Jetpack isfe

## Spawning vehicles

# The vehicle names (in capitals) should not be changed, as they are part of the game. The
# descriptions (in brackets) do need to be translated.
cheat-time-to-kick-ass = Rhino cagir (askeri tank)
cheat-old-speed-demon = Bloodring Banger cagir (derbi arabasi)
cheat-tinted-rancher = Cam filmli Rancher cagir (iki kapili SUV)
cheat-not-for-public-roads = Hotring Racer A cagir (yaris arabasi)
cheat-just-try-and-stop-me = Hotring Racer B cagir (yaris arabasi)
cheat-wheres-the-funeral = Romero cagir (cenaze arabasi)
cheat-celebrity-status = Stretch Limousine cagir (Limuzin)
cheat-true-grime = Trashmaster cagir (cop arabasi)
cheat-18-holes = Caddy cagir (golf arabasi)
cheat-jump-jet = Hydra cagir (VTOL atak jeti)
cheat-i-want-to-hover = Vortex cagir (suzulebilen arac)
cheat-oh-dude = Hunter cagir (askeri atak helikopteri)
cheat-four-wheel-fun = Quad cagir (ATV/Dort teker)
cheat-hit-the-road-jack = Tanker ve kasasini cagir (tanker kamyonu)
cheat-its-all-bull = Dozer cagir (bulldozer)
cheat-flying-to-stunt = Stunt Plane cagir (gosteri ucagi)
cheat-monster-mash = Monster Truck cagir (canavar kamyon)
## Gang recruitment
cheat-wanna-be-in-my-gang = Silahini dogrultarak herkese cetene ekleyebil ve onlara tabanca ver
cheat-noone-can-stop-us = AK47 dogrultarak herkesi cetene ekleyebil ve onlara AK47 ver
cheat-rocket-mayhem = RPG dogrultarak herkesi cetene ekleyebil ve onlara RPG ver
## Traffic
cheat-all-drivers-are-criminals = Butun npc soforlerin aranmasi var ve agresif suruyor
cheat-pink-is-the-new-cool = Pembe trafik
cheat-so-long-as-its-black = Siyah trafik
cheat-everyone-is-poor = Fakir trafik
cheat-everyone-is-rich = Spor araba trafik
## Pedestrians
cheat-rough-neighbourhood = Oyuncuya beyzbol sopasi verir ve ic savas cikar
cheat-stop-picking-on-me = Siviller oyuncuya saldirir
cheat-surrounded-by-nutters = Sivillere silah ver
cheat-blue-suede-shoes = Butun siviller Elvis Presley
cheat-attack-of-the-village-people = Siviller oyuncuya silahlarla ve roketlerle saldirir
cheat-only-homies-allowed = Her yerde cete uyeleri
cheat-better-stay-indoors = Ceteler sokaklari kontrol eder
cheat-state-of-emergency = Ic savas
cheat-ghost-town = Az trafik ve hic sivil yok
## Themes
cheat-ninja-town = Ninja temasi
cheat-love-conquers-all = Pezevenk temasi
cheat-lifes-a-beach = Plaj partisi temasi
cheat-hicksville = Fakir temasi
cheat-crazy-town = Karnaval temasi
## General vehicle cheats
cheat-all-cars-go-boom = Butun araclari patlat
cheat-wheels-only-please = Gorunmez araclar
cheat-sideways-wheels = Arabalarin yatay tekeri var
cheat-speed-freak = Butun araclarda nitro
cheat-cool-taxis = Taksilerin suspansiyonu ve nitrosu var
## Vehicle cheats for the player
cheat-chitty-chitty-bang-bang = Ucan arabalad
cheat-cj-phone-home = Bisikletle cok yuksek ziplama
cheat-touch-my-car-you-die = Diger araclara dokundugunda onlari patlaf
cheat-bubble-cars = Arabalara vurdugunda ucmaya vaslasinlar
cheat-stick-like-glue = Gelismis suspansiyon ve vites
cheat-dont-try-and-stop-me = Trafik isiklari her zaman yesil
cheat-flying-fish = Ucan tekneler
## Weapon usage
cheat-full-clip = Herkesin sinirsiz mermisi var
cheat-i-wanna-driveby = Aracta full drive-by kontrolu
## Player effects
cheat-goodbye-cruel-world = Intihar et
cheat-take-a-chill-pill = Adrealin efekti
cheat-prostitutes-pay = Fahiseler sana para oduyor
## Miscellaneous
cheat-xbox-helper = XBOX basarimlarina yakinlasmak icin statlari degistir
## Pointless cheats

# Tells the user that a cheat will ALWAYS crash their game.
-cheat-crash-warning = COKERTIR!
cheat-slot-melee = { -cheat-crash-warning } El slotu
cheat-slot-handgun = { -cheat-crash-warning } Tabanca slotu
cheat-slot-smg = { -cheat-crash-warning } SMG slotu
cheat-slot-shotgun = { -cheat-crash-warning } Pompali slotu
cheat-slot-assault-rifle = { -cheat-crash-warning } Taramali tufek slotu
cheat-slot-long-rifle = { -cheat-crash-warning } Tufek slotu
cheat-slot-thrown = { -cheat-crash-warning } Atilabilir patlayici slotu
cheat-slot-heavy = { -cheat-crash-warning } Agir silah slotu
cheat-slot-equipment = { -cheat-crash-warning } Ekipman slotu
cheat-slot-other = { -cheat-crash-warning } Diger slotu
cheat-predator = Hicbir sey yapmiyor
