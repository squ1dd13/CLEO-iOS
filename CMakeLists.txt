cmake_minimum_required(VERSION 3.17)
project(csios-cmake CXX C)

set(CMAKE_CXX_STANDARD 20)

set(CMAKE_IOS_DEVELOPER_ROOT /Applications/Xcode.app/Contents/Developer/Platforms/iPhoneOS.platform/Developer)
set(CMAKE_IOS_SDK_ROOT /Applications/Xcode.app/Contents/Developer/Platforms/iPhoneOS.platform/Developer/SDKs/iPhoneOS.sdk)
set(CMAKE_OSX_SYSROOT /Applications/Xcode.app/Contents/Developer/Platforms/iPhoneOS.platform/Developer/SDKs/iPhoneOS.sdk)

include(iOS.cmake)

message(${CMAKE_OSX_SYSROOT})

include_directories(${CMAKE_CURRENT_SOURCE_DIR}/src/include)

file(GLOB_RECURSE SOURCE_FILES
        ${CMAKE_CURRENT_SOURCE_DIR}/src/*.hpp
        ${CMAKE_CURRENT_SOURCE_DIR}/src/*.cpp
        ${CMAKE_CURRENT_SOURCE_DIR}/src/*.mm
        ${CMAKE_CURRENT_SOURCE_DIR}/src/*.h
        )

set(CMAKE_CXX_FLAGS "${CMAKE_CXX_FLAGS} -Wno-format-security -arch arm64 -arch arm64e -fobjc-abi-version=2 -fobjc-arc -std=gnu++11 -stdlib=libc++ -isysroot ${CMAKE_OSX_SYSROOT}")
set(CMAKE_C_FLAGS "${CMAKE_C_FLAGS} -Wno-format-security -arch arm64 -arch arm64e -fobjc-abi-version=2 -fobjc-arc -isysroot ${CMAKE_OSX_SYSROOT}")

add_library(csios-cmake SHARED ${SOURCE_FILES} src/include/Util/Macros.hpp)

add_library(substrate SHARED IMPORTED)
set_target_properties(substrate PROPERTIES
        IMPORTED_LOCATION "${CMAKE_SOURCE_DIR}/lib/libsubstrate.dylib"
        )

target_link_libraries(csios-cmake
        "-framework UIKit"
        "-framework Foundation"
        substrate
        )

set_target_properties(csios-cmake PROPERTIES PREFIX "")
set_target_properties(csios-cmake PROPERTIES OUTPUT_NAME "CSiOS")

add_custom_command(TARGET csios-cmake POST_BUILD
        COMMAND /bin/bash ${CMAKE_CURRENT_SOURCE_DIR}/package.sh CSiOS
        WORKING_DIRECTORY ${CMAKE_CURRENT_SOURCE_DIR}
        COMMENT "Packaging and installing..."
        )