/*
 * Main.mm does the main hooking and initialisation/cleanup for the rest of the code.
 * Everything starts from here.
 */

#include <UIKit/UIKit.h>
#include <Custom/HookObjC.hpp>
#include <Game/Touch.hpp>
#include <Util/Macros.hpp>

// Now we're writing real Objective-C++ rather than Logos.
// This is good because it means pretty much any program can analyse our code.

static UITextView *overlay = nullptr;

// @hookbase makes the hook class a subclass of the second argument.
// In this case, it means we can use 'self' as a UIView * and not an NSObject *.
@hookbase(EAGLView, UIView)

- (void)createFramebuffer {
    orig();

    // We need to know the
    float size[2] {
        float(self.bounds.size.width * self.layer.contentsScale),
        float(self.bounds.size.height * self.layer.contentsScale)
    };

    Touch::setViewportSize(size[0], size[1]);
    Debug::logf("contentsScale = %f", self.layer.contentsScale);
    Debug::logf("VP size is { %f, %f }", size[0], size[1]);

#ifdef LOG_OVERLAY
    if(!overlay) {
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
    if(overlay && screenLog.updated) {
        NSMutableString *str = [NSMutableString stringWithString:overlay.text];

        for(const std::string &s : screenLog.log) {
            [str appendFormat:@"%s\n", s.c_str()];
        }

        screenLog.updated = false;

        overlay.text = str;

        // TODO: Remove this?
        if(overlay.text.length > 0) {
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
    Debug::logf("Showing splash screen");

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
        Debug::logf("Showing splash screen");
        customLabel.alpha = 0.0f;
    } completion:^(BOOL finished) {
        customLabel.hidden = true;
        [customLabel removeFromSuperview];
    }];

    orig();
    [self.view addSubview:customLabel];
}

@end

#include "Custom/Scripts.hpp"
#include "Game/Text.hpp"

// TODO: Load scripts in at end of game load sequence (0x100240178).

@ctor {
    Debug::logf("ASLR slide is 0x%llx (%llu decimal)", Memory::getASLRSlide(), Memory::getASLRSlide());

    Scripts::hook();
    Touch::hook();
    Text::hook();
}

@dtor {
    Scripts::release();
}