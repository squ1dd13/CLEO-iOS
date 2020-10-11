/*
 * Main.mm does the main hooking and initialisation/cleanup for the rest of the code.
 * Everything starts from here.
 */

#include <UIKit/UIKit.h>
#include <Custom/HookObjC.hpp>
#include <Game/Touch.hpp>

// Now we're writing real Objective-C++ rather than Logos.
// This is good because it means pretty much any program can analyse our code.

// @hookbase makes the hook class a subclass of the second argument.
// In this case, it means we can use 'self' as a UIView * and not an NSObject *.
@hookbase(EAGLView, UIView)

- (void)createFramebuffer {
    orig();

    float size[2] {
        float(self.bounds.size.width * self.layer.contentsScale),
        float(self.bounds.size.height * self.layer.contentsScale)
    };

    Touch::setViewportSize(size[0], size[1]);
    Debug::logf("contentsScale = %f", self.layer.contentsScale);
    Debug::logf("VP size is { %f, %f }", size[0], size[1]);
}

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
#include "Game/Menus.hpp"
#include "Game/Text.hpp"

// Uses the constructor and destructor as %ctor and %dtor.
struct TweakManager {
    TweakManager() {
        Debug::logf("ASLR slide is 0x%llx (%llu decimal)", Memory::getASLRSlide(), Memory::getASLRSlide());

        Scripts::hook();
        Menus::hook();
        Touch::hook();
        Text::hook();
    }

    ~TweakManager() {
        Scripts::release();
    }
};

// We need to create an instance so the constructor and destructor are called.
static TweakManager mgr;