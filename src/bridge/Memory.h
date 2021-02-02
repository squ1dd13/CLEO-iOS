#pragma once

#include "Types.h"
#include <mach-o/dyld.h>

namespace Memory {
    inline uint64 AslrSlide() {
        static auto slide = _dyld_get_image_vmaddr_slide(0);
        return (uint64)slide;
    }

    // Offset pointer by ASLR slide (and also cast the result).
    template <typename OutType, typename InType>
    inline OutType Slid(InType inValue) {
        return OutType(uint64(inValue) + AslrSlide());
    }

    // Offset pointer by ASLR slide, cast it and dereference it.
    template <typename OutType, typename InType>
    inline OutType Fetch(InType addr) {
        return *(OutType *)(Slid<void *>(addr));
    }

    template <typename Return = void, typename... Args>
    inline Return Call(uint64 address, Args... args) {
        return Memory::Slid<Return (*)(Args...)>(address)(args...);
    }
}