#!/bin/bash
# Takes 1 argument: the name of the output .dylib (but without the .dylib extension).

# Clear or create './out'.
rm -r ./out
mkdir -p ./out/package/Library/MobileSubstrate/DynamicLibraries
mkdir ./out/package/DEBIAN

cp "./meta/$1.plist" "./out/package/Library/MobileSubstrate/DynamicLibraries/$1.plist"
cp "./cmake-build-debug/$1.dylib" "./out/package/Library/MobileSubstrate/DynamicLibraries/$1.dylib"

codesign --force -s - "./out/package/Library/MobileSubstrate/DynamicLibraries/$1.dylib"

cp "./meta/control" "./out/package/DEBIAN/control"

# TODO: Change compression to whatever the normal one is. (Circuliser used gzip and Kirb said it broke Chariz's scripts.)
# gzip is great though because the packages end up tiny.
dpkg-deb -Z gzip -b ./out/package ./out/package.deb

# EXRTRA: Install the .deb file to a device.
scp "./out/package.deb" root@192.168.1.226:/User/Downloads/package.deb
ssh root@192.168.1.226 "dpkg -i /User/Downloads/package.deb"
