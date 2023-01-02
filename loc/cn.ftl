# Used in the settings menu to show the name of the language.
language-name = 中文（简体）

### ==== Splash screen ====

# First line at the bottom of the screen.
splash-legal = 版权所有 © 2020-2023 squ1dd13, AYZM, ODIN, RAiZOK, tharryz, wewewer1. 根据麻省理工学院许可协议（MIT许可协议）获得许可。

# Second line.
splash-fun = 中国的朋友们你们好！祝你们玩得愉快！于英国倾情制作。由tharryz将其翻译为中文（简体）。

### ==== Updates ====

# PLEASE NOTE: The update menu is currently made from a game menu, and the game fonts do not
# support non-ASCII characters. Please write with ASCII only here. If you can't do that, leave it
# in English for now. I plan to add Unicode support here in the future.

# Displayed at the top of the update screen.
update-prompt-title = 有可用更新

# Message shown on the update screen. { $new_version } will be replaced with the update's version number.
update-prompt-message = CLEO 版本 { $new_version } 可用，您是否想从GitHub上下载它?

# todo: Add "Yes" and "No" for update menu to localisation files.
# The yes/no options are part of the game, so they're not directly in CLEO's control (yet).

## Release channel settings

update-release-channel-opt-title = 发布渠道
update-release-channel-opt-desc = 您会收到的CLEO更新通知。测试版本会更快地推送一些新的功能，但可能也会存在更多的漏洞。不建议关闭更新。

update-release-channel-opt-disabled = 禁用
update-release-channel-opt-stable = 稳定版本
update-release-channel-opt-alpha = 测试版本

### ==== Menu ====

# Title for the button at the bottom of the screen that closes the CLEO menu.
menu-close = 关闭

# Title for the options tab.
options-tab-title = 选项

### ==== Script menu ====

## Script warning overview

# Shown at the top of the script menus when at least one error has been found in any script. This
# is not shown at all if there are zero scripts with errors in them.
menu-script-warning-overview = 在{ $numberOfScriptsWithErrors }个脚本中发现错误。错误脚本以橙色标注。

# The second line of the warning.
menu-script-see-below = 详情请见下文。

## Specific script warnings

# The script does things that CLEO doesn't support yet.
script-unimplemented-in-cleo = 此功能当前在CLEO的IOS版本中不可用。

# The script does things that are not possible on iOS (for system reasons).
script-impossible-on-ios = 此代码在iOS版本中不可用。

# The script is identical to another script. { $originalScript } will be replaced with the name of
# the script that this one is a duplicate of.
script-duplicate = 复制 { $originalScript }。

# There was an error when checking the script code for problems.
script-check-failed = 无法识别该脚本。请将此漏洞反馈到GitHub或Discord上。

# No problems were found when scanning the script. This is a safe script!
script-no-problems = 未检测到问题。

## Script status messages

# The script is running normally.
script-running = 运行中

# The script is not running.
script-not-running = 未运行

# The script has been forced to run by the user, even though there are problems with it. This only
# applies to CSA scripts.
script-csa-forced-running = 强制运行

## Script settings

script-mode-opt-title = 脚本处理方式
script-mode-opt-desc = 改变CLEO运行脚本代码的方式。如果您的脚本运行出错，请尝试修改此设置。

# In "don't break" mode, CLEO won't try to reduce script lag. This might be more stable sometimes.
script-mode-opt-dont-break = 慢速

# In "break" mode, CLEO will try to reduce script lag caused by long loops in script code.
script-mode-opt-break = 快速

### ==== FPS ====

## FPS lock option

fps-lock-opt-title = FPS限制
fps-lock-opt-desc = 游戏运行的最大帧率。30 FPS可能不够流畅，但可节省电量。

fps-lock-opt-30 = 30 FPS
fps-lock-opt-60 = 60 FPS

## FPS counter option

fps-counter-opt-title = FPS 计数器
fps-counter-opt-desc = 启用或禁用屏幕上的 FPS 计数器。

fps-counter-opt-hidden = 禁用
fps-counter-opt-enabled = 启用

### ==== Cheat system ====

## Menu

cheat-tab-title = 作弊器

# Two lines of text shown at the top of the cheats menu.
cheat-menu-warning = 使用作弊器可能会导致游戏崩溃且有可能丢失游戏进度。
cheat-menu-advice = 如果您不愿冒游戏存档损失的风险，请先将您的游戏进度备份到其它卡槽中。

## Status messages for cheats

cheat-on = 开
cheat-off = 关

# Cheat will be turned on when the menu is closed.
cheat-queued-on = 即将启用

# Cheat will be turned off when the menu is closed.
cheat-queued-off = 即将禁用

## Cheat saving option

cheat-transience-opt-title = 作弊保存方式
cheat-transience-opt-desc = 控制游戏重新加载或重启时，作弊的启用方式。

cheat-transience-opt-transient = 全部重置
cheat-transience-opt-persistent = 保存状态

### ==== Cheat descriptions ====

## Weapon sets
cheat-thugs-armoury = 武器背包 1
cheat-professionals-kit = 武器背包 2
cheat-nutters-toys = 武器背包 3
cheat-weapons-4 = 获得假阳具，加特林机枪和夜视仪。

## Debug cheats
cheat-debug-mappings = 排错 (显示地图)
cheat-debug-tap-to-target = 排错 (显示点击定位)
cheat-debug-targeting = 排错 (显示目标)

## Properly cheating
cheat-i-need-some-help = 满血, 获得护甲并得到 $250,000。
cheat-skip-mission = 直接完成某些任务。

## Superpowers
cheat-full-invincibility = 无敌
cheat-sting-like-a-bee = 大力拳
cheat-i-am-never-hungry = 人物永不饥饿
cheat-kangaroo = 10倍跳跃
cheat-noone-can-hurt-me = 锁血
cheat-man-from-atlantis = 无限肺活量

## Social player attributes
cheat-worship-me = 威望值满
cheat-hello-ladies = 吸引力值满

## Physical player attributes
cheat-who-ate-all-the-pies = 体脂值满
cheat-buff-me-up = 肌肉值满
cheat-max-gambling = 赌技值满
cheat-lean-and-mean = 最少体脂和肌肉
cheat-i-can-go-all-night = 耐力值满

## Player skills
cheat-professional-killer = 所有武器专业杀手级别
cheat-natural-talent = 车技值满

## Wanted level
cheat-turn-up-the-heat = 通缉等级提高两星
cheat-turn-down-the-heat = 清除通缉等级
cheat-i-do-as-i-please = 锁定当前通缉等级
cheat-bring-it-on = 六星通缉

## Weather
cheat-pleasantly-warm = 晴天
cheat-too-damn-hot = 艳阳天
cheat-dull-dull-day = 阴天
cheat-stay-in-and-watch-tv = 雨天
cheat-cant-see-where-im-going = 雾天
cheat-scottish-summer = 暴风雨
cheat-sand-in-my-ears = 沙尘暴

## Time
cheat-clock-forward = 时间增加4小时
cheat-time-just-flies-by = 时间加快
cheat-speed-it-up = 加快游戏速度
cheat-slow-it-down = 减缓游戏速度
cheat-night-prowler = 总是午夜
cheat-dont-bring-on-the-night = 总是晚上9点

## Spawning wearables
cheat-lets-go-base-jumping = 获得降落伞
cheat-rocketman = 获得喷气背包

## Spawning vehicles

# The vehicle names (in capitals) should not be changed, as they are part of the game. The
# descriptions (in brackets) do need to be translated.
cheat-time-to-kick-ass = 生成“犀牛”Rhino (军用坦克)
cheat-old-speed-demon = 生成“血环炸弹”Bloodring Banger (撞车比赛用车)
cheat-tinted-rancher = 生成“蓝彻”Rancher with tinted windows (两门SUV)
cheat-not-for-public-roads = 生成“场地赛车A”Hotring Racer A (赛车)
cheat-just-try-and-stop-me = 生成“场地赛车B” Hotring Racer B (赛车)
cheat-wheres-the-funeral = 生成“钢骨灵车”Romero (灵车)
cheat-celebrity-status = 生成“加长豪华轿车”Stretch Limousine (豪华轿车)
cheat-true-grime = 生成“垃圾大王”Trashmaster (垃圾车)
cheat-18-holes = 生成“球童”Caddy (高尔夫车)
cheat-jump-jet = 生成“九头蛇”Hydra (垂直起降攻击机)
cheat-i-want-to-hover = 生成“涡流”Vortex (气垫船)
cheat-oh-dude = 生成“猎人”Hunter (军事攻击直升机)
cheat-four-wheel-fun = 生成“四轮摩托车”Quad (四轮摩托车/全地形车/越野型沙滩车)
cheat-hit-the-road-jack = 生成“油罐车”Tanker and trailer (油罐拖车)
cheat-its-all-bull = 生成“推土机”Dozer (推土机)
cheat-flying-to-stunt = 生成“特技飞机”Stunt Plane (特技飞机)
cheat-monster-mash = 生成“怪兽卡车”Monster Truck (怪兽卡车)

## Gang recruitment
cheat-wanna-be-in-my-gang = 用手枪瞄准任何人，来将其招募进你的帮派且每人获得一把手枪
cheat-noone-can-stop-us = 用AK-47瞄准任何人，来将其招募进你的帮派且每人获得一把AK-47
cheat-rocket-mayhem = 用火箭筒瞄准任何人，来将其招募进你的帮派且每人获得一把火箭筒

## Traffic
cheat-all-drivers-are-criminals = 所有NPC司机都横冲直撞且受到一星通缉
cheat-pink-is-the-new-cool = 粉色交通
cheat-so-long-as-its-black = 黑色交通
cheat-everyone-is-poor = 乡村交通
cheat-everyone-is-rich = 跑车交通

## Pedestrians
cheat-rough-neighbourhood = 玩家获得高尔夫球杆且行人暴动
cheat-stop-picking-on-me = 行人攻击玩家
cheat-surrounded-by-nutters = 行人获得武器
cheat-blue-suede-shoes = 所有行人猫王穿着
cheat-attack-of-the-village-people = 行人用枪支和火箭弹攻击玩家
cheat-only-homies-allowed = 帮派成员无处不在
cheat-better-stay-indoors = 帮派们管理各个街区
cheat-state-of-emergency = 行人骚乱
cheat-ghost-town = 实时车辆减少且无行人

## Themes
cheat-ninja-town = 三合会主题
cheat-love-conquers-all = 皮条客主题
cheat-lifes-a-beach = 沙滩派对主题
cheat-hicksville = 乡村主题
cheat-crazy-town = 嘉年华主题

## General vehicle cheats
cheat-all-cars-go-boom = 摧毁所有车辆
cheat-wheels-only-please = 车辆隐形
cheat-sideways-wheels = 车辆获得侧轮
cheat-speed-freak = 所有车辆获得氮气加速
cheat-cool-taxis = 出租车获得液压悬挂和氮气加速

## Vehicle cheats for the player
cheat-chitty-chitty-bang-bang = 汽车飞行
cheat-cj-phone-home = 自行车海豚跳很高
cheat-touch-my-car-you-die = 碰撞时摧毁其它车辆
cheat-bubble-cars = 碰撞时车辆飞起来
cheat-stick-like-glue = 改进车辆悬挂和操控
cheat-dont-try-and-stop-me = 一路绿灯
cheat-flying-fish = 船只飞行

## Weapon usage
cheat-full-clip = 每人获得无限弹药
cheat-i-wanna-driveby = 所有武器可在车辆中使用

## Player effects
cheat-goodbye-cruel-world = 自杀
cheat-take-a-chill-pill = 肾上腺素
cheat-prostitutes-pay = 妓女向你付钱

## Miscellaneous
cheat-xbox-helper = 调整统计数据至接近值，以获得Xbox成就。

## Pointless cheats

# Tells the user that a cheat will ALWAYS crash their game.
-cheat-crash-warning = 崩溃!

cheat-slot-melee = { -cheat-crash-warning } 近战武器槽
cheat-slot-handgun = { -cheat-crash-warning } 手枪槽
cheat-slot-smg = { -cheat-crash-warning } 冲锋枪槽
cheat-slot-shotgun = { -cheat-crash-warning } 霰弹枪槽
cheat-slot-assault-rifle = { -cheat-crash-warning } 突击步枪槽
cheat-slot-long-rifle = { -cheat-crash-warning } 长步枪槽
cheat-slot-thrown = { -cheat-crash-warning } 投掷武器槽
cheat-slot-heavy = { -cheat-crash-warning } 重炮槽
cheat-slot-equipment = { -cheat-crash-warning } 装备槽
cheat-slot-other = { -cheat-crash-warning } 其它槽

cheat-predator = 无效
