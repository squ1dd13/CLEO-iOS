#!/bin/bash

# CLEO iOS build script
#
# You must set CLEO_IOS_TOOLCHAIN_PATH and CLEO_IOS_SDK_PATH before using this script. If you're on
# macOS, you probably have everything you need already, and you can set those to existing
# directories on your system. If you're on Linux, you can use
# https://github.com/sbingner/llvm-project as your iOS toolchain and it should work out-of-the-box.
#
# Here are my values:
#  CLEO_IOS_TOOLCHAIN_PATH="/home/squ1dd13/projects/ios/toolchain"
#        CLEO_IOS_SDK_PATH="/home/squ1dd13/projects/ios/iPhoneOS.sdk"
#
# Additionally, if you wish to install the tweak to a device after building it, you must supply an
# IP address or hostname with CLEO_INSTALL_HOST.

# We use Clang for converting the archive that `cargo build` produces into a dynamic library.
# It is necessary for whatever version of Clang we use to support the "arm64-apple-darwin"
# architecture, which is why we use Clang from the iOS toolchain.
CLANG_PATH="$CLEO_IOS_TOOLCHAIN_PATH/bin/clang"

# We use ldid for signing our dylib.
LDID_PATH="$CLEO_IOS_TOOLCHAIN_PATH/bin/ldid"

CLEO_DIR="$(pwd)"

if [[ $* == *--release* ]]; then
    # Avoid incorrectly-versioned builds by making the user acknowledge that they have checked the
    # version numbers.
    printf "Please ensure that all version numbers are up-to-date.\nPress enter to continue."
    read _

    echo "=> Building in release mode..."

    cargo build --target aarch64-apple-ios --release || exit 1

    OUTPUT_DIR="$(pwd)/target/aarch64-apple-ios/release"
else
    echo "=> Building in debug mode..."

    # The default build mode is debug, but we also include a `debug` feature that we use in the
    # CLEO code for conditional compilation.
    cargo build --target aarch64-apple-ios --features debug || exit 1

    OUTPUT_DIR="$(pwd)/target/aarch64-apple-ios/debug"
fi

cd "$OUTPUT_DIR" || exit

# Remove the old dylib if there is one. This won't complain if it's not there.
rm -f cleo.dylib

echo "=> Creating dynamic library..."

# Use Clang to convert the archive to a dynamic library.
$CLANG_PATH -fpic -shared -Wl,-all_load libcleo.a -o cleo.dylib -isysroot $CLEO_IOS_SDK_PATH -target arm64-apple-darwin -framework CoreFoundation -framework Security || exit 2

echo "=> Signing..."

$LDID_PATH -S cleo.dylib || exit 3

if [[ $* == *--package* ]]; then
    echo "=> Creating package directory..."

    rm -rf ./deb-archive ./cleo.deb

    # Create the layout.
    mkdir -p ./deb-archive/Library/MobileSubstrate/DynamicLibraries || exit 4
    mkdir ./deb-archive/DEBIAN || exit 4

    cd ./deb-archive || exit

    echo "=> Copying files into package..."

    # Copy in the bits we need from the main directory.
    cp "$CLEO_DIR/deb/control" "./DEBIAN/control" || exit 4
    cp "$CLEO_DIR/deb/cleo.plist" "./Library/MobileSubstrate/DynamicLibraries/CLEO.plist" || exit 4
    cp "$OUTPUT_DIR/cleo.dylib" "./Library/MobileSubstrate/DynamicLibraries/CLEO.dylib" || exit 4

    cd "$OUTPUT_DIR" || exit

    echo "=> Building archive..."

    # Build the .deb from the directory.
    dpkg-deb -Z xz -b ./deb-archive ./cleo.deb

    # Remove our archive directory.
    rm -rf ./deb-archive

    if [[ $* == *--install* ]]; then
        echo "=> Copying .deb to '$CLEO_INSTALL_HOST'..."
        (scp -q ./cleo.deb root@$CLEO_INSTALL_HOST:/User/Downloads/cleo.deb) || exit 5

        echo "=> Installing .deb on device..."
        ssh root@$CLEO_INSTALL_HOST 'exec $SHELL -l -c "dpkg -i /User/Downloads/cleo.deb && rm -f /User/Downloads/cleo.deb"'
    fi
elif [[ $* == *--install* ]]; then
    echo "=> Updating .dylib on '$CLEO_INSTALL_HOST'..."
    (scp -q cleo.dylib root@$CLEO_INSTALL_HOST:/Library/MobileSubstrate/DynamicLibraries/CLEO.dylib) || exit 6
fi
