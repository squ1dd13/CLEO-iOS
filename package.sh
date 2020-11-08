#!/bin/bash
# Takes 1 argument: the name of the output .dylib (but without the .dylib extension).

echo_progress() {
  echo "==> $1"
}

command_exists () {
    type "$1" &> /dev/null ;
}

echo_progress "Creating package dir..."

# Clear or create './out'.
rm -r ./out
mkdir -p ./out/package/Library/MobileSubstrate/DynamicLibraries
mkdir ./out/package/DEBIAN

echo_progress "Copying files to package dir..."

cp "./meta/control" "./out/package/DEBIAN/control"

cp "./meta/$1.plist" "./out/package/Library/MobileSubstrate/DynamicLibraries/$1.plist"
cp "./cmake-build-debug/$1.dylib" "./out/package/Library/MobileSubstrate/DynamicLibraries/$1.dylib"

echo_progress "Signing .dylib"

dylibPath="./out/package/Library/MobileSubstrate/DynamicLibraries/$1.dylib"

if ! command_exists codesign; then
  echo_progress "'codesign' not found. Trying 'ldid'..."

  if ! command_exists ldid; then
    echo "Error: No codesigning utility found."
    exit 1
  fi

  ldid -S "$dylibPath"
else
  codesign --force -s - "$dylibPath"
fi

# If we don't chmod the dylib with +x, it won't work.
# Here we just set the permissions/owners to something 'normal'.
chmod 0755 "$dylibPath"
chown root:staff "$dylibPath"

echo_progress "Packaging..."

# TODO: Change compression to whatever the normal one is. (Circuliser used gzip and Kirb said it broke Chariz's scripts.)
# gzip is great though because the packages end up tiny.
dpkg-deb -Z gzip -b ./out/package ./out/package.deb

echo_progress "Installing..."

ssh_ip=iP8 #192.168.1.94 #$iP8

# EXTRA: Install the .deb file to a device.
scp "./out/package.deb" root@$ssh_ip:/User/Downloads/package.deb
ssh root@$ssh_ip "dpkg -i /User/Downloads/package.deb && (killall -9 gta3sa || echo 'GTA:SA not running')"
