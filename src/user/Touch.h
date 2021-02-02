#pragma once

#include "bridge/Types.h"

class Touch {
    static int CalculateZone(float x, float y);
    static void UpdateZone(int n, bool b);

public:
    enum class Type: uint64 {
        Up = 0, Down = 2, Moved = 3
    } change;

    float srcX, srcY;
    float destX, destY;

    double timestamp;

    void Handle() const;

    // Check if touch zone 'n' is currently pressed.
    static bool TestZone(int n);

    // Sets up combination resolution for handle() calls.
    static void BeginUpdates();

    // Sets the viewport size for calculating touch zone areas.
    static void SetViewportSize(float w, float h);

    static bool interceptTouches;

    Touch(float oldX, float oldY, float newX, float newY, Type stage, double time);
};