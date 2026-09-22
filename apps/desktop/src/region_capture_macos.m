#import <ApplicationServices/ApplicationServices.h>
#import <AppKit/AppKit.h>
#import <Foundation/Foundation.h>
#import <ImageIO/ImageIO.h>
#import <ScreenCaptureKit/ScreenCaptureKit.h>

static id yonda_region_activation_observer = nil;
static id yonda_region_display_observer = nil;

static void yonda_region_on_main(void (^block)(void)) {
    if (NSThread.isMainThread) block();
    else dispatch_sync(dispatch_get_main_queue(), block);
}

void yonda_region_watch_application_switch(void (*callback)(void)) {
    yonda_region_on_main(^{
        if (yonda_region_activation_observer) return;
        yonda_region_activation_observer = [NSWorkspace.sharedWorkspace.notificationCenter
            addObserverForName:NSWorkspaceDidActivateApplicationNotification object:nil queue:nil
            usingBlock:^(NSNotification *notification) {
                NSRunningApplication *application = notification.userInfo[NSWorkspaceApplicationKey];
                if (![application.bundleIdentifier isEqualToString:NSBundle.mainBundle.bundleIdentifier]) callback();
            }];
    });
}

void yonda_region_stop_application_switch_watch(void) {
    @synchronized(NSWorkspace.sharedWorkspace) {
        if (!yonda_region_activation_observer) return;
        [NSWorkspace.sharedWorkspace.notificationCenter removeObserver:yonda_region_activation_observer];
        yonda_region_activation_observer = nil;
    }
}

void yonda_region_watch_display_change(void (*callback)(void)) {
    yonda_region_on_main(^{
        if (yonda_region_display_observer) return;
        yonda_region_display_observer = [[NSNotificationCenter defaultCenter]
            addObserverForName:NSApplicationDidChangeScreenParametersNotification object:nil queue:nil
            usingBlock:^(NSNotification *notification) { (void)notification; callback(); }];
    });
}

void yonda_region_stop_display_change_watch(void) {
    @synchronized([NSNotificationCenter defaultCenter]) {
        if (!yonda_region_display_observer) return;
        [[NSNotificationCenter defaultCenter] removeObserver:yonda_region_display_observer];
        yonda_region_display_observer = nil;
    }
}

char *yonda_region_capture(int x, int y, int width, int height) {
    if (width < 1 || height < 1 || !CGPreflightScreenCaptureAccess()) return NULL;
    if (@available(macOS 15.2, *)) {} else return NULL;
    __block CGImageRef image = NULL;
    dispatch_semaphore_t completed = dispatch_semaphore_create(0);
    [SCScreenshotManager captureImageInRect:CGRectMake(x, y, width, height) completionHandler:^(CGImageRef captured, NSError *error) {
        if (captured && !error) image = CGImageRetain(captured);
        dispatch_semaphore_signal(completed);
    }];
    if (dispatch_semaphore_wait(completed, dispatch_time(DISPATCH_TIME_NOW, 5 * NSEC_PER_SEC)) != 0) return NULL;
    if (!image) return NULL;
    // Normalize to 8-bit sRGB so the WebKit preview can always decode it.
    size_t imageWidth = CGImageGetWidth(image), imageHeight = CGImageGetHeight(image);
    CGColorSpaceRef colorSpace = CGColorSpaceCreateWithName(kCGColorSpaceSRGB);
    CGContextRef context = colorSpace ? CGBitmapContextCreate(NULL, imageWidth, imageHeight, 8, imageWidth * 4,
        colorSpace, kCGImageAlphaPremultipliedLast | kCGBitmapByteOrder32Big) : NULL;
    CGColorSpaceRelease(colorSpace);
    if (!context) { CGImageRelease(image); return NULL; }
    CGContextDrawImage(context, CGRectMake(0, 0, imageWidth, imageHeight), image);
    CGImageRef normalized = CGBitmapContextCreateImage(context);
    CGContextRelease(context); CGImageRelease(image);
    if (!normalized) return NULL;
    NSMutableData *png = [NSMutableData data];
    CGImageDestinationRef destination = CGImageDestinationCreateWithData((__bridge CFMutableDataRef)png, CFSTR("public.png"), 1, NULL);
    if (!destination) { CGImageRelease(normalized); return NULL; }
    CGImageDestinationAddImage(destination, normalized, NULL);
    BOOL written = CGImageDestinationFinalize(destination);
    CFRelease(destination); CGImageRelease(normalized);
    if (!written || png.length == 0) return NULL;
    NSString *encoded = [png base64EncodedStringWithOptions:0];
    return encoded ? strdup(encoded.UTF8String) : NULL;
}

void yonda_region_free(void *pointer) { free(pointer); }

char *yonda_region_source_application(void) {
    NSRunningApplication *application = NSWorkspace.sharedWorkspace.frontmostApplication;
    if ([application.bundleIdentifier isEqualToString:NSBundle.mainBundle.bundleIdentifier]) return NULL;
    NSString *name = application.localizedName;
    return name.length ? strdup(name.UTF8String) : NULL;
}
