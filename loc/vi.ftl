# Used in the settings menu to show the name of the language.
language-name = Tiếng Việt

# Shown when this language has been selected automatically.
language-auto-name = Tự động ({ language-name })

# The name of the language setting.
language-opt-title = Ngôn ngữ

# The language setting description.
language-opt-desc = Ngôn ngữ để sử dụng CLEO. Chế độ tự động sẽ sử dụng ngôn ngữ trong hệ thống của bạn. Vui lòng thêm ngôn ngữ của bạn ở trên Discord!

### ==== Splash screen ====

# First line at the bottom of the screen.
splash-legal = Bản quyền © 2020-2023 { $copyright_names }. Được cấp phép theo Giấy phép MIT.

# Second line.
splash-fun = Xin chào các bạn người Việt Nam! CLEO được sản xuất bằng niềm đam mê ở Vương Quốc Anh. Dịch sang tiếng Việt bởi tharryz và ClarusKay.

### ==== Updates ====

# PLEASE NOTE: The update menu is currently made from a game menu, and the game fonts do not
# support non-ASCII characters. Please write with ASCII only here. If you can't do that, leave it
# in English for now. I plan to add Unicode support here in the future.

# Displayed at the top of the update screen.
update-prompt-title = Cập nhật đang có sẵn

# Message shown on the update screen. { $new_version } will be replaced with the update's version number.
update-prompt-message = Phiên bản CLEO { $new_version } đang có sẵn. Bạn có muốn tải xuống từ GitHub không?

# todo: Add "Yes" and "No" for update menu to localisation files.
# The yes/no options are part of the game, so they're not directly in CLEO's control (yet).

## Release channel settings

update-release-channel-opt-title = Kênh phát hành
update-release-channel-opt-desc = Bạn sẽ nhận được thông báo cập nhật CLEO. Phiên bản thử nghiệm sẽ có nhiều tính năng mới hơn, nhưng cũng có thể có nhiều lỗi hơn. Không nên tắt cập nhật.
update-release-channel-opt-disabled = Cấm dùng
update-release-channel-opt-stable = Phiên bản ổn định
update-release-channel-opt-alpha = Phiên bản thử nghiệm

### ==== Menu ====

# Title for the button at the bottom of the screen that closes the CLEO menu.
menu-close = Tắt

# Title for the options tab.
menu-options-tab-title = Tùy chọn

## Menu gesture settings

menu-gesture-opt-title = Thao tác mở Menu
menu-gesture-opt-desc = Các thao tác chạm cần thiết để hiển thị menu CLEO.

# A single motion where one finger moves quickly down the screen.
menu-gesture-opt-one-finger-swipe = Vuốt xuống bằng một ngón tay

# A single swipe (as above) but with two fingers at the same time instead of just one.
menu-gesture-opt-two-finger-swipe = Vuốt xuống bằng hai ngón tay

# A short tap on the screen with two fingers at once.
menu-gesture-opt-two-finger-tap = Chạm bằng hai ngón tay

# A short tap on the screen with three fingers at once.
menu-gesture-opt-three-finger-tap = Chạm bằng ba ngón tay

### ==== Script menu ====

## Script warning overview

# Shown at the top of the script menus when at least one error has been found in any script. This
# is not shown at all if there are zero scripts with errors in them.
menu-script-warning-overview = Đã tìm thấy lỗi trong tập lệnh{ $num_scripts_with_errors }. Tập lệnh lỗi sẽ  được đánh dấu bằng màu cam.

# The second line of the warning.
menu-script-see-below = Xem bên dưới để biết thêm chi tiết.

menu-script-csa-tab-title = CSA
menu-script-csi-tab-title = CSI

## Specific script warnings

# The script does things that CLEO doesn't support yet.
script-unimplemented-in-cleo = Tính năng này hiện không khả dụng trong phiên bản CLEO dành cho iOS.

# The script does things that are not possible on iOS (for system reasons).
script-impossible-on-ios = Mã này không có sẵn trong phiên bản iOS.

# The script is identical to another script. { $originalScript } will be replaced with the name of
# the script that this one is a duplicate of.
script-duplicate = Sao chép { $original_script }.

# There was an error when checking the script code for problems.
script-check-failed = Không thể nhận dạng tập lệnh. Vui lòng báo cáo lỗi này trên GitHub hay Discord.

# No problems were found when scanning the script. This is a safe script!
script-no-problems = Không phát hiện lỗi.

# Formats for script names in the menu.
script-csa-row-title = { $script_name }
script-csi-row-title = { $script_name }

## Script status messages

# The script is running normally.
script-running = Đang chạy

# The script is not running.
script-not-running = Không chạy

# The script has been forced to run by the user, even though there are problems with it. This only
# applies to CSA scripts.
script-csa-forced-running = Buộc phải chạy

## Script settings

script-mode-opt-title = Phương thức xử lý tập lệnh
script-mode-opt-desc = Thay đổi cách CLEO chạy mã tập lệnh. Nếu tập lệnh của bạn chạy không chính xác, hãy thử sửa đổi cài đặt này.

# In "don't break" mode, CLEO won't try to reduce script lag. This might be more stable sometimes.
script-mode-opt-dont-break = Chậm

# In "break" mode, CLEO will try to reduce script lag caused by long loops in script code.
script-mode-opt-break = Nhanh

### ==== FPS ====

## FPS lock option

fps-lock-opt-title = Giới hạn FPS
fps-lock-opt-desc = Tốc độ khung hình tối đa mà trò chơi sẽ chạy. 30 FPS có thể không đủ mượt mà nhưng tiết kiệm pin.

fps-lock-opt-30 = 30 FPS
fps-lock-opt-60 = 60 FPS

## FPS counter option

fps-counter-opt-title = Bộ đếm FPS
fps-counter-opt-desc = Bật hay tắt bộ đếm FPS trên màn hình.

fps-counter-opt-hidden = Cấm dùng
fps-counter-opt-enabled = Được dùng

### ==== Cheat system ====

## Menu

cheat-tab-title = Gian lận

# Two lines of text shown at the top of the cheats menu.
cheat-menu-warning = Sử dụng gian lận có thể dẫn đến sự cố và có thể làm mất tiến trình trò chơi.
cheat-menu-advice = Nếu bạn không muốn mạo hiểm làm mất bộ lưu trữ trò chơi, trước tiên hãy sao lưu dữ liệu trò chơi của bạn sang một vị trí khác

## Status messages for cheats

cheat-on = Bật
cheat-off = Tắt

# Cheat will be turned on when the menu is closed.
cheat-queued-on = Sắp được dùng

# Cheat will be turned off when the menu is closed.
cheat-queued-off = Sắp tắt

# Formats for cheat codes in the menu.
cheat-code-row-title = { $cheat_code }
cheat-no-code-title = ???

## Cheat saving option

cheat-transience-opt-title = Chế độ lưu gian lận
cheat-transience-opt-desc = Kiểm soát các gian lận khi trò chơi được tải lại hay khởi động lại.

cheat-transience-opt-transient = Đặt lại tất cả
cheat-transience-opt-persistent = Lưu trạng thái

### ==== Cheat descriptions ====

## Weapon sets
cheat-thugs-armoury = Ba lô vũ khí 1
cheat-professionals-kit = Ba lô vũ khí 2
cheat-nutters-toys = Ba lô vũ khí 3
cheat-weapons-4 = Được một dương vật giả, một khẩu súng Gatling và ống kính nhìn trong ban đêm.

## Debug cheats
cheat-debug-mappings = Gỡ lỗi (hiển thị bản đồ)
cheat-debug-tap-to-target = Gỡ lỗi (nhấp để xác định mục tiêu)
cheat-debug-targeting = Gỡ lỗi (hiển thị mục tiêu)

## Properly cheating
cheat-i-need-some-help = Phục hồi lượng máu, được áo giáp và $250,000
cheat-skip-mission = Hoàn thành để bỏ qua một số nhiệm vụ

## Superpowers
cheat-full-invincibility = Vô địch
cheat-sting-like-a-bee = Nắm đấm sắt
cheat-i-am-never-hungry = Người chơi chẳng bao giờ đói
cheat-kangaroo = Nhảy 10 gấp 10 lần độ cao
cheat-noone-can-hurt-me = Bất tử
cheat-man-from-atlantis = Dung tích phổi không giới hạn

## Social player attributes
cheat-worship-me = Uy danh cao nhất
cheat-hello-ladies = Sức hấp dẫn cao nhất

## Physical player attributes
cheat-who-ate-all-the-pies = Béo phì
cheat-buff-me-up = Cơ bắp cuồn cuộn
cheat-max-gambling = Kỹ năng đánh bạc cực cao
cheat-lean-and-mean = Gầy gò
cheat-i-can-go-all-night = Đầy thanh chạy

## Player skills
cheat-professional-killer = Tất cả vũ khí đều dùng ở cấp độ sát thủ chuyên nghiệp
cheat-natural-talent = Kỹ năng lái xe cao nhất

## Wanted level
cheat-turn-up-the-heat = Mức truy nã tăng hai sao
cheat-turn-down-the-heat = Xóa mức truy nã
cheat-i-do-as-i-please = Khóa mức truy nã hiện tại
cheat-bring-it-on = Truy nã sáu sao

## Weather
cheat-pleasantly-warm = Trời nắng
cheat-too-damn-hot = Ngày nóng như thiêu đốt
cheat-dull-dull-day = Trời âm u
cheat-stay-in-and-watch-tv = Ngày mưa
cheat-cant-see-where-im-going = Ngày sương mù
cheat-scottish-summer = Bão tố
cheat-sand-in-my-ears = Bão cát bụi

## Time
cheat-clock-forward = Thời gian tăng thêm 4 giờ
cheat-time-just-flies-by = Tăng tốc thời gian
cheat-speed-it-up = Tăng tốc trò chơi
cheat-slow-it-down = Giảm tốc độ trò chơi
cheat-night-prowler = Luôn ở nửa đêm
cheat-dont-bring-on-the-night = Luôn 9 giờ tối

## Spawning wearables
cheat-lets-go-base-jumping = Được một cái dù
cheat-rocketman = Được động cơ phản lực thu nhỏ

## Spawning vehicles

# The vehicle names (in capitals) should not be changed, as they are part of the game. The
# descriptions (in brackets) do need to be translated.
cheat-time-to-kick-ass = Được "Tê Giác" (xe tăng quân sự)
cheat-old-speed-demon = Được "Bloodring Banger" (xe derby phá hủy)
cheat-tinted-rancher = Được "Rancher có cửa sổ màu" (SUV hai cửa)
cheat-not-for-public-roads = Được "Hotring Racer A" (xe đua)
cheat-just-try-and-stop-me = Được "Hotring Racer B" (xe đua)
cheat-wheres-the-funeral = Được "Romero" (xe tang)
cheat-celebrity-status = Được "Stretch Limousine" (xe limousine)
cheat-true-grime = Được "Trashmaster" (xe chở rác)
cheat-18-holes = Được "Caddy" (xe golf)
cheat-jump-jet = Được "Hydra" (máy bay phản lực tấn công VTOL)
cheat-i-want-to-hover = Được "Vortex" (tàu chạy trên đệm khí)
cheat-oh-dude = Được "Hunter" (máy bay trực thăng tấn công quân sự)
cheat-four-wheel-fun = Được "Quad" (quadbike/ATV/four-wheeler)
cheat-hit-the-road-jack = Được "Tanker và xe kéo" (xe bồn)
cheat-its-all-bull = Được "Dozer" (chiếc xe ủi)
cheat-flying-to-stunt = Được "Stunt Plane" (máy bay đóng thế)
cheat-monster-mash = Được "Monster Truck" (xe quái thú)

## Gang recruitment
cheat-wanna-be-in-my-gang = Chiêu mộ bất kỳ ai vào băng đảng của bạn và đưa cho họ một khẩu súng lục bằng cách nhắm khẩu súng lục vào họ
cheat-noone-can-stop-us = Chiêu mộ bất kỳ ai vào băng đảng của bạn và đưa cho họ một khẩu AK-47 bằng cách nhắm khẩu AK-47 vào họ
cheat-rocket-mayhem = Chiêu mộ bất kỳ ai vào băng đảng của bạn và cho họ một khẩu ba-dô-ca bằng cách nhắm khẩu ba-dô-ca vào họ

## Traffic
cheat-all-drivers-are-criminals = Tất cả các NPC đang lái xe nổi cơn thịnh nộ và đều bị truy nã một sao
cheat-pink-is-the-new-cool = Giao thông toàn màu hồng
cheat-so-long-as-its-black = Giao thông toàn xe đen
cheat-everyone-is-poor = Giao thông toàn xe nông thôn
cheat-everyone-is-rich = Giao thông toàn xe thể thao

## Pedestrians
cheat-rough-neighbourhood = Người chơi được gậy golf và người đi bộ thì bạo loạn
cheat-stop-picking-on-me = Người đi bộ tấn công người chơi
cheat-surrounded-by-nutters = Người đi bộ được vũ khí
cheat-blue-suede-shoes = Tất cả những người đi bộ đều là Elvis Presley
cheat-attack-of-the-village-people = Người đi bộ tấn công người chơi bằng súng và ba-dô-ca
cheat-only-homies-allowed = Thành viên băng nhóm ở mọi nơi
cheat-better-stay-indoors = Các băng nhóm kiểm soát đường phố
cheat-state-of-emergency = Người đi bộ bạo loạn
cheat-ghost-town = Giảm phương tiện lưu thông và không có người đi bộ

## Themes
cheat-ninja-town = Chủ đề hội Tam Hợp
cheat-love-conquers-all = Chủ đề ma cô
cheat-lifes-a-beach = Chủ đề tiệc bãi biển
cheat-hicksville = Chủ đề nông thôn
cheat-crazy-town = Chủ đề lễ hội

## General vehicle cheats
cheat-all-cars-go-boom = Nổ tung tất cả các xe cộ
cheat-wheels-only-please = Xe cộ tàng hình
cheat-sideways-wheels = Xe cộ có bánh xe phụ
cheat-speed-freak = Tất cả xe cộ có nitro
cheat-cool-taxis = Xe ta-xi có thủy lực và nitro

## Vehicle cheats for the player
cheat-chitty-chitty-bang-bang = Xe cộ bay
cheat-cj-phone-home = Nhảy kiểu thỏ rất cao
cheat-touch-my-car-you-die = Phá hủy các xe cộ khác khi va chạm
cheat-bubble-cars = Xe cộ bay lên khi bị va chạm
cheat-stick-like-glue = Cải thiện mức độ điều khiển xe cộ
cheat-dont-try-and-stop-me = Đèn giao thông luôn xanh
cheat-flying-fish = Thuyền bay

## Weapon usage
cheat-full-clip = Mọi người đều được đạn không giới hạn
cheat-i-wanna-driveby = Tất cả vũ khí có thể được sử dụng trong xe cộ

## Player effects
cheat-goodbye-cruel-world = Tự sát
cheat-take-a-chill-pill = Hiệu ứng a-đrê-na-lin
cheat-prostitutes-pay = Gái mại dâm trả tiền cho bạn

## Miscellaneous
cheat-xbox-helper = Điều chỉnh số liệu thống kê để gần đạt được thành tích Xbox

## Pointless cheats

# Tells the user that a cheat will ALWAYS crash their game.
cheat-crash-warning = Sự cố!

cheat-slot-melee = { cheat-crash-warning } Khe cận chiến
cheat-slot-handgun = { cheat-crash-warning } Khe súng lục
cheat-slot-smg = { cheat-crash-warning } Khe súng tiểu liên
cheat-slot-shotgun = { cheat-crash-warning } Khe súng ngắn
cheat-slot-assault-rifle = { cheat-crash-warning } Khe súng trường tấn công
cheat-slot-long-rifle = { cheat-crash-warning } Khe súng trường dài
cheat-slot-thrown = { cheat-crash-warning } Khe ném vũ khí
cheat-slot-heavy = { cheat-crash-warning } Khe pháo hạng nặng
cheat-slot-equipment = { cheat-crash-warning } Khe trang bị
cheat-slot-other = { cheat-crash-warning } Khe khác

cheat-predator = Vô hiệu hoá
