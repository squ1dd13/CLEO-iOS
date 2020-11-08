#include <shared/Interface.h>
#include <shared/Memory.h>
#include <cmath>

namespace Interface {

    bool Touch::interceptTouches = false;

// The zone states (pressed or not pressed).
    static bool screenZones[9]{};

// Whether zones have had their states updated since the last update.
// This allows multiple touches in the same zone to be handled correctly.
    static bool updatedZones[9]{};

    static float viewportWidth = 1.f, viewportHeight = 1.f;

    void Touch::setViewportSize(float w, float h) {
        viewportWidth = w;
        viewportHeight = h;
    }

    int Touch::calculateZone(float x, float y) {
        int xSegment = std::ceil((x / viewportWidth) * 3);
        int ySegment = std::ceil((y / viewportHeight) * 3);

        return (ySegment + (3 * xSegment)) - 3;
    }

    void Touch::updateZone(int n, bool b) {
        if (updatedZones[n]) {
            screenZones[n] |= b;
        } else {
            screenZones[n] = b;
            updatedZones[n] = true;
        }
    }

    void Touch::beginUpdates() {
        std::fill_n(updatedZones, 9, 0);
        std::fill_n(screenZones, 9, 0);
    }

    bool Touch::testZone(int n) {
        return screenZones[n];
    }

    void Touch::handle() const {
        if (interceptTouches) {
            int zone = calculateZone(destX, destY);

            if (change == Type::Moved) {
                int previousZone = calculateZone(srcX, srcY);

                if (previousZone != zone) {
                    // Touch moved out of its old zone.
                    updateZone(previousZone, false);
                } else {
                    updateZone(previousZone, true);
                }
            } else {
                updateZone(zone, change != Type::Up);
            }
        }

        // Call the game's touch handler.
        Memory::call(0x1004e831c, destX, destY, change, timestamp);
    }

    Touch::Touch(float oldX, float oldY, float newX, float newY, Type stage, double time) {
        srcX = oldX;
        srcY = oldY;
        destX = newX;
        destY = newY;
        change = stage;
        timestamp = time;
    }
}