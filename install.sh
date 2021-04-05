# Compile the Rust code and produce an ar archive.
# todo: This should be debug, not release.
printf "=> Compiling Rust code...\n"
cargo lipo --release --allow-run-on-non-macos || exit 1
printf "=> Finished compiling Rust code.\n"

# Move to the output directory.
cd target/aarch64-apple-ios/release
unlink ./libcleo.dylib

# Convert the ar archive to a dylib.
printf "=> Converting to .dylib... "
{/home/squ1dd13/Documents/Projects/iOS-Toolchain/alternative/ios-arm64e-clang-toolchain/bin/clang -fpic -shared -Wl,-all_load ./libcleo.a -o ./libcleo.dylib -B/home/squ1dd13/Documents/Projects/iOS-Toolchain/alternative/ios-arm64e-clang-toolchain/bin -isysroot /home/squ1dd13/Documents/Projects/iOS-Toolchain/SDK/iPhoneOS.sdk -target arm64-apple-darwin -I/home/squ1dd13/Documents/Projects/iOS-Toolchain/SDK/iPhoneOS.sdk/usr/include -I/home/squ1dd13/Documents/Projects/iOS-Toolchain/SDK/iPhoneOS.sdk/usr/include/c++/4.2.1 -arch arm64 -framework CoreFoundation -framework Security 2>&1 || exit 1 } | grep -v "hides a non-existent symbol"
printf "done.\n"

# Fakesign the dylib.
printf "=> Fakesigning .dylib... "
ldid -S ./libcleo.dylib || exit 1
printf "done.\n"

# Send the dylib to the device.
# todo: Produce a deb and install that to the device using scp and ssh.
printf "=> Installing... "
(scp ./libcleo.dylib root@$1:/Library/MobileSubstrate/DynamicLibraries/CLEO.dylib >/dev/null) || exit 1
printf "done.\n"
