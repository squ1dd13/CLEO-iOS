#!/bin/bash

# See README.md for information on how to use this script.

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
$CLEO_CLANG -fpic -shared -Wl,-all_load libcleo.a -o cleo.dylib -isysroot $CLEO_IOS_SDK -target arm64-apple-darwin -framework CoreFoundation -framework Security || exit 2

echo "=> Signing..."

$CLEO_LDID -S cleo.dylib || exit 3

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
