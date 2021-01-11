//
// Created on 11/10/2020.
//

#ifndef CSIOS_CMAKE_MACROS_H
#define CSIOS_CMAKE_MACROS_H

// FIXME: Ends up with ctorln__LINE__ and not ctorln<line no>
#define mcrcat(x, y) x##y
#define ctor class NSObject; __attribute__((constructor)) void mcrcat(ctorln, __LINE__)()
#define dtor class NSObject; __attribute__((destructor)) void mcrcat(dtorln, __LINE__)()

#endif //CSIOS_CMAKE_MACROS_H
