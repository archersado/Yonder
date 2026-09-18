#import <ApplicationServices/ApplicationServices.h>
#import <Foundation/Foundation.h>
#import <ImageIO/ImageIO.h>
#import <ScreenCaptureKit/ScreenCaptureKit.h>

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
    NSMutableData *png = [NSMutableData data];
    CGImageDestinationRef destination = CGImageDestinationCreateWithData((__bridge CFMutableDataRef)png, CFSTR("public.png"), 1, NULL);
    if (!destination) { CGImageRelease(image); return NULL; }
    CGImageDestinationAddImage(destination, image, NULL);
    BOOL written = CGImageDestinationFinalize(destination);
    CFRelease(destination); CGImageRelease(image);
    if (!written || png.length == 0) return NULL;
    NSString *encoded = [png base64EncodedStringWithOptions:0];
    return encoded ? strdup(encoded.UTF8String) : NULL;
}

void yonda_region_free(void *pointer) { free(pointer); }
