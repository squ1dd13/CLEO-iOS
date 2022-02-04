// cleo:mode = running

print("cleo.js loaded successfully!");

const gxtKey = "JS_MSG";
addGxtString(gxtKey, "Hello from JavaScript!");

// wait [time: int]
scmCall(0x0001, 10000);

// Text.Print(key: gxt_key, time: int, flag: int)
scmCall(0x00BB, gxtKey, 2000, 0);
