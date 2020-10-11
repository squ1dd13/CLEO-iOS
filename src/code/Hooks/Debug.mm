#include "Util/Debug.hpp"
#include <UIKit/UIKit.h>

std::vector<std::string> Debug::logStrings;

// This is a *very* primitive system, so if the game runs too long it'll run out of memory.

@interface EAGLView : UIView
@property(nonatomic, strong) UITextView *overlay;
@end

// Not using Logos means we can hook inside #ifdefs.
#ifdef SHOW_DEBUG_OVERLAY
#include <Custom/HookObjC.hpp>

@hookbase(EAGLView, UIView)

-(id)initWithCoder:(id)coder {
    auto me = (__bridge EAGLView *)orig();

    me.overlay = [[UITextView alloc] initWithFrame:[[UIScreen mainScreen] bounds]];
    me.overlay.userInteractionEnabled = false;
    me.overlay.editable = false;
    me.overlay.backgroundColor = [UIColor clearColor];
    me.overlay.textColor = [UIColor whiteColor];
    me.overlay.font = [UIFont fontWithName:@"Menlo" size:8.f];
    [me addSubview:me.overlay];

    // Cast to 'id' because the compiler doesn't think this class is actually
    //  an EAGLView (technically this method only becomes part of the EAGLView
    //  class at runtime).
    return (id)me;
}

- (bool)presentFramebuffer {
    auto me = (__bridge EAGLView *)orig();

    if(Debug::needsUpdate()) {
        // Display log stuff.
        NSMutableString *addition = [NSMutableString string];
        for(const std::string &s : Debug::logStrings) {
            [addition appendFormat:@"%s\n", s.c_str()];
        }

        me.overlay.text = [me.overlay.text stringByAppendingString:addition];
        Debug::logStrings.clear();

        if(me.overlay.text.length > 0) {
            NSRange bottom = NSMakeRange(me.overlay.text.length - 1, 1);
            [me.overlay scrollRangeToVisible:bottom];
        }
    }

    return bool(orig());
}

@end
#endif