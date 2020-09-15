#include "Custom/Scripts.hpp"
#include "Game/Menus.hpp"
#include "Game/Text.hpp"
#include "Game/Touch.hpp"

void inject() {
    Debug::logf("ASLR slide is 0x%llx (%llu decimal)", Memory::getASLRSlide(), Memory::getASLRSlide());

    Scripts::hook();
    Menus::hook();
    Touch::hook();
    Text::hook();
}

void cleanUp() {
    Scripts::release();
}