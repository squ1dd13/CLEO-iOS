#[[
    Core.cmake

    Basic iOS stuff.
]]

# Input variables:
#   iOS_TOOLCHAIN_BASE - Folder containing /bin, /lib and other folders.
#                        This is "/path/to/ios-arm64e-clang-toolchain" if using
#                        https://github.com/sbingner/llvm-project.
#
#   iOS_SDK_BASE -       Path to SDK. Example:
#                         /home/me/dev/iPhoneOS.sdk

# Output variables:
#   iOS_INCLUDE_DIRS - Paths to include using target_include_directories.
#
#   iOS_LINK_DIRS    - Paths to link using target_link_directories.

set(CMAKE_C_COMPILER "${iOS_TOOLCHAIN_BASE}/bin/clang")
set(CMAKE_CXX_COMPILER "${iOS_TOOLCHAIN_BASE}/bin/clang++")

set(extra_flags "-isysroot '${iOS_SDK_BASE}' -target arm64-apple-darwin -arch arm64 -arch arm64e")

set(CMAKE_C_FLAGS "${CMAKE_C_FLAGS} ${extra_flags}")
set(CMAKE_CXX_FLAGS "${CMAKE_CXX_FLAGS} ${extra_flags}")

unset(extra_flags)

# Specifying 4.2.1 could cause issues...
set(iOS_INCLUDE "${iOS_SDK_BASE}/usr/include" "${iOS_SDK_BASE}/usr/include/c++/4.2.1" "${iOS_SDK_BASE}/usr/include/pthread")
set(iOS_LINK "${iOS_SDK_BASE}/usr/lib")