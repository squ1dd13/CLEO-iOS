//
// Created by squ1dd13 on 29/01/2021.
//

#include "Menu.h"

#include <vector>
#include <UIKit/UIKit.h>
#include <cmath>

#include "Manager.h"
#include "Logging.h"
#include "bridge/Memory.h"

std::vector<id> blocks;
UIView *menu;

void SetupIfNeeded() {
    // We do nothing if the menu exists or if the scripts haven't been loaded yet.
    if (menu || !Scripts::Manager::Initialized()) {
        return;
    }

    // We have to add the menu to the game's window because the EAGLView draws over the top
    //  of its subviews, and sublayers don't work well either.
    // FIXME: keyWindow is deprecated. Works for now because SA doesn't use scenes (which
    //  probably won't change).
    UIWindow *window = [[UIApplication sharedApplication] keyWindow];

    double menuWidth = std::round(window.bounds.size.width * 0.7);
    double menuHeight = std::round(window.bounds.size.height * 0.7);

    CGRect rect {
        std::round((window.bounds.size.width - menuWidth) / 2),
        std::round((window.bounds.size.height - menuHeight) / 2),
        menuWidth,
        menuHeight };

    menu = [[UIView alloc] initWithFrame:rect];
    menu.backgroundColor = [UIColor colorWithWhite:.0f alpha:.95f];

    CGRect titleFrame {
        0,
        0,
        rect.size.width,
        std::round(0.2f * rect.size.height) };
    UILabel *titleLabel = [[UILabel alloc] initWithFrame:titleFrame];

    titleLabel.text = @"Scripts";
    titleLabel.font = [UIFont fontWithName:@"PricedownGTAVInt" size:35.f];
    titleLabel.textColor = [UIColor whiteColor];
    titleLabel.adjustsFontSizeToFitWidth = true;
    titleLabel.textAlignment = NSTextAlignmentCenter;

    CGRect scrollViewFrame {
        0,
        std::round(0.2f * rect.size.height),
        rect.size.width,
        std::round(rect.size.height * 0.8f) };

    UIScrollView *scrollView = [[UIScrollView alloc] initWithFrame:scrollViewFrame];

    scrollView.bounces = false;

    double buttonHeight = std::round(0.15f * menuHeight);

    std::set<std::string> &invoked = Scripts::Manager::InvokedScripts();
    scrollView.contentSize = {
        rect.size.width,
        buttonHeight * invoked.size()
    };

    size_t i = 0;
    for (auto &scriptName : invoked) {
        CGRect buttonFrame {
            0,
            i++ * buttonHeight,
            rect.size.width,
            buttonHeight };

        UIButton *btn = [[UIButton alloc] initWithFrame:buttonFrame];

        [[btn titleLabel] setFont:[UIFont fontWithName:@"PricedownGTAVInt" size:20.f]];
        [btn setTitle:[NSString stringWithUTF8String:scriptName.c_str()] forState:UIControlStateNormal];

        id block = [^{
            LogImportant("Load %s", scriptName.c_str());
            Scripts::Manager::Invoke(scriptName);
            Scripts::Menu::Hide();
        } copy];

        blocks.push_back(block);

        [btn addTarget:block action:@selector(invoke) forControlEvents:UIControlEventTouchUpInside];
        [scrollView addSubview:btn];
    }

    [menu addSubview:titleLabel];
    [menu addSubview:scrollView];
    [window addSubview:menu];
}

void Scripts::Menu::Show() {
    SetupIfNeeded();

    (*Memory::Slid<float *>(0x1007d3b18)) = 0.01f;
    menu.hidden = false;
}

void Scripts::Menu::Hide() {
    if (Visible()) {
        *Memory::Slid<float *>(0x1007d3b18) = 1.f;
        menu.hidden = true;
    }
}

bool Scripts::Menu::Visible() {
    return menu && !menu.hidden;
}
