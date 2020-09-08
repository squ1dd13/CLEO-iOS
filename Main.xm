#include "Main.cpp"
@import UIKit;

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