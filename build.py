#!/usr/bin/python

import os
import shutil
import sys
import tempfile

args = sys.argv[1:]

verbose = "--verbose" in args
rootless = "--rootless" in args

build_cmd = ["cargo"]

# If we're on macOS, we can just build a dylib directly.
building_dylib = sys.platform == "darwin"

if building_dylib:
    # Override the "staticlib" that we set in Cargo.toml.
    # bug: This just doesn't work.
    build_cmd.append('--config lib.crate-type[\\"cdylib\\"]')

build_cmd.append("build --target aarch64-apple-ios")

cleo_dir = os.getcwd()

if "--release" in args:
    output_dir = os.path.join(cleo_dir, "target/aarch64-apple-ios/release")

    build_cmd.append("--release")
else:
    output_dir = os.path.join(cleo_dir, "target/aarch64-apple-ios/debug")

    # The `debug` feature is a part of CLEO that we use to run different code in debug and release
    # builds.
    build_cmd.append("--features debug")

# Turn the different parts of our build command into a single string.
build_cmd = " ".join(build_cmd)


def force_system(cmd: str, err_name: str):
    if verbose:
        print(cmd)

    if os.system(cmd) != 0:
        print(err_name, "returned a non-zero exit code!")
        exit(1)


# Run the build command.
force_system(build_cmd, "'cargo build'")


def force_var(variable: str) -> str:
    try:
        return os.environ[variable]
    except:
        print(variable, "not set!")
        exit(1)


def double_quoted(s: str) -> str:
    return '"' + s + '"'


def single_quoted(s: str) -> str:
    return "'" + s + "'"


dylib_path = os.path.join(output_dir, "libcleo.dylib")

# If we built as "staticlib" (which is what happens if we're not on macOS), we will have an AR
# archive that we need to turn into a dylib. We can do that with clang.
if not building_dylib:
    ar_path = os.path.join(output_dir, "libcleo.a")

    clang_cmd = [
        double_quoted(force_var("CLEO_CLANG")),
        "-fpic -shared -Wl,-all_load",
        ar_path,
        "-o",
        dylib_path,
        "-isysroot",
        double_quoted(force_var("CLEO_IOS_SDK")),
        "-target arm64-apple-darwin -framework CoreFoundation -framework Security",
    ]

    clang_cmd = " ".join(clang_cmd)

    force_system(clang_cmd, "clang")

# At this point, we should have a dylib at `dylib_path`, regardless of platform.

ldid_cmd = [double_quoted(force_var("CLEO_LDID")), "-S", dylib_path]
ldid_cmd = " ".join(ldid_cmd)

force_system(ldid_cmd, "ldid")

package = "--package" in args
install = "--install" in args


def copy_to_device(host: str, local_src: str, remote_dst: str):
    scp_cmd = ["scp -q", local_src, "root@" + host + ":" + remote_dst]

    scp_cmd = " ".join(scp_cmd)
    force_system(scp_cmd, "scp")


plist_path = os.path.join(cleo_dir, "deb/cleo.plist")

if package:
    with tempfile.TemporaryDirectory() as package_dir:
        debian_dir = os.path.join(package_dir, "DEBIAN")
        os.mkdir(debian_dir)

        dylib_dir_path = "Library/MobileSubstrate/DynamicLibraries"

        if rootless:
            dylib_dir_path = "var/jb/" + dylib_dir_path

        dylib_dir = os.path.join(package_dir, dylib_dir_path)

        os.makedirs(dylib_dir)

        control_ext = "rootless" if rootless else "rootful"

        control_path = os.path.join(cleo_dir, "deb/control." + control_ext)
        shutil.copy(control_path, os.path.join(debian_dir, "control"))

        shutil.copy(plist_path, os.path.join(dylib_dir, "CLEO.plist"))
        shutil.copy(dylib_path, os.path.join(dylib_dir, "CLEO.dylib"))

        deb_name = "cleo.rootless.deb" if rootless else "cleo.rootful.deb"

        deb_path = os.path.join(output_dir, deb_name)

        # If there's an old .deb file, remove it.
        if os.path.exists(deb_path):
            os.remove(deb_path)

        dpkg_cmd = ["dpkg-deb -Z xz -b", package_dir, deb_path]
        dpkg_cmd = " ".join(dpkg_cmd)

        force_system(dpkg_cmd, "dpkg-deb")

    if install:
        host = force_var("CLEO_INSTALL_HOST")
        copy_to_device(host, deb_path, "/tmp/cleo.deb")

        install_cmd = "dpkg -i /tmp/cleo.deb && rm -f /tmp/cleo.deb"

        shell_cmd = ["exec \$SHELL -l -c", single_quoted(install_cmd)]
        shell_cmd = " ".join(shell_cmd)

        ssh_cmd = ["ssh", "root@" + host, double_quoted(shell_cmd)]
        ssh_cmd = " ".join(ssh_cmd)

        force_system(ssh_cmd, "ssh")

elif install:
    # We're installing without packaging. That means overwriting the old .dylib and .plist files
    # with the new ones.

    host = force_var("CLEO_INSTALL_HOST")

    target = (
        "/var/jb/Library/MobileSubstrate/DynamicLibraries/CLEO"
        if rootless
        else "/Library/MobileSubstrate/DynamicLibraries/CLEO"
    )

    copy_to_device(host, plist_path, target + ".plist")
    copy_to_device(host, dylib_path, target + ".dylib")
