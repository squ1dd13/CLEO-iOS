#include "Main.cpp"
@import UIKit;

@interface EAGLView : UIView
@end

%hook EAGLView

-(void)createFramebuffer {
    %orig;

    float size[2] { float(self.bounds.size.width * self.layer.contentsScale), float(self.bounds.size.height * self.layer.contentsScale) };
    Touch::setViewportSize(size[0], size[1]);
    Debug::logf("contentsScale = %f", self.layer.contentsScale);
    Debug::logf("VP size is { %f, %f }", size[0], size[1]);
}

%end

@interface LegalSplash : UIViewController
@end

%hook LegalSplash 

// We hook here to display the "CSiOS" splash screen.
-(void)viewDidLoad {
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
    
    [UIView animateWithDuration:0.2 delay:1.0 options:0 animations:^{
        Debug::logf("Showing splash screen");
        customLabel.alpha = 0.0f;
    } completion:^(BOOL finished) {
        customLabel.hidden = true;
        [customLabel removeFromSuperview];
    }];

    %orig;
    [self.view addSubview:customLabel];
}

%end

%ctor {
    inject();
}

%dtor {
    cleanUp();
}