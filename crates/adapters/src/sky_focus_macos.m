#import <AppKit/AppKit.h>
#import <ApplicationServices/ApplicationServices.h>
#include <stdint.h>

extern int yonda_work_target_focus(int32_t pid, uint32_t window_id);

static BOOL number(NSDictionary *item, CFStringRef key, int64_t *value) {
  id raw=item[(__bridge NSString *)key];
  if(![raw isKindOfClass:NSNumber.class])return NO;
  *value=[raw longLongValue];return YES;
}

/* bundle id 必须来自 Sky list_apps 的 canonical id，不接受 Agent 提交 PID/window。 */
int yonda_sky_app_focus(const char *raw_bundle_id) {
  @autoreleasepool {
    if(!raw_bundle_id)return 8;
    NSString *bundle=[NSString stringWithUTF8String:raw_bundle_id];
    if(!bundle||bundle.length==0)return 8;
    NSArray<NSRunningApplication *> *listed=[NSRunningApplication runningApplicationsWithBundleIdentifier:bundle];
    NSMutableArray<NSRunningApplication *> *running=[NSMutableArray array];
    for(NSRunningApplication *app in listed)if(!app.terminated&&app.processIdentifier>0)[running addObject:app];
    if(running.count!=1)return 4;
    pid_t pid=running.firstObject.processIdentifier;
    CFArrayRef raw=CGWindowListCopyWindowInfo(kCGWindowListOptionAll,kCGNullWindowID);
    if(!raw)return 3;
    NSArray *windows=CFBridgingRelease(raw);uint32_t selected=0;double largest=0;
    for(NSDictionary *item in windows){
      int64_t owner=0,identifier=0,layer=1;
      if(!number(item,kCGWindowOwnerPID,&owner)||!number(item,kCGWindowNumber,&identifier)||!number(item,kCGWindowLayer,&layer)||owner!=pid||identifier<=0||identifier>UINT32_MAX||layer!=0)continue;
      CGRect bounds=CGRectZero;NSDictionary *encoded=item[(__bridge NSString *)kCGWindowBounds];
      if(![encoded isKindOfClass:NSDictionary.class]||!CGRectMakeWithDictionaryRepresentation((__bridge CFDictionaryRef)encoded,&bounds))continue;
      double area=bounds.size.width*bounds.size.height;if(bounds.size.width<=1||bounds.size.height<=1||area<=largest)continue;
      largest=area;selected=(uint32_t)identifier;
    }
    if(selected==0)return 3;
    return yonda_work_target_focus(pid,selected);
  }
}
