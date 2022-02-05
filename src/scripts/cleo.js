// cleo:mode = running

class ScmVar {
    location;
    index;

    static Global = Symbol("Global");
    static Local = Symbol("Local");

    constructor(location, index) {
        this.location = location;
        this.index = index;
    }

    static local(index) {
        return new ScmVar(ScmVar.Local, index);
    }

    static global(index) {
        return new ScmVar(ScmVar.Global, index);
    }

    get() {
        let isGlobal = this.location === ScmVar.Global;
        return scmVarVal(this.index, isGlobal);
    }

    set(value) {
        let isGlobal = this.location === ScmVar.Global;
        return scmVarVal(this.index, isGlobal, value);
    }

    canBeScm() {
        return true;
    }
}

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

let moneyVar = ScmVar.local(10);

// Get the player's money.
scmCall(0x010b, 0, moneyVar);

print("money:", moneyVar.get());