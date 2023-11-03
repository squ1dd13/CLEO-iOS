### ==== Language settings ====

# Used in the settings menu to show the name of the language.
language-name = Русский

# Shown when this language has been selected automatically.
language-auto-name = Автоматически: ({ language-name })

# The name of the language setting.
language-opt-title = Язык

# The language setting description.
language-opt-desc = Язык, использующийся для CLEO. Автоматический режим будет использовать ваши системные настройки. Добавьте свой язык в Discord!

### ==== Splash screen ====

# First line at the bottom of the screen.
splash-legal = Авторские права © 2020-2023 { $copyright_names }. Лицензия MIT.

# Second line.
splash-fun = Сделано с любовью в Англии. Удачной игры!

### ==== Updates ====

# PLEASE NOTE: The update menu is currently made from a game menu, and the game fonts do not
# support non-ASCII characters. Please write with ASCII only here. If you can't do that, leave it
# in English for now. I plan to add Unicode support here in the future.

# Displayed at the top of the update screen.
update-prompt-title = Update Available

# Message shown on the update screen. { $new_version } will be replaced with the update's version number.
update-prompt-message = CLEO Version { $new_version } is available. Do you want to go to GitHub to download it?

# todo: Add "Yes" and "No" for update menu to localisation files.
# The yes/no options are part of the game, so they're not directly in CLEO's control (yet).

## Release channel settings

update-release-channel-opt-title = Канал релизов
update-release-channel-opt-desc = Какие уведомления об обновлениях CLEO вы будете получать. Режим «Альфа» дает возможность к будущим возможностям, но будут иметь много багов. Отключать уведомления не рекомендуется.

update-release-channel-opt-disabled = Отключено
update-release-channel-opt-stable = Стабильно
update-release-channel-opt-alpha = Альфа

### ==== Menu ====

# Title for the button at the bottom of the screen that closes the CLEO menu.
menu-close = Закрыть

# Title for the options tab.
menu-options-tab-title = Настройки

## Menu gesture settings

menu-gesture-opt-title = Меню жестов
menu-gesture-opt-desc = Потребуется касания пальцов, чтоб открыть CLEO меню.

# A single motion where one finger moves quickly down the screen.
menu-gesture-opt-one-finger-swipe = Листание вниз одним пальцем

# A single swipe (as above) but with two fingers at the same time instead of just one.
menu-gesture-opt-two-finger-swipe = Листание вниз двумя пальцами

# A short tap on the screen with two fingers at once.
menu-gesture-opt-two-finger-tap = Касание двумя пальцами

# A short tap on the screen with three fingers at once.
menu-gesture-opt-three-finger-tap = Касание тремя пальцами

### ==== Script menu ====

## Script warning overview

# Shown at the top of the script menus when at least one error has been found in any script. This
# is not shown at all if there are zero scripts with errors in them.
menu-script-warning-overview =
    { $num_scripts_with_errors ->
        [one] Найдена неполадка в скрипте. Скрипт помечен оранжевым.
        *[other] Найдены неполадки в { $num_scripts_with_errors } скриптах. Скрипты помечены оранжевым.
    }

# The second line of the warning.
menu-script-see-below = Узнать подробности.

menu-script-csa-tab-title = CSA
menu-script-csi-tab-title = CSI

## Specific script warnings

# The script does things that CLEO doesn't support yet.
script-unimplemented-in-cleo = Использует возможности которые пока не доступны для CLEO iOS.

# The script does things that are not possible on iOS (for system reasons).
script-impossible-on-ios = Используется код, который не работает на iOS.

# The script is identical to another script. { $original_script } будет заменен
# the script that this one is a duplicate of.
script-duplicate = Дубликат { $original_script }.

# There was an error when checking the script code for problems.
script-check-failed = Не удается сканировать скрипт. Пожалуйста, сообщите что это баг либо через GitHub или Discord.

# No problems were found when scanning the script. This is a safe script!
script-no-problems = Неполадок не обнаружено.

# Formats for script names in the menu.
script-csa-row-title = { $script_name }
script-csi-row-title = { $script_name }

## Script status messages

# The script is running normally.
script-running = Работает

# The script is not running.
script-not-running = Не работает

# The script has been forced to run by the user, even though there are problems with it. This only
# applies to CSA scripts.
script-csa-forced-running = Прин. работает

## Script settings

script-mode-opt-title = Режим процесса скрипта
script-mode-opt-desc = Изменяет процесс скрипта для CLEO. Попытайтесь поменять это, если у вас возникают проблемы со скриптами.

# In "don't break" mode, CLEO won't try to reduce script lag. This might be more stable sometimes.
script-mode-opt-dont-break = Медленно

# In "break" mode, CLEO will try to reduce script lag caused by long loops in script code.
script-mode-opt-break = Быстро

### ==== FPS ====

## FPS lock option

fps-lock-opt-title = Ограничитель кадров
fps-lock-opt-desc = Макс. кол-во кадров, которая игра может использовать. 30 кадров выглядит похуже, зато это будет экономить заряд.

fps-lock-opt-30 = 30 к/с
fps-lock-opt-60 = 60 к/с

## FPS counter option

fps-counter-opt-title = Счетчик кадров в секунду
fps-counter-opt-desc = Включает или выключает внутриигровой счетчик кадров.

fps-counter-opt-hidden = Выкл.
fps-counter-opt-enabled = Вкл.

### ==== Cheat system ====

## Menu

cheat-tab-title = Читы

# Two lines of text shown at the top of the cheats menu.
cheat-menu-warning = Использование читов может привести к вылетам и даже к потере игрового прогресса.
  Если не хотите рисковать сохранением, создайте резерв. копию в другом слоте сохранения.

## Status messages for cheats

cheat-on = Вкл.
cheat-off = Выкл.

# Cheat will be turned on when the menu is closed.
cheat-queued-on = Активируется

# Cheat will be turned off when the menu is closed.
cheat-queued-off = Деактивируется

# Formats for cheat codes in the menu.
cheat-code-row-title = { $cheat_code }
cheat-no-code-title = ???

## Cheat saving option

cheat-transience-opt-title = Сохранения читов
cheat-transience-opt-desc = Контролирует состояния читов при запуске/перезапуске игры.

cheat-transience-opt-transient = Сбросить все
cheat-transience-opt-persistent = Сохранить статистику

### ==== Cheat descriptions ====

## Weapon sets
cheat-thugs-armoury = Набор оружия 1
cheat-professionals-kit = Набор оружия 2
cheat-nutters-toys = Набор оружия 3
cheat-weapons-4 = Дать дилдо, миниган и очки теплового/ночного видения

## Debug cheats
cheat-debug-mappings = Отладка (показывать маппинги)
cheat-debug-tap-to-target = Отладка (показывать прикосновение к цели)
cheat-debug-targeting = Отладка (показывать прицеливание)

## Properly cheating
cheat-i-need-some-help = Дать здоровье, броню и $250,000
cheat-skip-mission = Пропустить прохождение некоторых миссий

## Superpowers
cheat-full-invincibility = Неуязвимость
cheat-sting-like-a-bee = Сильные удары
cheat-i-am-never-hungry = Игрок не голодает
cheat-kangaroo = Усиление прыжка в 10 раз
cheat-noone-can-hurt-me = Бесконечное здоровье
cheat-man-from-atlantis = Бесконечный объем легких

## Social player attributes
cheat-worship-me = Максимальное уважение
cheat-hello-ladies = Максимальная сексуальность

## Physical player attributes
cheat-who-ate-all-the-pies = Жир на максимум
cheat-buff-me-up = Мускулы на максимум
cheat-max-gambling = Макс. азартный навык
cheat-lean-and-mean = Жир и мускулы на минимум
cheat-i-can-go-all-night = Макс. сексуальность

## Player skills
cheat-professional-killer = Уровень «Киллер» для всех видов оружия
cheat-natural-talent = Макс. Навык вождения

## Wanted level
cheat-turn-up-the-heat = Увеличить уровень розыска на 2 звезды
cheat-turn-down-the-heat = Отчистить уровень розыска
cheat-i-do-as-i-please = Оставить уровень розыска до текущего значения
cheat-bring-it-on = Макс. уровень розыска

## Weather
cheat-pleasantly-warm = Солнечная погода
cheat-too-damn-hot = Очень солнечная погода
cheat-dull-dull-day = Пасмурная погода
cheat-stay-in-and-watch-tv = Дождливая погода
cheat-cant-see-where-im-going = Туман
cheat-scottish-summer = Грозовая погода
cheat-sand-in-my-ears = Песчаная буря

## Time
cheat-clock-forward = Промотать время на 4 часа вперед
cheat-time-just-flies-by = Быстрое время
cheat-speed-it-up = Быстрый игр. процесс
cheat-slow-it-down = Медленный игр. процесс
cheat-night-prowler = Всегда полночь
cheat-dont-bring-on-the-night = Всегда 9 часов вечера.

## Spawning wearables
cheat-lets-go-base-jumping = Получить парашют
cheat-rocketman = Получить реактивный ранец (Джетпак)

## Spawning vehicles

# The vehicle names (in capitals) should not be changed, as they are part of the game. The
# descriptions (in brackets) do need to be translated.
cheat-time-to-kick-ass = Заспавнить Rhino (танк)
cheat-old-speed-demon = Заспавнить Bloodring Banger (Машина для дерби)
cheat-tinted-rancher = Заспавнить Rancher с тонированными окнами (Двухдверный внедорожник)
cheat-not-for-public-roads = Заспавнить Hotring Racer A (гоночная машина)
cheat-just-try-and-stop-me = Заспавнить Hotring Racer B (гоночная машина)
cheat-wheres-the-funeral = Заспавнить Romero (гробовозка)
cheat-celebrity-status = Заспавнить лимузин Stretch (лимузин)
cheat-true-grime = Заспавнить Trashmaster (мусоровоз)
cheat-18-holes = Заспавнить Caddy (гольф-карт)
cheat-jump-jet = Заспванить Hydra (истребитель)
cheat-i-want-to-hover = Заспавинть Vortex (судно на воздушной подушке)
cheat-oh-dude = Заспавнить Hunter (Военный вертолет)
cheat-four-wheel-fun = Заспавнить Quad (квадроцикл)
cheat-hit-the-road-jack = Заспавнить Tanker и прицеп (грузовик с прицепом)
cheat-its-all-bull = Заспавнить Dozer (бульдозер)
cheat-flying-to-stunt = Заспавнить Stunt Plane (трюковой самолет)
cheat-monster-mash = Заспавнить Monster Truck (монстр)

## Gang recruitment
cheat-wanna-be-in-my-gang = Нанять кого-то в банду а затем дать пистолет прицеливанием
cheat-noone-can-stop-us = Нанять кого-то в банду  а затем дать AK-47 прицеливанием
cheat-rocket-mayhem = Нанять кого-то в банду а затем дать RPG прицеливанием

## Traffic
cheat-all-drivers-are-criminals = Все NPC-водители водят агрессивно имея уровень розыска за собой
cheat-pink-is-the-new-cool = Трафик из розовых авто
cheat-so-long-as-its-black = Трафик из черных авто
cheat-everyone-is-poor = Сельский трафик
cheat-everyone-is-rich = Трафик из спорткаров

## Pedestrians
cheat-rough-neighbourhood = Дать игроку клюшку для гольфа и заставить пешеходов бунтовать
cheat-stop-picking-on-me = Пешеходы атакуют игрока
cheat-surrounded-by-nutters = Дать пешеходам оружие
cheat-blue-suede-shoes = Все пешеходы заменяются Элвисами Пресли
cheat-attack-of-the-village-people = Пешеходы атакуют игрока пушками и ракетницами
cheat-only-homies-allowed = Члены банд повсюду
cheat-better-stay-indoors = Банды контролируют улицы
cheat-state-of-emergency = Бунт пешеходов
cheat-ghost-town = Уменьшенный трафик, на улице нет пешеходов

## Themes
cheat-ninja-town = Тема Триад
cheat-love-conquers-all = Тема сутенера
cheat-lifes-a-beach = Тема пляжной вечеринки
cheat-hicksville = Сельская тема
cheat-crazy-town = Карнавальная тема

## General vehicle cheats
cheat-all-cars-go-boom = Взорвать все машины
cheat-wheels-only-please = Невидимый транспорт
cheat-sideways-wheels = У машин теперь боковые колеса
cheat-speed-freak = У всех машин стоит нитро
cheat-cool-taxis = У такси стоит гидравлика и нитро

## Vehicle cheats for the player
cheat-chitty-chitty-bang-bang = Летающие машины
cheat-cj-phone-home = Высокий прыжок на велосипеде
cheat-touch-my-car-you-die = Взрывать машины при столкновении
cheat-bubble-cars = Машины при столкновении взлетают
cheat-stick-like-glue = Улучшенное управление авто
cheat-dont-try-and-stop-me = Светофоры горят зеленым
cheat-flying-fish = Летающие лодки

## Weapon usage
cheat-full-clip = У всех бесконечные патроны
cheat-i-wanna-driveby = Улучшенная стрельба из транспорта

## Player effects
cheat-goodbye-cruel-world = Самоубийство
cheat-take-a-chill-pill = Адреналин
cheat-prostitutes-pay = Проститутки платят вам

## Miscellaneous
cheat-xbox-helper = Поменять статистику чтоб быть ближе к получению достижений Xbox

## Pointless cheats

# Tells the user that a cheat will ALWAYS crash their game.
cheat-crash-warning = ВЫЛЕТЫ!

cheat-slot-melee = { cheat-crash-warning } Слот с оружием ближ. боя
cheat-slot-handgun = { cheat-crash-warning } Слот с пистолетом
cheat-slot-smg = { cheat-crash-warning } Слот с ППМ
cheat-slot-shotgun = { cheat-crash-warning } Слот с дробовиком
cheat-slot-assault-rifle = { cheat-crash-warning } Слот с винтовкой
cheat-slot-long-rifle = { cheat-crash-warning } Слот с длинной винтовкой
cheat-slot-thrown = { cheat-crash-warning } Слот с бросающимися орудиями
cheat-slot-heavy = { cheat-crash-warning } Слот с тяжелой артиллерией
cheat-slot-equipment = { cheat-crash-warning } Слот с экипировкой
cheat-slot-other = { cheat-crash-warning } Другой слот

cheat-predator = Ничего не делает
