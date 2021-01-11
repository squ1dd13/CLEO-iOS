/*
 * Main.mm does the main hooking and initialisation/cleanup for the rest of the code.
 * Everything starts from here.
 */

#include <UIKit/UIKit.h>
#include "Hook.h"
#include "../shared/Interface.h"
#include "Macros.h"
#include "../shared/Memory.h"
#include "../shared/Text.h"

// Now we're writing real Objective-C++ rather than Logos.
// This is good because it means pretty much any program can analyse our code.

// One thing we can't easily do with macros is %property, so we have to use normal variables.
static UITextView *overlay = nullptr;

// @hookbase makes the hook class a subclass of the second argument.
// In this case, it means we can use 'self' as a UIView * and not an NSObject *.
@hookbase(EAGLView, UIView)

void processTouches(UIView *view, NSSet *touches, Interface::Touch::Type type) {
    if ([touches count] == 0) {
        return;
    }

    Interface::Touch::beginUpdates();
    for (UITouch *touch in touches) {
        auto oldPos = [touch previousLocationInView:view];
        auto pos = [touch locationInView:view];

        auto oldX = float(oldPos.x * view.layer.contentsScale);
        auto oldY = float(oldPos.y * view.layer.contentsScale);

        auto x = float(pos.x * view.layer.contentsScale);
        auto y = float(pos.y * view.layer.contentsScale);

        double time = [touch timestamp];

        Interface::Touch(oldX, oldY, x, y, type, time).handle();
    }
}

- (void)touchesBegan:(NSSet *)touches withEvent:(UIEvent *)event {
    processTouches(self, touches, Interface::Touch::Type::Down);
}

- (void)touchesMoved:(NSSet<UITouch *> *)touches withEvent:(UIEvent *)event {
    processTouches(self, touches, Interface::Touch::Type::Moved);
}

- (void)touchesEnded:(NSSet *)touches withEvent:(UIEvent *)event {
    processTouches(self, touches, Interface::Touch::Type::Up);
}

- (void)touchesCancelled:(NSSet *)touches withEvent:(UIEvent *)event {
    processTouches(self, touches, Interface::Touch::Type::Up);
}

- (void)createFramebuffer {
    orig();

    // We need to know the
    float size[2] {
        float(self.bounds.size.width * self.layer.contentsScale),
        float(self.bounds.size.height * self.layer.contentsScale)
    };

    Interface::Touch::setViewportSize(size[0], size[1]);

#ifdef LOG_OVERLAY
    if (!overlay) {
        overlay = [[UITextView alloc] initWithFrame:[[UIScreen mainScreen] bounds]];
        overlay.userInteractionEnabled = false;
        overlay.editable = false;
        overlay.backgroundColor = [UIColor clearColor];
        overlay.textColor = [UIColor whiteColor];
        overlay.font = [UIFont fontWithName:@"Menlo" size:8.f];
        [self addSubview:overlay];
    }
#endif
}

#ifdef LOG_OVERLAY

- (bool)presentFramebuffer {
    if (overlay && Log::updated) {
        NSMutableString *str = [NSMutableString stringWithString:@""];

        for (const std::string &s : Log::log) {
            [str appendFormat:@"%s\n", s.c_str()];
        }

        Log::updated = false;

        overlay.text = str;

        // TODO: Remove this?
        if (overlay.text.length > 0) {
            NSRange bottom = NSMakeRange(overlay.text.length - 1, 1);
            [overlay scrollRangeToVisible:bottom];
        }
    }

    return bool(orig());
}

#endif

@end

@hookbase(LegalSplash, UIViewController)

// We hook here to display the "CSiOS" splash screen.
- (void)viewDidLoad {
    UILabel *customLabel = [[UILabel alloc] initWithFrame:self.view.bounds];
    customLabel.text = @"CSiOS";
    customLabel.textColor = [UIColor whiteColor];
    customLabel.font = [UIFont fontWithName:@"PricedownGTAVInt" size:50.f];
    customLabel.textAlignment = NSTextAlignmentCenter;

    // If the user taps here they can actually skip our splash and the legal one.
    customLabel.userInteractionEnabled = false;

    customLabel.backgroundColor = [UIColor blackColor];

    customLabel.hidden = false;
    customLabel.alpha = 1.f;

    [UIView animateWithDuration:0.2 delay:1.0 options:UIViewAnimationOptionCurveEaseIn animations:^{
        customLabel.alpha = 0.0f;
    }                completion:^(BOOL finished) {
        customLabel.hidden = true;
        [customLabel removeFromSuperview];
    }];

    orig();
    [self.view addSubview:customLabel];
}

@end

#include "../scripts/ScriptManager.h"

HookFunction(loadGame, 0x100240178, {
    original(datPath);

    Interface::Touch::interceptTouches = true;
    ScriptManager::Init();
}, void, char *datPath)

@ctor {
    Log("ASLR slide is 0x%llx (%llu decimal)", Memory::getASLRSlide(), Memory::getASLRSlide());

    Text::hook();
}

@dtor {
    ScriptManager::UnloadAll();
}