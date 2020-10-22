#ifndef GAME_MEMORY
#define GAME_MEMORY

#include "Core.hpp"
#include <mach-o/dyld.h>
#include <mach/mach.h>
#include <memory>

namespace Memory {

// Get ASLR offset.
inline uint64 getASLRSlide() {
    static auto slide = _dyld_get_image_vmaddr_slide(0);
    return slide;
}

// Offset pointer by ASLR slide (and also cast the result).
template <typename OutType, typename InType>
inline OutType slid(InType inValue) {
    return OutType(uint64(inValue) + getASLRSlide());
}

// Offset pointer by ASLR slide, cast it and dereference it.
template <typename OutType, typename InType>
inline OutType fetch(InType addr) {
    return *(OutType *)(slid<void *>(addr));
}

// Write to game memory. Handles R/W protection.
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
    return vm_protect(port, dest, length, false, VM_PROT_READ | VM_PROT_EXECUTE) == KERN_SUCCESS;
}

// Hook a function using Substrate and return a function pointer for the original implementation.
template <typename OriginalAddress, typename Replacement>
inline Replacement hook(OriginalAddress address, Replacement replacement) {
    void *original = slid<void *>(address);
    MSHookFunction((void *)original, (void *)replacement, (void **)&original);

    return Replacement(original);
}

template <typename Return = void, typename ...Args>
inline Return call(uint64 address, Args... args) {
    return Memory::slid<Return(*)(Args...)>(address)(args...);
}

}; // namespace Memory

// %hookf equivalent. Creates a struct that contains the new implementation.
// A single static instance of that struct is then created to run the constructor,
//  which calls Memory::hook. %orig can be called with 'original'.
#define hookf(name, addr, impl, ret, ...) typedef ret (*name##_funcptr)( __VA_ARGS__ );                                          \
struct hookf_##name##_ { \
                                      \
            static name##_funcptr original; \
           static ret name( __VA_ARGS__ )     impl                                                                               \
           hookf_##name##_() noexcept {                                                                                   \
               original = Memory::hook(addr, name); \
           }                                                      \
                                                                 \
};                                                                       \
name##_funcptr hookf_##name##_::original; \
 static hookf_##name##_ hookf_##name##_##_INSTANCE;

#endif