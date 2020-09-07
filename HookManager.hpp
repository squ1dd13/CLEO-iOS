#include <ctype.h>

class HookManager {
public:
    static size_t HookAddress(size_t hookBaseAddr, void *hookImpl);
    static size_t getSlide();
};