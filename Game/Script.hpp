
#include "../Headers/Types.h"
#include <string>

#ifndef SCRIPT_HEADER
#define SCRIPT_HEADER

// This class is a copy of what the game uses (so we can integrate our code with the game's).
// However, there are also some utility methods and some reimplementations. These are only 
//  used for custom scripts. Methods don't change the field positions, so the class can still
//  be used with game code.

// Unknown/unnamed fields have been left as individual bytes in case they're needed in the future.
// It's easier to rename a single field than it is to split an array. I'm lazy.
class GameScript {
  public:
    GameScript *nextScript;
    GameScript *previousScript;

    char name[8];

    uint8 *startPointer;
    uint8 *currentPointer;

    uint8 *callStack[8];
    uint16 callStackPos;

  private:
    uint8 field_0x6A, field_0x6B;

  public:
    // Unsure about size here (probably really 32 and not 42, but we don't use this ATM anyway).
    uint32 localStorage[42];

  private:
    uint8 field_0x114;

  public:
    bool conditionResult;

  private:
    uint8 field_0x116,
        field_0x117,
        field_0x118,
        field_0x119,
        field_0x11A,
        field_0x11B;

  public:
    // When the script will next receive focus.
    uint32 activationTime;

    uint16 conditionCount;

    bool instructionIsConditional;

  private:
    uint8 field_0x123,
        field_0x124,
        field_0x125,
        field_0x126,
        field_0x127,
        field_0x128,
        field_0x129,
        field_0x12A,
        field_0x12B;

  public:
    bool localStorageIsGlobalStorage;

    static GameScript load(const std::string &path);
    static uint32 time();

    // Reimplementations of game code (for the most part).
    void executeBlock();
    uint8 executeInstruction();

    void release();

  private:
    static uint64 calculateHandlerOffset(unsigned opcode);
} __attribute__((__packed__));

// static_assert(sizeof(GameScript) == 301, "sizeof(GameScript) must be 301");

#endif