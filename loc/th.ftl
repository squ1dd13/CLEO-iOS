### ==== Language settings ====

# Used in the settings menu to show the name of the language.
language-name = ภาษาไทย

# Shown when this language has been selected automatically.
language-auto-name = อัตโนมัติ ({ language-name })

# The name of the language setting.
language-opt-title = ตัวเลือกภาษา

# The language setting description.
language-opt-desc = เลือกภาษาที่จะใช้ใน CLEO โหมดอัตโนมัติจะใช้ภาษาเดียวกับระบบของคุณ. เพิ่มภาษาของคุณที่ Discord

### ==== Splash screen ====

# First line at the bottom of the screen.
splash-legal = สงวนลิขสิทธิ์ © 2020-2023 squ1dd13, AYZM, Bruno Melo, Flylarb, ODIN, RAiZOK, tharryz, wewewer1. ภายใต้ MIT License

# Second line.
splash-fun = ทำด้วยใจรักที่ประเทศอังกฤษ แปลที่ประเทศไทย เล่นให้สนุกนะ!

### ==== Updates ====

# PLEASE NOTE: The update menu is currently made from a game menu, and the game fonts do not
# support non-ASCII characters. Please write with ASCII only here. If you can't do that, leave it
# in English for now. I plan to add Unicode support here in the future.

# Displayed at the top of the update screen.
update-prompt-title = มีอัพเดทพร้อมให้ใช้งาน

# Message shown on the update screen. { $new_version } will be replaced with the update's version number.
update-prompt-message = CLEO เวอร์ชั่น { $new_version } พร้อมแล้ว. คุณต้องการไปที่ GitHub เพื่อดาวน์โหลดอัพเดทหรือไม่

# todo: Add "Yes" and "No" for update menu to localisation files.
# The yes/no options are part of the game, so they're not directly in CLEO's control (yet).

## Release channel settings

update-release-channel-opt-title = ตัวเลือกการอัพเดท
update-release-channel-opt-desc = เลือกว่าจะอัพเดท CLEO แบบไหน รุ่นทดลองอาจมีฟีเจอร์ใหม่ๆและการปรับปรุงที่เร็วกว่ารุ่นเสถียร แต่อาจจะมีปัญหาได้ ไม่แนะนำให้ปิดใช้งานการอัพเดท

update-release-channel-opt-disabled = ปิดการอัพเดท
update-release-channel-opt-stable = รุ่นเสถียร
update-release-channel-opt-alpha = รุ่นทดลอง

### ==== Menu ====

# Title for the button at the bottom of the screen that closes the CLEO menu.
menu-close = ปิดเมนู

# Title for the options tab.
menu-options-tab-title = การตั้งค่า

## Menu gesture settings

menu-gesture-opt-title = วิธีในการเปิดเมนู CLEO
menu-gesture-opt-desc = คำสั่งนึ้วที่ใช้ในการเปิดเมนู CLEO

# A single motion where one finger moves quickly down the screen.
menu-gesture-opt-one-finger-swipe = ใช้หนึ่งนิ้วเลื่อนลงเพื่อเปิดเมนู
# A single swipe (as above) but with two fingers at the same time instead of just one.
menu-gesture-opt-two-finger-swipe = ใช้สองนิ้วเลื่อนลงเพื่อเปิดเมนู

# A short tap on the screen with two fingers at once.
menu-gesture-opt-two-finger-tap = ใช้สองนิ้วแตะจอเพื่อเปิดเมนู

# A short tap on the screen with three fingers at once.
menu-gesture-opt-three-finger-tap = ใช้สามนิ้วแตะจอเพื่อเปิดเมนู

### ==== Script menu ====

## Script warning overview

# Shown at the top of the script menus when at least one error has been found in any script. This
# is not shown at all if there are zero scripts with errors in them.
menu-script-warning-overview =
    { $num_scripts_with_errors ->
        [one] เจอปัญหาในสคริปต์ สคริปต์ที่มีปัญหาจะถูกไฮไลต์ด้วยสีส้ม
        *[other] เจอปัญหาใน { $num_scripts_with_errors } สคริปต์. สคริปต์ที่มีปัญหาทั้งหมดจะถูกไฮไลต์ด้วยสีส้ม.
    }

# The second line of the warning.
menu-script-see-below = ดูด้านล่างสำหรับดิเทลเพิ่มเติม

menu-script-csa-tab-title = CSA
menu-script-csi-tab-title = CSI

## Specific script warnings

# The script does things that CLEO doesn't support yet.
script-unimplemented-in-cleo = สคริปต์นี้ใช้คำสั่งที่ CLEO บน iOS ยังไม่รองรับ

# The script does things that are not possible on iOS (for system reasons).
script-impossible-on-ios = สคริปต์นี้ใช้คำสั่งบางตัวที่ไม่มีบน iOS หรือ iPadOS

# The script is identical to another script. { $original_script } will be replaced with the name of
# the script that this one is a duplicate of.
script-duplicate = สคริปต์ซ้ำกับ { $original_script }.

# There was an error when checking the script code for problems.
script-check-failed = ไม่สามารถสแกนสคริปต์ได้ ให้แจ้งปัญหาใน Github หรือ Discord

# No problems were found when scanning the script. This is a safe script!
script-no-problems = ไม่ตรวจพบปัญหา

# Formats for script names in the menu.
script-csa-row-title = { $script_name }
script-csi-row-title = { $script_name }

## Script status messages

# The script is running normally.
script-running = ทำงานอยู่

# The script is not running.
script-not-running = ไม่ได้ทำงาน

# The script has been forced to run by the user, even though there are problems with it. This only
# applies to CSA scripts.
script-csa-forced-running = บังคับใช้สคริปต์

## Script settings

script-mode-opt-title = ความเร็วในการรันสคริปต์
script-mode-opt-desc = เปลี่ยนวิธีที่ CLEO ใช้ในการรันสคริปต์ หากคุณเจอปัญหาในการรันสคริปต์ให้ลองเปลี่ยนโหมดการรันดู

# In "don't break" mode, CLEO won't try to reduce script lag. This might be more stable sometimes.
script-mode-opt-dont-break = ช้า

# In "break" mode, CLEO will try to reduce script lag caused by long loops in script code.
script-mode-opt-break = เร็ว

### ==== FPS ====

## FPS lock option

fps-lock-opt-title = เฟรมเรทสูงสุด
fps-lock-opt-desc = ค่าเฟรมเรทที่สูงที่สุดที่เกมจะแสดง 30 เฟรมเรทจะดูกระตุก แต่จะประหยัดแบตมากขึ้น

fps-lock-opt-30 = 30 เฟรมเรท
fps-lock-opt-60 = 60 เฟรมเรท

## FPS counter option

fps-counter-opt-title = ตัวดูค่าเฟรมเรท
fps-counter-opt-desc = เปิดหรือปิดตัวดูค่าเฟรมเรทบนหน้าจอ.

fps-counter-opt-hidden = ปิดการใช้งาน
fps-counter-opt-enabled = เปิดการใช้งาน

### ==== Cheat system ====

## Menu

cheat-tab-title = สูตรโกง

# Two lines of text shown at the top of the cheats menu.
cheat-menu-warning = การใช้สูตรโกงอาจทำให้เกมเสียหาย ให้เซฟเกมไวัในสล็อตอื่นก่อนเพื่อป้องกันเกมที่เล่นไว้พัง

## Status messages for cheats

cheat-on = เปิด
cheat-off = ปิด

# Cheat will be turned on when the menu is closed.
cheat-queued-on = อยู่ในคิวการทำงาน

# Cheat will be turned off when the menu is closed.
cheat-queued-off = อยู่ในคิวการยกเลิก

# Formats for cheat codes in the menu.
cheat-code-row-title = { $cheat_code }
cheat-no-code-title = ???

## Cheat saving option

cheat-transience-opt-title = การบันทึกสูตรโกง
cheat-transience-opt-desc = ควบคุมการบันทึกสูตรโกงว่าจะรีเซ็ตทุกครั้งที่ออกจากเกมหรือบันทึกไว้

cheat-transience-opt-transient = รีเซ็ทการตั้งค่า
cheat-transience-opt-persistent = บันทึกการตั้งค่า

### ==== Cheat descriptions ====

## Weapon sets
cheat-thugs-armoury = อาวุธชุดที่หนึ่ง
cheat-professionals-kit = อาวุธชุดที่สอง
cheat-nutters-toys = อาวุธชุดที่สาม
cheat-weapons-4 = มีดีโด้,มินิกันและแว่นตรวจจับความร้อนหรือแว่นกลางคืน
## Debug cheats
cheat-debug-mappings = ดีบั๊ก (แสดงการจัดวาง)
cheat-debug-tap-to-target = ดีบั๊ก (แสดงเป้าหมายที่กด)
cheat-debug-targeting = ดีบั๊ก (แสดงเป้าหมาย)

## Properly cheating
cheat-i-need-some-help = ให้เสื้อกันกระสุนและเงิน 250,000$
cheat-skip-mission = ข้ามไปตอนจบของด่าน

## Superpowers
cheat-full-invincibility = ตายไม่ได้
cheat-sting-like-a-bee = ต่อยอย่างรุนแรง
cheat-i-am-never-hungry = ผู้เล่นจะไม่หิว
cheat-kangaroo = กระโดดสูงขึ้น 10 เท่า
cheat-noone-can-hurt-me = เลือดไม่จำกัด
cheat-man-from-atlantis = อากาศไม่จำกัดขณะดำน้ำ

## Social player attributes
cheat-worship-me = ทุกคนให้ความเคารพ
cheat-hello-ladies = ความมีเพศสัมพันธ์สูง

## Physical player attributes
cheat-who-ate-all-the-pies = อ้วนสูงสุด
cheat-buff-me-up = กล้ามเนื้อแข็งแรง
cheat-max-gambling = มีความสามารถในการเล่นพนันสูงที่สุด
cheat-lean-and-mean = ผอมและมีกล้ามเนื้อน้อยที่สุด
cheat-i-can-go-all-night = มีความทรหดอดทนสูง

## Player skills
cheat-professional-killer = ความแม่นปืนระดับ Hitman สำรับอาวุธทุกชนิด
cheat-natural-talent = มีสกิลในการขับขี่สูงที่สุด

## Wanted level
cheat-turn-up-the-heat = เพิ่มดาวตำรวจสองดาว
cheat-turn-down-the-heat = ลบดาวตำรวจออกทั้งหมด
cheat-i-do-as-i-please = ติดดาวเท่าเดิม ไม่ลดหรือเพิ่ม
cheat-bring-it-on = ติดหกดาว

## Weather
cheat-pleasantly-warm = แดดร้อน
cheat-too-damn-hot = แดดร้อนมาก
cheat-dull-dull-day = พายุเข้า
cheat-stay-in-and-watch-tv = ฝนตก
cheat-cant-see-where-im-going = หมอกลง
cheat-scottish-summer = ฝนฟ้าคะนอง
cheat-sand-in-my-ears = พายุทราย

## Time
cheat-clock-forward = เร่งเวลาไปสี่ชั่วโมง
cheat-time-just-flies-by = เวลาเร็วขึ้น
cheat-speed-it-up = เล่นเกมเร็วขึ้น
cheat-slow-it-down = เล่นเกมช้าลง
cheat-night-prowler = กลางคืนตลอด
cheat-dont-bring-on-the-night = เก้าโมงเช้าตลอด
## Spawning wearables
cheat-lets-go-base-jumping = ร่มชูชีพ
cheat-rocketman = เจ็ทแพค

## Spawning vehicles

# The vehicle names (in capitals) should not be changed, as they are part of the game. The
# descriptions (in brackets) do need to be translated.
cheat-time-to-kick-ass = Rhino (รถถัง)
cheat-old-speed-demon = Bloodring Banger (รถแข่งเดอร์บี้)
cheat-tinted-rancher = Rancher พร้อมกระจกฟิล์มทึบ (เอสยูวีสองประตู)
cheat-not-for-public-roads = Hotring Racer A (รถแข่ง)
cheat-just-try-and-stop-me = Hotring Racer B (รถแข่ง)
cheat-wheres-the-funeral = Romero (รถขนศพ)
cheat-celebrity-status = Stretch Limousine (ลิมูซีน)
cheat-true-grime = Trashmaster (รถขยะ)
cheat-18-holes = Caddy (รถกอล์ฟ)
cheat-jump-jet = Hydra (เครื่องบินรบ)
cheat-i-want-to-hover = Vortex (เรือเร็ว)
cheat-oh-dude = Hunter (เฮลิคอปเตอร์ทหาร)
cheat-four-wheel-fun = Quad (รถเอทีวี)
cheat-hit-the-road-jack = Tanker and trailer (รถน้ำมัน)
cheat-its-all-bull = Dozer (บูลเดอร์เซอร์)
cheat-flying-to-stunt = Stunt Plane (เครื่องบินผาดโผน)
cheat-monster-mash = Monster Truck (รถตีนโต/มอนสเตอร์)

## Gang recruitment
cheat-wanna-be-in-my-gang = ทุกคนเป็นแก้งค์เดียวกับคุณและให้อาวุธด้วยการเล็งอาวุธไปที่แก้งค์
cheat-noone-can-stop-us = ทุกคนเป็นแก้งค์เดียวกับคุณและให้ปืน AK-47 ด้วยการเล็ง AK-47 ไปที่คนที่ต้องการ
cheat-rocket-mayhem = ทุกคนเป็นแก้งค์เดียวกับคุณและให้ปืนอาร์พีจีด้วยการเล็งอาร์พีจีไปที่คนที่ต้องการ

## Traffic
cheat-all-drivers-are-criminals = เอ็นพีซีขับรถเกรี้ยวกราดและติดดาว
cheat-pink-is-the-new-cool = รถสีชมพูทุกคัน
cheat-so-long-as-its-black = รถสีดำทุกคัน
cheat-everyone-is-poor = รถเก่าๆทุกคัน
cheat-everyone-is-rich = รถสปอร์ตทุกคัน

## Pedestrians
cheat-rough-neighbourhood = ให้ไม้กอลฟ์กับผู้เล่นและเอ็นพีซีเกรี้ยวกราด
cheat-stop-picking-on-me = เอ็นพีซีทุกคนทำร้ายคุณ
cheat-surrounded-by-nutters = ให้อาวุธกับทุกคน
cheat-blue-suede-shoes = ทุกคนเป็น Elvis Presley
cheat-attack-of-the-village-people = ทุกคนทำร้ายคุณด้วยปืนและจรวด
cheat-only-homies-allowed = ชาวแก๊งทุกคน
cheat-better-stay-indoors = แก๊งคุมถนน
cheat-state-of-emergency = ทุกคนแตกตื่น
cheat-ghost-town = ไม่มีรถและคนเดินถนน (เมืองร้าง)

## Themes
cheat-ninja-town = ธีมสาม
cheat-love-conquers-all = มีแต่โสเภณี
cheat-lifes-a-beach = ธีมชุดชายหาด
cheat-hicksville = ธีมชนบท
cheat-crazy-town = ธีมงานรื่นเริง

## General vehicle cheats
cheat-all-cars-go-boom = รถทุกคันระเบิด
cheat-wheels-only-please = รถล่องหน
cheat-sideways-wheels = รถทุกคันมีล้อเอียง
cheat-speed-freak = รถมีไนโตรทุกคัน
cheat-cool-taxis = แท็กซี่มีไฮโดรลิกและไนโตรทุกคัน

## Vehicle cheats for the player
cheat-chitty-chitty-bang-bang = รถบินได้
cheat-cj-phone-home = บันนี่ฮอปสูง
cheat-touch-my-car-you-die = รถคันอื่นพังเมื่อชน
cheat-bubble-cars = รถลอยออกไปเมื่อชน
cheat-stick-like-glue = ช่วงล่างและเกาะถนนดีขึ้น
cheat-dont-try-and-stop-me = ไฟเขียวตลอด
cheat-flying-fish = เรือบินได้

## Weapon usage
cheat-full-clip = ทุกคนมีกระสุนไม่จำกัด
cheat-i-wanna-driveby = ควบคุมอาวุธทุกชนิดเต็มรูปแบบบนรถ

## Player effects
cheat-goodbye-cruel-world = ฆ่าตัวตาย เซ็งนายก
cheat-take-a-chill-pill = มีอะดรีนาลีนสูง
cheat-prostitutes-pay = โสเภณีจ่ายเงินให้คุณ

## Miscellaneous
cheat-xbox-helper = ทำให้แสตทใกล้เคียงกับ Xbox

## Pointless cheats

# Tells the user that a cheat will ALWAYS crash their game.
cheat-crash-warning = เกมจะเด้งออก!

cheat-slot-melee = { cheat-crash-warning } Melee slot
cheat-slot-handgun = { cheat-crash-warning } Handgun slot
cheat-slot-smg = { cheat-crash-warning } SMG slot
cheat-slot-shotgun = { cheat-crash-warning } Shotgun slot
cheat-slot-assault-rifle = { cheat-crash-warning } Assault rifle slot
cheat-slot-long-rifle = { cheat-crash-warning } Long rifle slot
cheat-slot-thrown = { cheat-crash-warning } Thrown weapon slot
cheat-slot-heavy = { cheat-crash-warning } Heavy artillery slot
cheat-slot-equipment = { cheat-crash-warning } Equipment slot
cheat-slot-other = { cheat-crash-warning } Other slot

cheat-predator = ไม่ทำอะไร
