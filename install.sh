# Compile the Rust code and produce an ar archive.
# todo: This should be debug, not release.
cargo lipo --release --allow-run-on-non-macos -vv

# Move to the output directory.
cd target/aarch64-apple-ios/release
unlink ./libgreetings.dylib

# Convert the ar archive to a dylib.
/home/squ1dd13/Documents/Projects/iOS-Toolchain/alternative/ios-arm64e-clang-toolchain/bin/clang -fpic -shared -Wl,-all_load ./libgreetings.a -o ./libgreetings.dylib -B/home/squ1dd13/Documents/Projects/iOS-Toolchain/alternative/ios-arm64e-clang-toolchain/bin -isysroot /home/squ1dd13/Documents/Projects/iOS-Toolchain/SDK/iPhoneOS.sdk -target arm64-apple-darwin -I/home/squ1dd13/Documents/Projects/iOS-Toolchain/SDK/iPhoneOS.sdk/usr/include -I/home/squ1dd13/Documents/Projects/iOS-Toolchain/SDK/iPhoneOS.sdk/usr/include/c++/4.2.1 -arch arm64 -framework CoreFoundation -framework Security

# Fakesign the dylib.
ldid -S ./libgreetings.dylib

# Send the dylib to the device.
# todo: Produce a deb and install that to the device using scp and ssh.
scp ./libgreetings.dylib root@192.168.1.226:/var/mobile/Documents/