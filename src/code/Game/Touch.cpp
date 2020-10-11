#include "Game/Touch.hpp"
#include "Core.hpp"
#include "Game/Memory.hpp"
#include <cmath>

enum TouchStage : uint64 {
    TouchUp = 0,
    TouchDown = 2,
    TouchMoved = 3
};

static float viewportWidth = 1.f, viewportHeight = 1.f;

void Touch::setViewportSize(float w, float h) {
    viewportWidth = w;
    viewportHeight = h;
}

inline int zoneForPoint(float x, float y) {
    int xthird = std::ceil((x / viewportWidth) * 3);
    int ythird = std::ceil((y / viewportHeight) * 3);

    return (ythird + (3 * xthird)) - 3;
}

DeclareFunctionType(HandleTouchFunc, void, float, float, TouchStage, double);
static HandleTouchFunc origTouch;

static bool screenZones[9] {};

// 0x1004e831c
// This gets called by the Objective-C touch handler (touchesBegan:withEvent:) on EAGLView.
void handleTouch_old(float x, float y, TouchStage stage, double time) {
    int zone = zoneForPoint(x, y);
    Debug::assertf(0 < zone && zone < 10, "Touch zone range error (%d)! Has the viewport size changed?", zone);

    if(stage == TouchStage::TouchUp) {
        screenZones[zone] = false;
    } else {
        // NOTE: Not sure about Android behaviour for touch zones. Does a TouchMoved event in a zone count if there was no TouchDown?
        // At the moment, the user has to touch the zone, so sliding your finger between zones won't work.
        // I'm assuming that's how the Android version behaves.
        screenZones[zone] |= stage == TouchStage::TouchDown;
    }

    origTouch(x, y, stage, time);
}

hookf(handleTouch, 0x1004e831c, {
    int zone = zoneForPoint(x, y);
    Debug::assertf(0 < zone && zone < 10, "Touch zone range error (%d)! Has the viewport size changed?", zone);

    if(stage == TouchStage::TouchUp) {
        screenZones[zone] = false;
    } else {
        // NOTE: Not sure about Android behaviour for touch zones. Does a TouchMoved event in a zone count if there was no TouchDown?
        // At the moment, the user has to touch the zone, so sliding your finger between zones won't work.
        // I'm assuming that's how the Android version behaves.
        screenZones[zone] |= stage == TouchStage::TouchDown;
    }

    original(x, y, stage, time);
}, void, float x, float y, TouchStage stage, double time);

bool Touch::touchAreaPressed(int n) {
    return screenZones[n];
}

void Touch::hook() {
//    origTouch = Memory::hook(0x1004e831c, handleTouch);
}