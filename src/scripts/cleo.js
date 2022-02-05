// cleo:mode = running

function wait(time) {
    scmCall(0x0001, time);
}

function showBottomText(key, time) {
    scmCall(0x00bb, key, time, 0);
}

print("cleo.js loaded successfully!");

const gxtKey = "JS_MSG";
addGxtString(gxtKey, "Hello from JavaScript!");

// Wait 10 seconds so we know the game is definitely ready.
wait(10_000);

// Show the message for 3 seconds.
showBottomText(gxtKey, 3_000);

// Wait until the text has stopped showing.
wait(3_000);

// Change the message.
addGxtString(gxtKey, "This is the second message.");

// Show the second message for another 3 seconds.
showBottomText(gxtKey, 3_000);
