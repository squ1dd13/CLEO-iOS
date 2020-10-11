#ifndef TYPES_HEADER
#define TYPES_HEADER

#define DeclareFunctionType(name, ret, ...) typedef ret (*name)( __VA_ARGS__ )
#define FunctionMember(name, ret, ...) ret (*name)( __VA_ARGS__ )

#define squished __attribute__((packed))

#include <cstdint>
#include <string>

// Basically Go types, so just without the '_t'.
using uint8 = uint8_t;
using int8 = int8_t;

using uint16 = uint16_t;
using int16 = int16_t;

using uint32 = uint32_t;
using int32 = int32_t;

using uint64 = uint64_t;
using int64 = int64_t;

using string_ref = const std::string &;

using char16 = char16_t;
using string16 = const char16_t *;

#endif