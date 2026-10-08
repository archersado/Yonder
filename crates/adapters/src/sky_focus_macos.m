#import <AppKit/AppKit.h>
#import <ApplicationServices/ApplicationServices.h>
#include <math.h>
#include <stdint.h>

extern int yonda_work_target_focus(int32_t pid, uint32_t window_id);

static BOOL number(NSDictionary *item, CFStringRef key, int64_t *value) {
  id raw=item[(__bridge NSString *)key];
  if(![raw isKindOfClass:NSNumber.class])return NO;
  *value=[raw longLongValue];return YES;
}

/* bundle id 必须来自 Sky list_apps 的 canonical id，不接受 Agent 提交 PID/window。 */
int yonda_activate_window_for_app(const char *raw_bundle_id) {
  @autoreleasepool {
    if(!raw_bundle_id)return 8;
    NSString *bundle=[NSString stringWithUTF8String:raw_bundle_id];
    if(!bundle||bundle.length==0)return 8;
    NSArray<NSRunningApplication *> *listed=[NSRunningApplication runningApplicationsWithBundleIdentifier:bundle];
    NSMutableArray<NSRunningApplication *> *running=[NSMutableArray array];
    for(NSRunningApplication *app in listed)if(!app.terminated&&app.processIdentifier>0)[running addObject:app];
    if(running.count!=1)return 4;
    NSRunningApplication *application=running.firstObject;
    pid_t pid=application.processIdentifier;
    // 部分混合应用在后台时 AXWindows 为空。先发出公开的应用级激活请求，
    // 再把 AXFrontmost 设为 true，使这类应用真正发布 AXWindows；随后仍由
    // yonda_work_target_focus 对精确窗口执行 AX raise 并验证后置条件。
    // 两个调用的返回值都只代表请求已接收，不能替代后面的窗口验证。
    [application activateWithOptions:NSApplicationActivateAllWindows];
    AXUIElementRef ax_application=AXUIElementCreateApplication(pid);
    if(ax_application){
      AXUIElementSetAttributeValue(ax_application,kAXFrontmostAttribute,kCFBooleanTrue);
      CFRelease(ax_application);
    }
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

/*
 * Sky 0.7.1 的应用截图使用 backing 像素，动作 API 使用窗口逻辑点。
 * 这里只接受 Application 已经从前台事实取得的 pid/window_id；先验证
 * WindowServer 所有者，再取窗口重叠面积最大的显示器的系统倍率。
 * 0 表示无法建立可信映射，调用方必须拒绝坐标动作而不能猜测倍率。
 */
double yonda_backing_scale_for_window(int32_t expected_pid, uint32_t window_id) {
  @autoreleasepool {
    if(expected_pid<=0||window_id==0)return 0;
    CFArrayRef raw=CGWindowListCopyWindowInfo(kCGWindowListOptionIncludingWindow,window_id);
    if(!raw)return 0;
    NSArray *windows=CFBridgingRelease(raw);
    CGRect window_bounds=CGRectZero;BOOL found=NO;
    for(NSDictionary *item in windows){
      int64_t owner=0,identifier=0;
      if(!number(item,kCGWindowOwnerPID,&owner)||!number(item,kCGWindowNumber,&identifier)
          ||owner!=expected_pid||identifier!=window_id)continue;
      NSDictionary *encoded=item[(__bridge NSString *)kCGWindowBounds];
      if(![encoded isKindOfClass:NSDictionary.class]
          ||!CGRectMakeWithDictionaryRepresentation((__bridge CFDictionaryRef)encoded,&window_bounds)
          ||window_bounds.size.width<=1||window_bounds.size.height<=1)continue;
      found=YES;break;
    }
    if(!found)return 0;

    uint32_t count=0;
    if(CGGetActiveDisplayList(0,NULL,&count)!=kCGErrorSuccess||count==0||count>64)return 0;
    CGDirectDisplayID displays[64];
    if(CGGetActiveDisplayList(count,displays,&count)!=kCGErrorSuccess)return 0;
    CGDirectDisplayID selected=0;double largest=0;
    for(uint32_t index=0;index<count;index++){
      CGRect intersection=CGRectIntersection(window_bounds,CGDisplayBounds(displays[index]));
      if(CGRectIsNull(intersection)||CGRectIsEmpty(intersection))continue;
      double area=intersection.size.width*intersection.size.height;
      if(area>largest){largest=area;selected=displays[index];}
    }
    if(selected==0)return 0;
    for(NSScreen *screen in NSScreen.screens){
      NSNumber *identifier=screen.deviceDescription[@"NSScreenNumber"];
      if(![identifier isKindOfClass:NSNumber.class]||identifier.unsignedIntValue!=selected)continue;
      double scale=screen.backingScaleFactor;
      return isfinite(scale)&&scale>=1.0&&scale<=4.0?scale:0;
    }
    return 0;
  }
}
