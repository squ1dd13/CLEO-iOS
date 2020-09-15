#include "Util/Debug.hpp"
std::vector<std::string> Debug::logStrings;

// This is a *very* primitive system, so if the game runs too long it'll run out of memory.
@import UIKit;

@interface EAGLView : UIView
@property(nonatomic, strong) UITextView *overlay;
@end

%group ScreenLog
%hook EAGLView
%property(nonatomic, strong) UITextView *overlay;

- (id)initWithCoder:(id)coder {

    EAGLView *orig = %orig;
#ifdef SHOW_DEBUG_OVERLAY
    orig.overlay = [[UITextView alloc] initWithFrame:[[UIScreen mainScreen] bounds]];
    orig.overlay.userInteractionEnabled = false;
    orig.overlay.editable = false;
    orig.overlay.backgroundColor = [UIColor clearColor];
    orig.overlay.textColor = [UIColor whiteColor];
    orig.overlay.font = [UIFont fontWithName:@"Menlo" size:8.f];
    [orig addSubview:orig.overlay];
#endif
    return orig;
}

// Called every frame.
- (bool)presentFramebuffer {
#ifdef SHOW_DEBUG_OVERLAY
    if(Debug::needsUpdate()) {
        // Display log stuff.
        NSMutableString *addition = [NSMutableString string];
        for(const std::string &s : Debug::logStrings) {
            [addition appendFormat:@"%s\n", s.c_str()];
        }

        self.overlay.text = [self.overlay.text stringByAppendingString:addition];
        Debug::logStrings.clear();

        if(self.overlay.text.length > 0) {
            NSRange bottom = NSMakeRange(self.overlay.text.length - 1, 1);
            [self.overlay scrollRangeToVisible:bottom];
        }
    }
#endif

    return %orig;
}

%end 
%end

%ctor {
#ifdef SHOW_DEBUG_OVERLAY
    %init(ScreenLog);
#endif
}