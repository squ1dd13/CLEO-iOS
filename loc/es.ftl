### ==== Language settings ====

# Used in the settings menu to show the name of the language.
language-name = Español

# Shown when this language has been selected automatically.
language-auto-name = Automático ({ language-name })

# The name of the language setting.
language-opt-title = Idioma

# The language setting description.
language-opt-desc = El idioma a usar para CLEO. El modo automático utilizará la configuración de su sistema. ¡Agrega tu propio idioma en Discord!

### ==== Splash screen ====

# First line at the bottom of the screen.
splash-legal = Derechos de Autor © 2020-2023 squ1dd13, AYZM, Flylarb, ODIN, RAiZOK, tharryz, wewewer1. Licenciado bajo la Licencia MIT.

# Second line.
splash-fun = Hecho con amor en el Reino Unido. ¡Divertirse! ¡Y no seas un busta!

### ==== Updates ====

# PLEASE NOTE: The update menu is currently made from a game menu, and the game fonts do not
# support non-ASCII characters. Please write with ASCII only here. If you can't do that, leave it
# in English for now. I plan to add Unicode support here in the future.

# Displayed at the top of the update screen.
update-prompt-title = Actualización disponible

# Message shown on the update screen. { $new_version } will be replaced with the update's version number.
update-prompt-message = CLEO versión { $new_version } está disponible. ¿Quieres ir a GitHub para descargarlo?

# todo: Add "Yes" and "No" for update menu to localisation files.
# The yes/no options are part of the game, so they're not directly in CLEO's control (yet).

## Release channel settings

update-release-channel-opt-title = Canal de lanzamiento
update-release-channel-opt-desc = Para qué actualizaciones de CLEO recibes notificaciones. Alpha brinda características más nuevas antes, pero puede tener más errores. No se recomienda deshabilitar las actualizaciones.

update-release-channel-opt-disabled = Desactivado
update-release-channel-opt-stable = Estable
update-release-channel-opt-alpha = Alfa

### ==== Menu ====

# Title for the button at the bottom of the screen that closes the CLEO menu.
menu-close = Cierra

# Title for the options tab.
menu-options-tab-title = Opciones

## Menu gesture settings

menu-gesture-opt-title = Gesto de menú
menu-gesture-opt-desc = La acción táctil requerida para mostrar el menú CLEO.

# A single motion where one finger moves quickly down the screen.
menu-gesture-opt-one-finger-swipe = Deslizar un dedo hacia abajo

# A single swipe (as above) but with two fingers at the same time instead of just one.
menu-gesture-opt-two-finger-swipe = Deslizar dos dedos hacia abajo

# A short tap on the screen with two fingers at once.
menu-gesture-opt-two-finger-tap = Toque con dos dedos

# A short tap on the screen with three fingers at once.
menu-gesture-opt-three-finger-tap = Toque con tres dedos

### ==== Script menu ====

## Script warning overview

# Shown at the top of the script menus when at least one error has been found in any script. This
# is not shown at all if there are zero scripts with errors in them.
menu-script-warning-overview =
    { $num_scripts_with_errors ->
        [one] Se encontraron problemas en un script. Este script está resaltado en naranja.
        *[other] Se encontraron problema en un { $num_scripts_with_errors } scripts. Estos scripts están resaltados en naranja.
    }

# The second line of the warning.
menu-script-see-below = Ve abajo para mas detalles.

menu-script-csa-tab-title = CSA
menu-script-csi-tab-title = CSI

## Specific script warnings

# The script does things that CLEO doesn't support yet.
script-unimplemented-in-cleo = Utiliza funciones actualmente no compatibles con CLEO iOS.

# The script does things that are not possible on iOS (for system reasons).
script-impossible-on-ios = Utiliza algún código que no funcionará en iOS.

# The script is identical to another script. { $original_script } will be replaced with the name of
# the script that this one is a duplicate of.
script-duplicate = Duplicado de { $original_script }.

# There was an error when checking the script code for problems.
script-check-failed = No se puede escanear el script. Informe esto como un error en GitHub o Discord.

# No problems were found when scanning the script. This is a safe script!
script-no-problems = No se detectaron problemas.

# Formats for script names in the menu.
script-csa-row-title = { $script_name }
script-csi-row-title = { $script_name }

## Script status messages

# The script is running normally.
script-running = Ejecutando

# The script is not running.
script-not-running = No ejecutando

# The script has been forced to run by the user, even though there are problems with it. This only
# applies to CSA scripts.
script-csa-forced-running = Forzado

## Script settings

script-mode-opt-title = Modo de procesamiento de secuencias de comandos
script-mode-opt-desc = Cambia la forma en que CLEO procesa el código de script. Intente cambiar esto si tiene problemas con los scripts.

# In "don't break" mode, CLEO won't try to reduce script lag. This might be more stable sometimes.
script-mode-opt-dont-break = Lento

# In "break" mode, CLEO will try to reduce script lag caused by long loops in script code.
script-mode-opt-break = Rapido

### ==== FPS ====

## FPS lock option

fps-lock-opt-title = FPS Limite
fps-lock-opt-desc = La velocidad de fotogramas máxima a la que se ejecutará el juego. 30 FPS se ve peor pero ahorra batería.

fps-lock-opt-30 = 30 FPS
fps-lock-opt-60 = 60 FPS

## FPS counter option

fps-counter-opt-title = FPS Contador
fps-counter-opt-desc = Activa o desactiva el contador de FPS en pantalla.

fps-counter-opt-hidden = Desactivado
fps-counter-opt-enabled = Activado

### ==== Cheat system ====

## Menu

cheat-tab-title = Codigo De Trucos

# Two lines of text shown at the top of the cheats menu.
cheat-menu-warning = El uso de códigos de trucos puede provocar fallas y posiblemente una pérdida del progreso del juego.
  Si no quiere arriesgarse a romper su guardado, primero haga una copia de seguridad de su progreso en una ranura diferente.

## Status messages for cheats

cheat-on = Prendido
cheat-off = Apagado

# Cheat will be turned on when the menu is closed.
cheat-queued-on = Puesto en cola

# Cheat will be turned off when the menu is closed.
cheat-queued-off = Fuera de cola

# Formats for cheat codes in the menu.
cheat-code-row-title = { $cheat_code }
cheat-no-code-title = ???

## Cheat saving option

cheat-transience-opt-title = Modo de ahorro de código de trucos
cheat-transience-opt-desc = Controla cómo se gestionan los trucos al recargar/reiniciar el juego.

cheat-transience-opt-transient = Resetear todo
cheat-transience-opt-persistent = Guardar estados

### ==== Cheat descriptions ====

## Weapon sets
cheat-thugs-armoury = Conjunto de armas 1
cheat-professionals-kit = Conjunto de armas 2
cheat-nutters-toys = Conjunto de armas 3
cheat-weapons-4 = Dar consolador, minicañón y gafas térmicas/de visión nocturna

## Debug cheats
cheat-debug-mappings = Depurar (mostrar asignaciones)
cheat-debug-tap-to-target = Depurar (mostrar tocar para apuntar)
cheat-debug-targeting = Depurar (mostrar segmentación)

## Properly cheating
cheat-i-need-some-help = Dar salud, armadura y $250,000
cheat-skip-mission = Saltar hasta completar algunas misiones

## Superpowers
cheat-full-invincibility = Invencibilidad total
cheat-sting-like-a-bee = Súper golpes
cheat-i-am-never-hungry = El jugador nunca tiene hambre
cheat-kangaroo = 10x altura de salto
cheat-noone-can-hurt-me = Salud infinita
cheat-man-from-atlantis = Capacidad pulmonar infinita

## Social player attributes
cheat-worship-me = Maximo respeto
cheat-hello-ladies = Máximo atractivo sexual

## Physical player attributes
cheat-who-ate-all-the-pies = Gordura máxima
cheat-buff-me-up = Músculo máximo
cheat-max-gambling = Máxima habilidad de juego
cheat-lean-and-mean = Mínimo de gordura y músculo
cheat-i-can-go-all-night = Máxima resistencia

## Player skills
cheat-professional-killer = Nivel hitman para todas las armas.
cheat-natural-talent = Habilidades máximas del vehículo

## Wanted level
cheat-turn-up-the-heat = Aumenta el nivel de búsqueda en dos estrellas
cheat-turn-down-the-heat = Completar el nivel querido
cheat-i-do-as-i-please = Bloquear el nivel deseado al valor actual
cheat-bring-it-on = Nivel de búsqueda de seis estrellas

## Weather
cheat-pleasantly-warm = Tiempo soleado
cheat-too-damn-hot = Clima muy soleado
cheat-dull-dull-day = Tiempo nublado
cheat-stay-in-and-watch-tv = Clima lluvioso
cheat-cant-see-where-im-going = Clima nublado
cheat-scottish-summer = Clima tormentoso
cheat-sand-in-my-ears = Tormenta de arena

## Time
cheat-clock-forward = Adelanta el reloj 4 horas
cheat-time-just-flies-by = Tiempo más rápido
cheat-speed-it-up = Modalidad de juego más rápida
cheat-slow-it-down = Juego más lento
cheat-night-prowler = Siempre medianoche
cheat-dont-bring-on-the-night = Siempre 9 p.m.

## Spawning wearables
cheat-lets-go-base-jumping = Aparece paracaídas
cheat-rocketman = Aparece mochila propulsora

## Spawning vehicles

# The vehicle names (in capitals) should not be changed, as they are part of the game. The
# descriptions (in brackets) do need to be translated.
cheat-time-to-kick-ass = Aparecer Rhino (tanque de guerra)
cheat-old-speed-demon = Aparecer Bloodring Banger (coche derby de demolición )
cheat-tinted-rancher = Aparecer Rancher con vidrios polarizados (SUV de dos puertas)
cheat-not-for-public-roads = Aparecer Hotring Racer A (coche de carreras)
cheat-just-try-and-stop-me = Aparecer Hotring Racer B (coche de carreras)
cheat-wheres-the-funeral = Aparecer Romero (coche fúnebre)
cheat-celebrity-status = Aparecer Stretch Limousine (limusina)
cheat-true-grime = Aparecer Trashmaster (camión de la basura)
cheat-18-holes = Aparecer Caddy (carro de golf)
cheat-jump-jet = Aparecer Hydra (avión de ataque VTOL)
cheat-i-want-to-hover = Aparecer Vortex (aerodeslizador)
cheat-oh-dude = Aparecer Hunter (helicóptero de ataque militar)
cheat-four-wheel-fun = Aparecer Quad (cuatriciclo)
cheat-hit-the-road-jack = Aparecer Tanker y remolque (camión cisterna)
cheat-its-all-bull = Aparecer Dozer (excavadora)
cheat-flying-to-stunt = Aparecer Stunt Plane (avión de acrobacias)
cheat-monster-mash = Aparecer Monster Truck (camión monstruo)

## Gang recruitment
cheat-wanna-be-in-my-gang = Recluta a cualquiera en tu banda y dale una pistola apuntándole con una pistola.
cheat-noone-can-stop-us = Recluta a cualquiera en tu pandilla y dales un AK-47 apuntándoles con un AK-47
cheat-rocket-mayhem = Recluta a cualquiera en tu pandilla y dales un lanzacohetes apuntándoles con un lanzacohetes

## Traffic
cheat-all-drivers-are-criminals = Todos los conductores NPC conducen agresivamente y tienen un nivel deseado
cheat-pink-is-the-new-cool = Tráfico rosado
cheat-so-long-as-its-black = Tráfico negro
cheat-everyone-is-poor = Tráfico rural
cheat-everyone-is-rich = Tráfico de autos deportivos

## Pedestrians
cheat-rough-neighbourhood = Dale al jugador un palo de golf y haz que los peatones se amotinen
cheat-stop-picking-on-me = Los peatones atacan al jugador
cheat-surrounded-by-nutters = Dar armas a los peatones
cheat-blue-suede-shoes = Todos los peatones son Elvis Presley
cheat-attack-of-the-village-people = Los peatones atacan al jugador con armas y cohetes
cheat-only-homies-allowed = Miebros de pandillas en todas partes
cheat-better-stay-indoors = Pandilleros en todas partes las pandillas controlan las calles
cheat-state-of-emergency = Disturbios de peatones
cheat-ghost-town = Tráfico en vivo reducido y sin peatones

## Themes
cheat-ninja-town = Tema de la tríada
cheat-love-conquers-all = Tema de chulo
cheat-lifes-a-beach = Tema de fiesta en la playa
cheat-hicksville = Tema rural
cheat-crazy-town = Tema del carnaval

## General vehicle cheats
cheat-all-cars-go-boom = Explotar todos los vehículos
cheat-wheels-only-please = Vehículos invisibles
cheat-sideways-wheels = Los coches tienen ruedas laterales
cheat-speed-freak = Todos los coches tienen nitro
cheat-cool-taxis = Los taxis tienen hidraulica y nitro

## Vehicle cheats for the player
cheat-chitty-chitty-bang-bang = Coches voladores
cheat-cj-phone-home = Lúpulo de conejo muy alto
cheat-touch-my-car-you-die = Destruye otros vehículos en caso de colisión
cheat-bubble-cars = Los automóviles vuelan cuando se impactan
cheat-stick-like-glue = Suspensión y manejo mejorados
cheat-dont-try-and-stop-me = Los semáforos son siempre verdes
cheat-flying-fish = Botes voladores

## Weapon usage
cheat-full-clip = Todos tiene municiones ilimitadas
cheat-i-wanna-driveby = Control total de armas en vehículos

## Player effects
cheat-goodbye-cruel-world = Suicido
cheat-take-a-chill-pill = Efectos de adrenalina
cheat-prostitutes-pay = Las prostitutas te pagan

## Miscellaneous
cheat-xbox-helper = Ajusta las estadísticas para estar cerca de conseguir los logros de Xbox

## Pointless cheats

# Tells the user that a cheat will ALWAYS crash their game.
cheat-crash-warning = ¡CHOQUES!

cheat-slot-melee = { cheat-crash-warning } Tragamonedas cuerpo a cuerpo
cheat-slot-handgun = { cheat-crash-warning } Ranura para pistola
cheat-slot-smg = { cheat-crash-warning } Ranura SMG
cheat-slot-shotgun = { cheat-crash-warning } Ranura para escopeta
cheat-slot-assault-rifle = { cheat-crash-warning } Ranura para rifle de asalto
cheat-slot-long-rifle = { cheat-crash-warning } Ranura larga para rifle
cheat-slot-thrown = { cheat-crash-warning } Ranura para arma arrojadiza
cheat-slot-heavy = { cheat-crash-warning } Ranura de artillería pesada
cheat-slot-equipment = { cheat-crash-warning } Ranura de equipo
cheat-slot-other = { cheat-crash-warning } Otro tragamonedas

cheat-predator = No hace nada
