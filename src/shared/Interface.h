#ifndef TOUCH_HEADER
#define TOUCH_HEADER

#include <Core.h>

namespace Interface {

    class Touch {
        static int calculateZone(float x, float y);
        static void updateZone(int n, bool b);

    public:
        enum class Type : uint64 {
            Up = 0,
            Down = 2,
            Moved = 3
        } change;

        float srcX, srcY;
        float destX, destY;

        double timestamp;

        void handle() const;

        // Check if touch zone 'n' is currently pressed.
        static bool testZone(int n);

        // Sets up combination resolution for handle() calls.
        static void beginUpdates();

        // Sets the viewport size for calculating touch zone areas.
        static void setViewportSize(float w, float h);

        static bool interceptTouches;

        Touch(float oldX, float oldY, float newX, float newY, Type stage, double time);
    };
}

#endif