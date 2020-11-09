cmake_minimum_required(VERSION 3.17)
project(csios-cmake CXX C)

set(CMAKE_CXX_STANDARD 20)

set(iOS_TOOLCHAIN_BASE /home/squ1dd13/Documents/Projects/iOS-Toolchain/alternative/ios-arm64e-clang-toolchain)
set(iOS_SDK_BASE /home/squ1dd13/Documents/Projects/iOS-Toolchain/SDK/iPhoneOS.sdk)

include_directories(${CMAKE_CURRENT_SOURCE_DIR}/src/include)

include(meta/Core.cmake)

file(GLOB_RECURSE SOURCE_FILES
    ${CMAKE_CURRENT_SOURCE_DIR}/src/*.hpp
    ${CMAKE_CURRENT_SOURCE_DIR}/src/*.cpp
    ${CMAKE_CURRENT_SOURCE_DIR}/src/*.mm
    ${CMAKE_CURRENT_SOURCE_DIR}/src/*.h
)

set(CMAKE_CXX_FLAGS "${CMAKE_CXX_FLAGS} -Wno-format-security -fobjc-abi-version=2 -fobjc-arc -o CSiOS.dylib")
set(CMAKE_C_FLAGS "${CMAKE_C_FLAGS} -Wno-format-security -fobjc-abi-version=2 -fobjc-arc -o CSiOS.dylib")

add_library(csios-cmake SHARED ${SOURCE_FILES} src/include/other/Macros.h src/include/other/RenderWare.h src/code/shared/RenderWare.cpp src/code/scripts/ScriptManager.cpp src/include/scripts/ScriptManager.h src/code/scripts/Script.cpp src/include/scripts/Script.h src/code/shared/UserFolder.cpp src/include/shared/UserFolder.h src/code/mods/TextureHook.cpp src/include/mods/TextureHook.h src/code/mods/StreamHook.cpp src/include/mods/StreamHook.h src/code/shared/Debug.cpp)

message(${CMAKE_CXX_COMPILER})

target_compile_options(csios-cmake PRIVATE -stdlib=libc++)

target_link_directories(csios-cmake PRIVATE ${iOS_LINK_DIRS})
target_include_directories(csios-cmake PRIVATE ${iOS_INCLUDE_DIRS})

add_library(substrate SHARED IMPORTED)
set_target_properties(substrate PROPERTIES
    IMPORTED_LOCATION "${CMAKE_SOURCE_DIR}/lib/libsubstrate.dylib"
)

target_link_libraries(csios-cmake
    "-framework UIKit"
    "-framework Foundation"
    substrate
    -dylib
)

set_target_properties(csios-cmake PROPERTIES
    SUFFIX ".dylib"
    PREFIX ""
    NO_SONAME 1
    OUTPUT_NAME "CSiOS"
)

add_custom_command(TARGET csios-cmake POST_BUILD
    COMMAND /bin/bash ${CMAKE_CURRENT_SOURCE_DIR}/package.sh CSiOS
    WORKING_DIRECTORY ${CMAKE_CURRENT_SOURCE_DIR}
)