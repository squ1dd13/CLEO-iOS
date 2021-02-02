#include <UIKit/UIKit.h>
#include <cmath>

#include "user/Touch.h"
#include "hook/ObjectiveC.h"
#include "Logging.h"
#include "scripts/Menu.h"

void ProcessTouches(UIView *view, NSSet *touches, Touch::Type type) {
    if ([touches count] == 0) {
        return;
    }

    Touch::BeginUpdates();
    for (UITouch *touch in touches) {
        auto oldPos = [touch previousLocationInView:view];
        auto pos = [touch locationInView:view];

        auto oldX = float(oldPos.x * view.layer.contentsScale);
        auto oldY = float(oldPos.y * view.layer.contentsScale);

        auto x = float(pos.x * view.layer.contentsScale);
        auto y = float(pos.y * view.layer.contentsScale);

        double time = [touch timestamp];

        Touch(oldX, oldY, x, y, type, time).Handle();
    }
}

struct TouchInfo {
    float x, y;
    double time;
};

// Underscore to prevent name collisions with some OpenCL thing.
int sign_(float num) {
    return num < 0 ? -1 : 1;
}

bool IsMenuSwipe(const TouchInfo &start, const TouchInfo &end) {
    // There should probably be an option for controlling the speed and distance.
    // (Preferably combine them with a single "swipe sensitivity" value.)
    constexpr float requiredSpeed = 700;
    constexpr float requiredDist = 25;

    if (start.time <= 0) {
        // Starting touch isn't initialised.
        return false;
    }

    struct {
        float x, y;
        double time;
    } delta {
        end.x - start.x,
        end.y - start.y,
        end.time - start.time
    };

    float distance = std::sqrt(delta.x * delta.x + delta.y * delta.y);
    if (distance < requiredDist) {
        // Didn't meet the minimum distance.
        return false;
    }

    double speed = distance / delta.time;
    if (speed < requiredSpeed) {
        // Didn't meet the minimum speed.
        return false;
    }

    // Normalise the deltas so we can determine the direction.
    int normX = std::abs(delta.x / distance) > 0.4 ? sign_(delta.x) : 0;
    int normY = std::abs(delta.y / distance) > 0.4 ? sign_(delta.y) : 0;

    // Return true only for downwards swipes.
    return normX == 0 && normY == 1;
}

// clang-format doesn't like our Objective-C hook macros, so we have to disable formatting.
/* clang-format off */

// @hookbase makes the hook class a subclass of the second argument.
// In this case, it means we can use 'self' as a UIView * and not an NSObject *.
@hookbase(EAGLView, UIView)

// TODO: %property (probably won't happen)
struct {
    TouchInfo touch;
} EAGLViewProperties;

- (void)touchesBegan:(NSSet *)touches withEvent:(UIEvent *)event {
    CGPoint pos = [[touches anyObject] locationInView:self];

    EAGLViewProperties.touch = {
        float(pos.x),
        float(pos.y),
        [[touches anyObject] timestamp]
    };

    ProcessTouches(self, touches, Touch::Type::Down);
}

- (void)touchesMoved:(NSSet *)touches withEvent:(UIEvent *)event {
    ProcessTouches(self, touches, Touch::Type::Moved);
}

- (void)touchesEnded:(NSSet *)touches withEvent:(UIEvent *)event {
    CGPoint pos = [[touches anyObject] locationInView:self];

    TouchInfo endTouch {
        float(pos.x),
        float(pos.y),
        [[touches anyObject] timestamp]
    };

    if (IsMenuSwipe(EAGLViewProperties.touch, endTouch)) {
        LogImportant("Activate menu!");
        Scripts::Menu::Show();
    } else {
        Scripts::Menu::Hide();
    }

    ProcessTouches(self, touches, Touch::Type::Up);
    EAGLViewProperties.touch.time = -1;
}

- (void)touchesCancelled:(NSSet *)touches withEvent:(UIEvent *)event {
    ProcessTouches(self, touches, Touch::Type::Up);
}

- (void)createFramebuffer {
    orig();

    float size[2] {
        float(self.bounds.size.width * self.layer.contentsScale),
        float(self.bounds.size.height * self.layer.contentsScale)
    };

    Touch::SetViewportSize(size[0], size[1]);
}

@end

// Splash screen
@hookbase(LegalSplash, UIViewController)

- (void)viewDidLoad {
    UILabel *customLabel = [[UILabel alloc] initWithFrame:self.view.bounds];
    customLabel.text = @"Zinc";
    customLabel.textColor = [UIColor whiteColor];
    customLabel.font = [UIFont fontWithName:@"PricedownGTAVInt" size:50.f];
    customLabel.textAlignment = NSTextAlignmentCenter;

    // If the user taps here they can actually skip our splash and the legal one.
    customLabel.userInteractionEnabled = false;

    customLabel.backgroundColor = [UIColor blackColor];

    customLabel.hidden = false;
    customLabel.alpha = 1.f;

    orig();

    // Start the legal text at 0 alpha.
    for (UIView *v in self.view.subviews) {
        v.alpha = 0.0f;
    }

    [self.view addSubview:customLabel];

    // Show "Zinc" for 1 second and then fade out over 0.5 seconds.
    [UIView animateWithDuration:0.5 delay:1 options:UIViewAnimationOptionCurveEaseIn animations:^{
        customLabel.alpha = 0.0f;

        // Scale up the "Zinc" text slightly as we fade it out.
        customLabel.transform = CGAffineTransformScale(customLabel.transform, 2, 2);
    } completion:^(BOOL finished) {
        customLabel.hidden = true;
        [customLabel removeFromSuperview];

        // Fade the legal text in.
        [UIView animateWithDuration:0.3 animations:^{
            for (UIView *v in self.view.subviews) {
                v.alpha = 1.0f;
            }
        }];
    }];
}

@end

/* clang-format on */