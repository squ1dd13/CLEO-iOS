# Used in the settings menu to show the name of the language.
language-name = العربيه

### ==== Splash screen ====

# First line at the bottom of the screen.
splash-legal = الحقوق © 2020-2023 squ1dd13, AYZM, ODIN, RAiZOK, tharryz, wewewer1. MIT مرخصة بموجب ترخيص

# Second line.
splash-fun = صنعت وبحب في المملكة المتحدة.  ترجمت الى العربيه بواسطة RAiZOK. استمتع

### ==== Updates ====

# PLEASE NOTE: The update menu is currently made from a game menu, and the game fonts do not
# support non-ASCII characters. Please write with ASCII only here. If you can't do that, leave it
# in English for now. I plan to add Unicode support here in the future.

# Displayed at the top of the update screen.
update-prompt-title = يوجد تحديث متوفر

# Message shown on the update screen. { $new_version } will be replaced with the update's version number.
update-prompt-message = اصدار cleo { $new_version } متاح. هل تريد الانتقال إلى GitHub لتنزيله؟

# todo: Add "Yes" and "No" for update menu to localisation files.
# The yes/no options are part of the game, so they're not directly in CLEO's control (yet).

## Release channel settings

update-release-channel-opt-title = قناة النشر
update-release-channel-opt-desc = تحديثات cleo التي تحصل على إشعارات لها. يوفر alpha ميزات أحدث في وقت قريب ولكن قد يحتوي على المزيد من الأخطاء. لا ينصح بتعطيل التحديثات.

update-release-channel-opt-disabled = معطل
update-release-channel-opt-stable = مستقر
update-release-channel-opt-alpha = ألفا

### ==== Menu ====

# Title for the button at the bottom of the screen that closes the CLEO menu.
menu-close = اغلق

# Title for the options tab.
menu-options-tab-title = خيارات

### ==== Script menu ====

## Script warning overview

# Shown at the top of the script menus when at least one error has been found in any script. This
# is not shown at all if there are zero scripts with errors in them.
menu-script-warning-overview =
    { $num_scripts_with_errors ->
        [one] توجد مشكله في هذا السكربت. تم تمييز هذا السكربت باللون البرتقالي.
        *[other] وجدت مشاكل في { $num_scripts_with_errors } سكربت. تم تمييز هذه السكربتات باللون البرتقالي.
    }

# The second line of the warning.
menu-script-see-below = انظر أدناه للحصول على مزيد من التفاصيل.

## Specific script warnings

# The script does things that CLEO doesn't support yet.
script-unimplemented-in-cleo = الميزات المستخدمة غير مدعومة حاليًا بواسطة CLEO iOS.

# The script does things that are not possible on iOS (for system reasons).
script-impossible-on-ios = استخدمت بعض الاكواد البرمجية التي لا تعمل على iOS.

# The script is identical to another script. { $original_script } will be replaced with the name of
# the script that this one is a duplicate of.
script-duplicate = نسخة مكررة من { $original_script }.

# There was an error when checking the script code for problems.
script-check-failed = تعذر فحص السكربت. الرجاء الإبلاغ عن هذا باعتباره خطأ في GitHub أو Discord.

# No problems were found when scanning the script. This is a safe script!
script-no-problems = لم يتم الكشف عن مشاكل.

# Formats for script names in the menu.
script-csa-row-title = { $script_name }
script-csi-row-title = { $script_name }

## Script status messages

# The script is running normally.
script-running = قيد التشغيل

# The script is not running.
script-not-running = لا يعمل

# The script has been forced to run by the user, even though there are problems with it. This only
# applies to CSA scripts.
script-csa-forced-running = اجباري

## Script settings

script-mode-opt-title = وضع معالجة السكربت
script-mode-opt-desc = تغيرات كيفية معالجة cleo لاكواد السكربت. حاول تغيير هذا إذا كنت تواجه مشكلات في السكربتات.

# In "don't break" mode, CLEO won't try to reduce script lag. This might be more stable sometimes.
script-mode-opt-dont-break = بطيء

# In "break" mode, CLEO will try to reduce script lag caused by long loops in script code.
script-mode-opt-break = سريع

### ==== FPS ====

## FPS lock option

fps-lock-opt-title = حد الاطارات (فريمات)
fps-lock-opt-desc = الحد الأقصى لمعدل الإطارات (الفريمات) التي ستعمل بها اللعبة. 30 إطارًا في الثانية تبدو سيئة ولكنها توفر البطارية.

fps-lock-opt-30 = 30 اطار (فريم)
fps-lock-opt-60 = 60 اطار (فريم)

## FPS counter option

fps-counter-opt-title = عداد الاطارات (فريمات)
fps-counter-opt-desc = تمكين أو تعطيل عداد الاطارات (الفريمات) على الشاشة.

fps-counter-opt-hidden = معطل
fps-counter-opt-enabled = مفعل

### ==== Cheat system ====

## Menu

cheat-tab-title = قائمة الغش

# Two lines of text shown at the top of the cheats menu.
cheat-menu-warning = يمكن أن يؤدي استخدام الغش إلى حدوث أعطال وربما فقدان التقدم اللعبة.
  اذا كنت لا تريد المخاطرة في فقدان تقدمك, اعمل نسخ احتياطي لتقدمك في مكان اخر.

## Status messages for cheats

cheat-on = تغشيل
cheat-off = ايقاف

# Cheat will be turned on when the menu is closed.
cheat-queued-on = في الانتظار

# Cheat will be turned off when the menu is closed.
cheat-queued-off = الغي الانتظار

# Formats for cheat codes in the menu.
cheat-code-row-title = { $cheat_code }
cheat-no-code-title = ???

## Cheat saving option

cheat-transience-opt-title = حفظ الغش
cheat-transience-opt-desc = ستبقى اعدادات الغش كما هي عند اعادة تشغيل اللعبة

cheat-transience-opt-transient = إعادة ضبط الجميع
cheat-transience-opt-persistent = حفط الحالة

### ==== Cheat descriptions ====

## Weapon sets
cheat-thugs-armoury = مجموعة الاسلحه 1
cheat-professionals-kit = مجموعة الاسلحه 2
cheat-nutters-toys = مجموعة الاسلحه 3
cheat-weapons-4 = اعطاء قضيب, سلاح مينيغون ونظارات ليليه/رؤيه حراريه

## Debug cheats
cheat-debug-mappings = تصحيح (إظهار التعيينات)
cheat-debug-tap-to-target = تصحيح (النقر للاستهداف)
cheat-debug-targeting = تصحيح (إظهار الاستهداف)

## Properly cheating
cheat-i-need-some-help = الحصول على الصحة والدروع و 250.000 دولار
cheat-skip-mission = تخطي للانتهاء في بعض المهام

## Superpowers
cheat-full-invincibility = مناعة كاملة
cheat-sting-like-a-bee = اللكمات الخارقة
cheat-i-am-never-hungry = عدم الشعور بالجوع
cheat-kangaroo = قفزة 10 اضعاف
cheat-noone-can-hurt-me = صحة غير محدودة
cheat-man-from-atlantis = تنفس لا نهائي

## Social player attributes
cheat-worship-me =أقصى درجات الاحترام
cheat-hello-ladies = أقصى جاذبية جنسية

## Physical player attributes
cheat-who-ate-all-the-pies = الحد الاقصى للسمنه
cheat-buff-me-up = الحد الاقصى للعضلات
cheat-max-gambling = مهارة القمار القصوى
cheat-lean-and-mean = الحد الأدنى من السمنه والعضلات
cheat-i-can-go-all-night = أقصى قدرة على التحمل

## Player skills
cheat-professional-killer = مستوى قاتل لجميع الاسلحة
cheat-natural-talent = مهارات السياره القصوى

## Wanted level
cheat-turn-up-the-heat = زيادة النجمات بنجمتين
cheat-turn-down-the-heat = مسح جميع النجمات
cheat-i-do-as-i-please = قفل مستوى الطلوب على هذا القدر
cheat-bring-it-on = 6 نجمات

## Weather
cheat-pleasantly-warm = طقس مشمس
cheat-too-damn-hot = طقس مشمس جدا
cheat-dull-dull-day = طقس غائم
cheat-stay-in-and-watch-tv = طقس ممطر
cheat-cant-see-where-im-going = طقس ضبابي
cheat-scottish-summer = طقس عاصف
cheat-sand-in-my-ears = عاصفه رمليه

## Time
cheat-clock-forward = تقدم الساعة بمقدار 4 ساعات
cheat-time-just-flies-by = تسريع الوقت
cheat-speed-it-up = تسريع اللعبة
cheat-slow-it-down = تبطيء اللعبة
cheat-night-prowler = دائما في منتصف الليل
cheat-dont-bring-on-the-night = دائما 9 مساءاٌ.

## Spawning wearables
cheat-lets-go-base-jumping = احضار المظلة
cheat-rocketman = احضار حقيبة نفاثه

## Spawning vehicles

# The vehicle names (in capitals) should not be changed, as they are part of the game. The
# descriptions (in brackets) do need to be translated.
cheat-time-to-kick-ass = احضار Rhino (دبابة الجيش)
cheat-old-speed-demon = احضار Bloodring Banger (سيارة سباق)
cheat-tinted-rancher = احضار Rancher with tinted windows (سيارة لها بابين)
cheat-not-for-public-roads = احضار Hotring Racer A (سيارة سباق 1)
cheat-just-try-and-stop-me = احضار Hotring Racer B (سيارة سباق 2)
cheat-wheres-the-funeral = احضار Romero (سيارة الجنازة)
cheat-celebrity-status = احضار Stretch Limousine (ليموزين)
cheat-true-grime = احضار Trashmaster (شاحنة القمامة / لوري)
cheat-18-holes = احضار Caddy (عربة الكولف)
cheat-jump-jet = احضار Hydra (VTOL الطيارة الحربيه)
cheat-i-want-to-hover = احضار Vortex (سفينة تمشي على اليابسة والماء)
cheat-oh-dude = احضار Hunter (هليكوبتر الجيش)
cheat-four-wheel-fun = احضار Quad (دراجه اربع عجلات)
cheat-hit-the-road-jack = احضار Tanker and trailer (شاحنة الصهريج)
cheat-its-all-bull = احضار Dozer (جرافة)
cheat-flying-to-stunt = احضار Stunt Plane (طائرة العاب بهلوانية)
cheat-monster-mash = احضار Monster Truck (شاحنة المونستر)

## Gang recruitment
cheat-wanna-be-in-my-gang = تجنيد اي شخص من عصابتك واعطائه سلاح بالتقنيص عليهم بالمسدس
cheat-noone-can-stop-us = تجنيد اي شخص من عصابتك واعطائه AK-47 بالتقنيص عليهم بالAK-47
cheat-rocket-mayhem = تجنيد اي شخص في عصابتك واعطائه RPG بالتقنيص عليهم بال RPG

## Traffic
cheat-all-drivers-are-criminals = يقود جميع السائقين السيارات بقوة ولديهم مستوى المطلوب
cheat-pink-is-the-new-cool = جميع السيارات وورديه
cheat-so-long-as-its-black = جميع السيارات سوداء
cheat-everyone-is-poor = حركه مروريه ريفيه
cheat-everyone-is-rich = حركه مروريه سيارات سباق

## Pedestrians
cheat-rough-neighbourhood = اعطاء المارة مضربا للجولف وجعلهم في حاله شغب
cheat-stop-picking-on-me = المارة يهاجمونك
cheat-surrounded-by-nutters = اعطاء المارة اسلحه
cheat-blue-suede-shoes = جميع المارة بشخصيه إلفيس بريسلي
cheat-attack-of-the-village-people =يهاجمك المارة بالبنادق والصواريخ
cheat-only-homies-allowed = رجال العصابات في كل مكان
cheat-better-stay-indoors = العصابات تسيطر على الشوارع
cheat-state-of-emergency = المارة في حاله شغب
cheat-ghost-town = انخفاض حركة المرور وعدم وجود مشاة

## Themes
cheat-ninja-town = المارة بشخصيات اصحاب البدلات السود
cheat-love-conquers-all = المارة بشخصيات الدعارة
cheat-lifes-a-beach = المارة بشخصيات اصحاب الشاطئ
cheat-hicksville = المارة بخشخصيات الريف
cheat-crazy-town = المارة بشخصيات الحفلات

## General vehicle cheats
cheat-all-cars-go-boom = تفجير جميع المركبات
cheat-wheels-only-please = مركبات غير مرئية
cheat-sideways-wheels = السيارات لها عجلات جانبية
cheat-speed-freak = جميع السيارات لها نايترو
cheat-cool-taxis = جميع السيارات لها هيدروليك ونايترو

## Vehicle cheats for the player
cheat-chitty-chitty-bang-bang = السيارات تطير
cheat-cj-phone-home = قفزة عاليه جدا
cheat-touch-my-car-you-die = تدمير المركبات الأخرى عند الاصطدام
cheat-bubble-cars = تطفو السيارات بعيدًا عند اصطدامها
cheat-stick-like-glue = Improved suspension and handling
cheat-dont-try-and-stop-me = إشارات المرور خضراء دائمًا
cheat-flying-fish = القوارب تطير

## Weapon usage
cheat-full-clip = كل شخص لديه ذخيرة لا نهائية
cheat-i-wanna-driveby = التحكم الكامل للسلاح في المركبات

## Player effects
cheat-goodbye-cruel-world = انتحار
cheat-take-a-chill-pill = آثار الأدرينالين
cheat-prostitutes-pay = العاهرات تدفع لك

## Miscellaneous
cheat-xbox-helper = ضبط الإحصائيات لتكون قريبة من الحصول على إنجازات Xbox

## Pointless cheats

# Tells the user that a cheat will ALWAYS crash their game.
-cheat-crash-warning = خروج مفاجئ

cheat-slot-melee = { -cheat-crash-warning } خانة عراك
cheat-slot-handgun = { -cheat-crash-warning } خانة اسلحه يدويه
cheat-slot-smg = { -cheat-crash-warning } SMG خانة
cheat-slot-shotgun = { -cheat-crash-warning } خانة بنادق
cheat-slot-assault-rifle = { -cheat-crash-warning } خانة رشاشات
cheat-slot-long-rifle = { -cheat-crash-warning } خانة اسلحه طويله
cheat-slot-thrown = { -cheat-crash-warning } رمية فتحة السلاح
cheat-slot-heavy = { -cheat-crash-warning } خانة مدفعية ثقيلة
cheat-slot-equipment = { -cheat-crash-warning } خانة معدات
cheat-slot-other = { -cheat-crash-warning } خانات اخرى

cheat-predator = لا يفعل شيئا
