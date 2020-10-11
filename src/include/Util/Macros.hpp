//
// Created on 11/10/2020.
//

#ifndef CSIOS_CMAKE_MACROS_HPP
#define CSIOS_CMAKE_MACROS_HPP

#define ctor class NSObject; __attribute__((constructor)) void ctorln##__LINE__()
#define dtor class NSObject; __attribute__((destructor)) void dtorln##__LINE__()

#endif //CSIOS_CMAKE_MACROS_HPP
