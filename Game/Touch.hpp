#ifndef TOUCH_HEADER
#define TOUCH_HEADER

#include "../Headers/Types.h"

namespace Touch {

// Android touch zones.
bool touchAreaPressed(int n);
void hook();

void setViewportSize(float w, float h);
};

#endif