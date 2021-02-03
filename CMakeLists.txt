cmake_minimum_required(VERSION 3.17)

set(CMAKE_C_COMPILER_WORKS 1)
set(CMAKE_CXX_COMPILER_WORKS 1)

project(cleo CXX C)

set(CMAKE_CXX_STANDARD 20)

set(iOS_TOOLCHAIN_BASE /home/squ1dd13/Documents/Projects/iOS-Toolchain/alternative/ios-arm64e-clang-toolchain)
set(iOS_SDK_BASE /home/squ1dd13/Documents/Projects/iOS-Toolchain/SDK/iPhoneOS.sdk)

include_directories(${CMAKE_CURRENT_SOURCE_DIR}/src)
include_directories(${CMAKE_CURRENT_SOURCE_DIR}/lib)

include(meta/Core.cmake)

file(GLOB_RECURSE SOURCE_FILES
    ${CMAKE_CURRENT_SOURCE_DIR}/src/*.hpp
    ${CMAKE_CURRENT_SOURCE_DIR}/src/*.cpp
    ${CMAKE_CURRENT_SOURCE_DIR}/src/*.mm
    ${CMAKE_CURRENT_SOURCE_DIR}/src/*.h
    )

set(CMAKE_CXX_FLAGS "${CMAKE_CXX_FLAGS} -Wno-format-security -fobjc-abi-version=2 -fobjc-arc -o CLEO.dylib")
set(CMAKE_C_FLAGS "${CMAKE_C_FLAGS} -Wno-format-security -fobjc-abi-version=2 -fobjc-arc -o CLEO.dylib")

add_library(cleo SHARED ${SOURCE_FILES} src/scripts/Manager.cpp src/scripts/Manager.h src/scripts/Script.cpp src/scripts/Script.h src/user/Directory.cpp src/user/Directory.h src/Load.cpp src/hook/Func.h src/scripts/Menu.mm src/scripts/Menu.h src/scripts/Mobile.cpp src/scripts/Mobile.h)

message(${CMAKE_CXX_COMPILER})

target_compile_options(cleo PRIVATE -stdlib=libc++)

target_link_directories(cleo PRIVATE ${iOS_LINK_DIRS})
target_include_directories(cleo PRIVATE ${iOS_INCLUDE_DIRS})

add_library(substrate SHARED IMPORTED)
set_target_properties(substrate PROPERTIES
    IMPORTED_LOCATION "${CMAKE_SOURCE_DIR}/lib/libsubstrate.dylib"
    )

target_link_libraries(cleo
    "-framework UIKit"
    "-framework Foundation"
    "-framework CoreGraphics"
    "-framework QuartzCore"
    "-framework CoreImage"
    substrate
    -dylib
    )

set_target_properties(cleo PROPERTIES
    SUFFIX ".dylib"
    PREFIX ""
    NO_SONAME 1
    OUTPUT_NAME "CLEO"
    )