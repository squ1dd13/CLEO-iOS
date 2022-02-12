// cleo:mode = running

function wait(time) {
    console.log("Called wait with time", time);
    scmCall(0x0001, [time]);
}

function showBottomText(key, time) {
    console.log("Called SBT with", key, time);
    scmCall(0x00bb, [key, time, 0]);
}

function sleep(sleepDuration) {
    var now = new Date().getTime();
    while (new Date().getTime() < now + sleepDuration) { /* Do nothing */ }
}

console.log("cleo.js loaded successfully!");

const gxtKey = "JS_MSG";
setGxtKeyValue(gxtKey, "Hello from JavaScript!");

// Wait 10 seconds so we know the game is definitely ready.
sleep(10_000);

// Show the message for 3 seconds.
showBottomText(gxtKey, 3_000);

// Wait until the text has stopped showing.
sleep(4_000);

// Change the message.
setGxtKeyValue(gxtKey, "This is the second message.");

// Show the second message for another 3 seconds.
showBottomText(gxtKey, 3_000);

console.log("Script finished");

// let moneyVar = ScmVar.local(10);

// // Get the player's money.
// scmCall(0x010b, 0, moneyVar);

// print("money:", moneyVar.get());