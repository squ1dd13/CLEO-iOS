# Used in the settings menu to show the name of the language.
language-name = ខ្មែរ

# Shown when this language has been selected automatically.
language-auto-name = ស្វ័យប្រវត្តិ (ខ្មែរ)

# The name of the language setting.
language-opt-title = ភាសាខ្មែរ

# The language setting description.
language-opt-desc = ភាសាសំរាប់ប្រើក្នុងឃ្លីអូ. របៀបដំណើការស្វ័យប្រវត្តិនឹងប្រើការកំណត់ប្រព័ន្ធរបស់អ្នក.អ្នកអាចស្នើសុំដាក់ភាសារបស់អ្នកនៅបណ្ដាញសង្គម Discord!

### ==== Splash screen ====

# First line at the bottom of the screen.
splash-legal = កម្មសិទ្ធបញ្ញា © 2020-2023 squ1dd13, AYZM, Flylarb, ODIN, RAiZOK, tharryz, wewewer1. មានអាជ្ញាប័ណ្ណក្រោមអាជ្ញាប័ណ្ណ MIT.

# Second line.
splash-fun = ធ្វើចេញពីក្ដីស្រលាញ់,សប្បាយៗ! បកប្រែជាភាសារខ្មែរដោយ BabeODIN.

### ==== Updates ====

# PLEASE NOTE: The update menu is currently made from a game menu, and the game fonts do not
# support non-ASCII characters. Please write with ASCII only here. If you can't do that, leave it
# in English for now. I plan to add Unicode support here in the future.

# Displayed at the top of the update screen.
update-prompt-title = មានកំណែអាប់ដេតថ្មី

# Message shown on the update screen. { $new_version } will be replaced with the update's version number.
update-prompt-message = កំណែកម្មវិធី CLEO { $new_version } មានកំណែទំរង់ថ្មី, តើអ្នកចង់ចូលទៅ GitHub ដើម្បីទាញយកទេ?

# todo: Add "Yes" and "No" for update menu to localisation files.
# The yes/no options are part of the game, so they're not directly in CLEO's control (yet).

## Release channel settings

update-release-channel-opt-title = ឆាែនលចេញផ្សាយ
update-release-channel-opt-desc = ការធ្វើបច្ចុប្បន្នភាព CLEO អ្នកនឹងទទួលបានការជូនដំណឹង។ អាល់ហ្វា ផ្តល់មុខងារថ្មីៗកាន់តែឆាប់ប៉ុន្តែអាចមានបញ្ហាច្រើន។យើងមិនណែនាំអោយបិទការធ្វើបច្ចុប្បន្នភាពទេ.
update-release-channel-opt-disabled = បិទ
update-release-channel-opt-stable = ស្ថិរភាព
update-release-channel-opt-alpha = អាល់ហ្វា

### ==== Menu ====

# Title for the button at the bottom of the screen that closes the CLEO menu.
menu-close = បិទមុឺនុយ

# Title for the options tab.
menu-options-tab-title = ជំម្រើស

### ==== Script menu ====

## Script warning overview

# Shown at the top of the script menus when at least one error has been found in any script. This
# is not shown at all if there are zero scripts with errors in them.
menu-script-warning-overview =
    { $num_scripts_with_errors ->
        [one] មានបញ្ហានៅក្នុងស្គ្រីបមួយនេះ. ស្គ្រីបនេះនឹងចេញពណ៌ទឹកក្រូច.
        *[other] មានបញ្ហាក្នុង { $num_scripts_with_errors } ស្គ្រីបទាំងនេះត្រូវបានបន្លិចជាពណ៌ទឹកក្រូច
    }

# The second line of the warning.
menu-script-see-below = សូមមើលខាងក្រោមសម្រាប់ព័ត៌មានលម្អិត.
menu-script-csa-tab-title = CSA
menu-script-csi-tab-title = CSI

## Specific script warnings

# The script does things that CLEO doesn't support yet.
script-unimplemented-in-cleo = ប្រើមុខងារដែលបច្ចុប្បន្នមិនទាន់គាំទ្រនៅលើ CLEO IOS

# The script does things that are not possible on iOS (for system reasons).
script-impossible-on-ios = ប្រើកូដដែលមិនដំណើរការនៅលើ IOS.

# The script is identical to another script. { $original_script } will be replaced with the name of
# the script that this one is a duplicate of.
script-duplicate = ស្គ្រីបដូចគ្នានឹង { $original_script }.

# There was an error when checking the script code for problems.
script-check-failed = មិនអាចស្កេនស្គ្រីបបានទេ។  សូមរាយការណ៍ថានេះជាបញ្ហាBUGនៅលើ GitHub ឬ Discord.

# No problems were found when scanning the script. This is a safe script!
script-no-problems = ស្គ្រីបមិនមានបញ្ហាទេ

# Formats for script names in the menu.
script-csa-row-title = { $script_name }
script-csi-row-title = { $script_name }

## Script status messages

# The script is running normally.
script-running = ដំណើរការ

# The script is not running.
script-not-running = មិនដំណើរការ

# The script has been forced to run by the user, even though there are problems with it. This only
# applies to CSA scripts.
script-csa-forced-running = បង្ខំអោយដំណើរការ

## Script settings

script-mode-opt-title = របៀបដំណើរការស្គ្រីប
script-mode-opt-desc = ផ្លាស់ប្តូររបៀបដែល CLEO ដំណើរការកូដស្គ្រីប. សាកល្បងផ្លាស់ប្តូរវាប្រសិនបើអ្នកកំពុងជួបប្រទះបញ្ហាជាមួយស្គ្រីប។

# In "don't break" mode, CLEO won't try to reduce script lag. This might be more stable sometimes.
script-mode-opt-dont-break = យឺត

# In "break" mode, CLEO will try to reduce script lag caused by long loops in script code.
script-mode-opt-break = លឿន

### ==== FPS ====

## FPS lock option

fps-lock-opt-title = ល្បឿនអតិបរមា
fps-lock-opt-desc =  ល្បឿនអតិបរមាដែលហ្គេមនឹងដំណើរការ. 30 FPS មើលទៅរាងយឺតបន្តិចប៉ុន្តែសន្សំសំចៃថ្ម
fps-lock-opt-30 = 30 FPS
fps-lock-opt-60 = 60 FPS

## FPS counter option

fps-counter-opt-title = បង្ហាញល្បឿន
fps-counter-opt-desc = បើកឬបិទការបង្ហាញល្បឿននៅលើអេក្រង់.
fps-counter-opt-hidden = លាក់បិទ
fps-counter-opt-enabled = បើកបង្ហាញ

### ==== Cheat system ====

## Menu

cheat-tab-title = ហេក

# Two lines of text shown at the top of the cheats menu.
cheat-menu-warning = ការប្រើហេកអាចនាំឱ្យគាំងឬអាចបាត់បង់ការរក្សាទុកហ្គេម.
  ប្រសិនបើអ្នកមិនចង់ប្រថុយបាត់បង់ការរក្សាទុកហ្គេមរបស់អ្នកទេសូមធ្វើការរក្សាទុកហ្គេមរបស់អ្នកទៅកាន់កន្លែងផ្ទុកផ្សេងជាមុនសិន.

## Status messages for cheats

cheat-on = បើកហេក
cheat-off = បិទហេក
# Cheat will be turned on when the menu is closed.
cheat-queued-on = ត្រៀមបើក
# Cheat will be turned off when the menu is closed.
cheat-queued-off = ត្រៀមបិទ
# Formats for cheat codes in the menu.
cheat-code-row-title = { $cheat_code }
cheat-no-code-title = ???

## Cheat saving option

cheat-transience-opt-title = របៀបរក្សាទុកដំណើរការហេក
cheat-transience-opt-desc = កំណត់ដំណើរការហេកនៅពេលបិទបើកហ្គេមសារជាថ្មី
cheat-transience-opt-transient = កំណត់ឡើងវិញទាំងអស់.
cheat-transience-opt-persistent = រក្សាទុក

### ==== Cheat descriptions ====

## Weapon sets
cheat-thugs-armoury = សំណុំអាវុធលេខ១
cheat-professionals-kit = សំណុំអាវុធលេខ២
cheat-nutters-toys = សំណុំអាវុធលេខ៣
cheat-weapons-4 = ពងជ័រ,កាំភ្លើងយន្តនិងវែនតាមើលយប់.

## Debug cheats
cheat-debug-mappings = Debug (បង្ហាយការចុចលើតួអង្គ)
cheat-debug-tap-to-target = Debug (ចុចលើគោលដៅ)
cheat-debug-targeting = Debug (បង្ហាយគោលដៅ)

## Properly cheating
cheat-i-need-some-help = ផ្ដល់ឈាម,អាវក្រស់កាពារនិងលុយ ២៥០,០០០$.
cheat-skip-mission = រំលងបេសកកម្ម

## Superpowers
cheat-full-invincibility = អត់ចេះងាប់អេខេអេជីវិតអមតៈ
cheat-sting-like-a-bee = មួយដៃងាប់
cheat-i-am-never-hungry = អត់ចេះឃ្លាន
cheat-kangaroo = ហក់ខ្ពស់ជាងមុន១០ដង
cheat-noone-can-hurt-me = អត់ចេះអស់ឈាម
cheat-man-from-atlantis = មុជទឹកបានរហូតអេខេអេអាឃ្វោមេន

## Social player attributes
cheat-worship-me = ក្លាយជាបងធំនៅក្នុងក្រុមកូនចៅគោរព
cheat-hello-ladies = ជើងខ្លាំងលើគ្រែ

## Physical player attributes
cheat-who-ate-all-the-pies = អាកាធាត់សុីជេ
cheat-buff-me-up = សាច់ដុំប្រាំមួយកង់
cheat-max-gambling = ស្ដេចល្បែងអាចែ
cheat-lean-and-mean = មាឌម៉ាល្មមស្តង់ដារ
cheat-i-can-go-all-night = រត់,ហែលទឹកអត់ចេះហត់

## Player skills
cheat-professional-killer = ជំនាញគ្រប់កាំភ្លើងទាំងអស់អេខេអេ ចនវីក
cheat-natural-talent = កំពូលអ្នកប្រណាំងបើកបរ

## Wanted level
cheat-turn-up-the-heat = ឡើងផ្កាយ២
cheat-turn-down-the-heat = អស់ផ្កាយ
cheat-i-do-as-i-please = ចាក់សោរមិនអោយឡើងឬធ្លាក់ផ្កាយ
cheat-bring-it-on = ផ្កាយ៦អេខេអេ រ៉ាស្មាច់

## Weather
cheat-pleasantly-warm = ក្ដៅ
cheat-too-damn-hot = ក្ដៅចោលម្រាយ
cheat-dull-dull-day = មេឃស្រទុំ
cheat-stay-in-and-watch-tv = ភ្លៀង
cheat-cant-see-where-im-going = ចុះអាប់
cheat-scottish-summer = ព្យុះ
cheat-sand-in-my-ears = ព្យុះដែរតែព្យុះខ្សាច់

## Time
cheat-clock-forward = បង្វិលពេលទៅមុខ៤ម៉ោង
cheat-time-just-flies-by = ម៉ោងដើរលឿន
cheat-speed-it-up = ពេលវេលាដើរលឿន
cheat-slow-it-down = ពេលវេលាដើរយឺត
cheat-night-prowler = ជាប់ត្រឹមម៉ោង១២
cheat-dont-bring-on-the-night = ជាប់ត្រឹមម៉ោង៩

## Spawning wearables
cheat-lets-go-base-jumping = ឆត័យោង
cheat-rocketman = អាវហោះ

## Spawning vehicles

# The vehicle names (in capitals) should not be changed, as they are part of the game. The
# descriptions (in brackets) do need to be translated.
cheat-time-to-kick-ass = Spawn Rhino (រថក្រោះ)
cheat-old-speed-demon = Spawn Bloodring Banger (ឡានពាសដែក)
cheat-tinted-rancher = Spawn Rancher with tinted windows (ឡានរុកព្រៃ)
cheat-not-for-public-roads = Spawn Hotring Racer A (ឡានប្រណាំង ក)
cheat-just-try-and-stop-me = Spawn Hotring Racer B (ឡានប្រណាំង ខ)
cheat-wheres-the-funeral = Spawn Romero (ឡានក្ដិតវែង)
cheat-celebrity-status = Spawn Stretch Limousine (ឡានសេដ្ធី)
cheat-true-grime = Spawn Trashmaster (ឡានសំរាម)
cheat-18-holes = Spawn Caddy (ឡានវាយកូនហ្គោល)
cheat-jump-jet = Spawn Hydra (យន្តហោះចំម្បាំង)
cheat-i-want-to-hover = Spawn Vortex (ទូកចេះហោះ)
cheat-oh-dude = Spawn Hunter (ហេឡេកុបទ័រចំម្បាំង)
cheat-four-wheel-fun = Spawn Quad (ម៉ូតូកង់បួន)
cheat-hit-the-road-jack = Spawn Tanker and trailer (ឡានកុងតាន័រ)
cheat-its-all-bull = Spawn Dozer (ឡានឈូសឆាយផ្លូវ)
cheat-flying-to-stunt = Spawn Stunt Plane (យន្តហោះតូច)
cheat-monster-mash = Spawn Monster Truck (ឡានកង់ធំ)

## Gang recruitment
cheat-wanna-be-in-my-gang = ជ្រើសរើសនរណាម្នាក់ចូលក្នុងក្រុមរបស់អ្នកហើយផ្តល់ឱ្យពួកគេនូវកាំភ្លើងខ្លីមួយដើម ដោយភ្ជង់កាំភ្លើងខ្លីលើពួកគេ.
cheat-noone-can-stop-us = ជ្រើសរើសនរណាម្នាក់ចូលក្នុងក្រុមរបស់អ្នកហើយផ្តល់ឱ្យពួកគេនូវកាំភ្លើងអាកា៤៧មួយដើម ដោយភ្ជង់កាំភ្លើងអាកា៤៧លើពួកគេ.
cheat-rocket-mayhem = ជ្រើសរើសនរណាម្នាក់ចូលក្នុងក្រុមរបស់អ្នកហើយផ្តល់ឱ្យពួកគេនូវអាបេមួយដើម ដោយភ្ជង់អាបេលើពួកគេ.
## Traffic
cheat-all-drivers-are-criminals = គ្រប់គ្នាបើកឡានដូចអ្នកស្រវឹង
cheat-pink-is-the-new-cool = គ្រប់គ្នាបើកឡានពណ៌ផ្កាឈូក
cheat-so-long-as-its-black = គ្រប់គ្នាបើកឡានពណ៌ខ្មៅ
cheat-everyone-is-poor = គ្រប់គ្នាបើកឡានធ្វើស្រែ
cheat-everyone-is-rich = គ្រប់គ្នាបើកឡានឡូយៗ
## Pedestrians
cheat-rough-neighbourhood = ទីក្រុងចលាចលយកដំបងកូនហ្គោលវាយគ្នា
cheat-stop-picking-on-me = គ្រន់គ្នាតាមសំលាប់អ្នក
cheat-surrounded-by-nutters = អោយមានកាំភ្លើងកាន់គ្រប់គ្នា
cheat-blue-suede-shoes = នគរប្រុសល្វោ
cheat-attack-of-the-village-people = គ្រប់គ្នាតាមបាញ់សំលាប់អ្នកមានទាំងអាបេទៀត
cheat-only-homies-allowed = ទីក្រុងក្មេងពាល
cheat-better-stay-indoors = ពាលគុណ២
cheat-state-of-emergency = ទីក្រុងចលាចល,សង្គ្រាមស៊ីវិល
cheat-ghost-town = ទីក្រុងខ្មោច
## Themes
cheat-ninja-town = ទីក្រុងអ្នកលេងដាវសាម៉ូរៃ
cheat-love-conquers-all = ទីក្រុងញៀនឆិច
cheat-lifes-a-beach = បុណ្យសមុទ្រ
cheat-hicksville = ទីក្រុងជនបទ
cheat-crazy-town = ទីក្រុងខេអ្វេសសុី
## General vehicle cheats
cheat-all-cars-go-boom = បំផ្ទុះឡានទាំងអស់
cheat-wheels-only-please = ឡានឃើញតែកង់
cheat-sideways-wheels = ឡានកង់ទទឹង
cheat-speed-freak = គ្រប់ឡានមានហ្គាស់ទាំងអស់
cheat-cool-taxis = ឡានតាក់សុីកែញាក់បូម,មានហ្គាស់
## Vehicle cheats for the player
cheat-chitty-chitty-bang-bang = ឡានហោះ
cheat-cj-phone-home = កង់លោតបានខ្ពស់អេខេអេកង់បែកខេ
cheat-touch-my-car-you-die = ឡានបុកផ្ទុះ
cheat-bubble-cars = ឡានបុកហោះ
cheat-stick-like-glue = ចង្កូតនឹងរឺស័រល្អជាងមុន
cheat-dont-try-and-stop-me = ភ្លើងស្ដុបខៀវរហូត
cheat-flying-fish = ទូកហោះទូកហើរ
## Weapon usage
cheat-full-clip = តួហុងកុង,កាំភ្លើងអត់ចេះអស់គ្រាប់
cheat-i-wanna-driveby = បាញ់កាំភ្លើងពេលបើកបរ
## Player effects
cheat-goodbye-cruel-world = ជ្រោយចង្វា,សម្លាប់ខ្លួន
cheat-take-a-chill-pill = បែកថ្នាំ
cheat-prostitutes-pay = ស្រីរកសុីអោយលុយពេលកច់ៗក្នុងឡាន
## Miscellaneous
cheat-xbox-helper = Adjust stats to be close to getting Xbox achievements
## Pointless cheats

# Tells the user that a cheat will ALWAYS crash their game.
cheat-crash-warning = គាំង!!
cheat-slot-melee = { cheat-crash-warning } កន្លែងផ្នុក កាំបិត
cheat-slot-handgun = { cheat-crash-warning } កន្លែងផ្នុក កាំភ្លើងខ្លី
cheat-slot-smg = { cheat-crash-warning } កន្លែងផ្នុក កាំភ្លើងអេសអឹមជី
cheat-slot-shotgun = { cheat-crash-warning } កន្លែងផ្នុក កាំភ្លើងស្នប់
cheat-slot-assault-rifle = { cheat-crash-warning } កន្លែងផ្នុក កាំភ្លើងវែង
cheat-slot-long-rifle = { cheat-crash-warning } កន្លែងផ្នុក កាំភ្លើងស្នេប
cheat-slot-thrown = { cheat-crash-warning } កន្លែងផ្នុក គ្រាប់បែក
cheat-slot-heavy = { cheat-crash-warning } កន្លែងផ្នុក អាវុធធន់ធ្ងន់
cheat-slot-equipment = { cheat-crash-warning } កន្លែងផ្នុក គ្រឿងប្រដាប់.បរិក្ខារ
cheat-slot-other = { cheat-crash-warning } កន្លែងផ្ទុក ផ្សេងៗទៀត
cheat-predator = ទទេរគឺទទេរ
