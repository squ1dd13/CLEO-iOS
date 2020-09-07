#ifndef UTILITY_HEADER
#define UTILITY_HEADER

#define DeclareFunctionType(name, ret, ...) typedef ret (*name)( __VA_ARGS__ )

#endif