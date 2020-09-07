#include "HookManager.hpp"
#include <mach-o/dyld.h>
#include "substrate.h"

size_t HookManager::HookAddress(size_t hookBaseAddr, void *hookImpl) {
    // Get the ASLR slide so we know how much to offset the address.
    intptr_t slide = _dyld_get_image_vmaddr_slide(0);

    void *hooked = (void *)(hookBaseAddr + slide);
    MSHookFunction(hooked, hookImpl, &hooked);

    return size_t(hooked);
}

size_t HookManager::getSlide() {
    static auto slide = _dyld_get_image_vmaddr_slide(0);
    return slide;
}