#ifndef GAME_MEMORY
#define GAME_MEMORY

#include "../Types.h"
#include <mach-o/dyld.h>
#include <memory>
#include "../Headers/substrate.h"
#include <mach-o/dyld.h>
#include <mach/mach.h>
#include "../Headers/Debug.hpp"

namespace Memory {
    inline uint64 getASLRSlide() {
        static auto slide = _dyld_get_image_vmaddr_slide(0);
        return slide;
    }

    template <typename OutType, typename InType>
    inline OutType slid(InType inValue) {
        return OutType(uint64(inValue) + getASLRSlide());
    }

    template <typename OutType, typename InType>
    inline OutType fetch(InType addr) {
        return *(OutType *)(slid<void *>(addr));
    }

    template <typename AddressType, typename DataPointer>
    inline bool write(AddressType addr, DataPointer data, size_t length) {
        vm_address_t dest = slid<vm_address_t>(addr);

        mach_port_t port = mach_task_self();
        kern_return_t kernelReturn = vm_protect(port, dest, length, false, VM_PROT_READ | VM_PROT_WRITE | VM_PROT_COPY);

        if(kernelReturn != KERN_SUCCESS) {
            Debug::logf("vm_protect failure (%d)", kernelReturn);
            return false;
        }

        kernelReturn = vm_write(port, dest, vm_address_t(data), length);
        if(kernelReturn != KERN_SUCCESS) {
            Debug::logf("vm_write failure (%d)", kernelReturn);
            return false;
        }

        // Reset protection.
        kernelReturn = vm_protect(port, dest, length, false, VM_PROT_READ | VM_PROT_EXECUTE);

        return kernelReturn == KERN_SUCCESS;
    }

    template <typename OriginalAddress, typename Replacement>
    inline Replacement hook(OriginalAddress address, Replacement replacement) {
        void *original = slid<void *>(address);
        MSHookFunction((void *)original, (void *)replacement, (void **)&original);

        return Replacement(original);
    }
};

#endif