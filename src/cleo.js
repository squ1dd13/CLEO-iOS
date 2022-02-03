// cleo:mode = running

print("cleo.js loaded successfully!");

const gxtKey = "JS_MSG";
addGxtString(gxtKey, "Hello from JavaScript!");

// Text.Print(key: gxt_key, time: int, flag: int)
scmCall(0x00BB, gxtKey, 2000, 0);
