THEOS_DEVICE_IP = 192.168.1.226
GO_EASY_ON_ME = 1
ARCHS = arm64 arm64e
TARGET = iphone:11.2:11.2
CC=/usr/local/opt/llvm/bin/clang
CXX=/usr/local/opt/llvm/bin/clang++
include ~/theos/makefiles/common.mk
TWEAK_NAME = CSiOS
CSiOS_FILES = Main.xm Hooks/Debug.xm Game/Script.cpp Custom/ScriptSystem.cpp Game/Text.cpp Game/Touch.cpp Custom/Instructions.cpp
CSiOS_CFLAGS = -fobjc-arc -Wno-format-security -Wno-auto-var-id -Wno-deprecated -Wno-deprecated-declarations -Wno-unused-function -Wno-unused-private-field
CSiOS_CFLAGS += -std=c++17 -stdlib=libc++
CSiOS_LIBRARIES = c++
include $(THEOS_MAKE_PATH)/tweak.mk