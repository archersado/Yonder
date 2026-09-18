#include <ApplicationServices/ApplicationServices.h>
#include <libproc.h>
#include <math.h>
#include <pthread.h>
#include <stdatomic.h>
#include <stdint.h>
#include <stdlib.h>
#include <unistd.h>

typedef struct {
  pid_t pid; uint32_t window_id; uint64_t start_sec, start_usec;
  AXUIElementRef app, target; CFStringRef title;
} YondaWorkRef;

static int process_start(pid_t pid, uint64_t *sec, uint64_t *usec) {
  struct proc_bsdinfo data = {0};
  if (proc_pidinfo(pid, PROC_PIDTBSDINFO, 0, &data, sizeof(data)) != sizeof(data) || data.pbi_start_tvsec == 0) return 0;
  *sec=data.pbi_start_tvsec; *usec=data.pbi_start_tvusec; return 1;
}

static int number(CFDictionaryRef item, CFStringRef key, int64_t *value) {
  CFNumberRef result=CFDictionaryGetValue(item,key);
  return result && CFGetTypeID(result)==CFNumberGetTypeID() && CFNumberGetValue(result,kCFNumberSInt64Type,value);
}

static int window_info(pid_t pid, uint32_t window_id, CFStringRef *title, CGRect *bounds) {
  CFArrayRef list=CGWindowListCopyWindowInfo(kCGWindowListOptionAll,kCGNullWindowID); if(!list)return 0;
  int found=0;
  for(CFIndex i=0;i<CFArrayGetCount(list);i++) {
    CFDictionaryRef item=CFArrayGetValueAtIndex(list,i); int64_t owner=0,identifier=0,layer=1;
    if(!number(item,kCGWindowOwnerPID,&owner)||!number(item,kCGWindowNumber,&identifier)||!number(item,kCGWindowLayer,&layer)||owner!=pid||identifier!=window_id||layer!=0)continue;
    CFStringRef name=CFDictionaryGetValue(item,kCGWindowName); CFDictionaryRef raw=CFDictionaryGetValue(item,kCGWindowBounds);
    if(name&&CFGetTypeID(name)==CFStringGetTypeID()&&CFStringGetLength(name)>0&&raw&&CGRectMakeWithDictionaryRepresentation(raw,bounds)) {if(title)*title=CFRetain(name);found=1;} break;
  }
  CFRelease(list); return found;
}

static CFTypeRef attribute(AXUIElementRef element, CFStringRef name) {
  CFTypeRef value=NULL; return AXUIElementCopyAttributeValue(element,name,&value)==kAXErrorSuccess?value:NULL;
}

static int focused_bounds(AXUIElementRef window,CGRect *bounds) {
  AXValueRef position=(AXValueRef)attribute(window,kAXPositionAttribute),size=(AXValueRef)attribute(window,kAXSizeAttribute);CGPoint point;CGSize dimensions;
  int valid=position&&size&&CFGetTypeID(position)==AXValueGetTypeID()&&CFGetTypeID(size)==AXValueGetTypeID()&&AXValueGetValue(position,kAXValueCGPointType,&point)&&AXValueGetValue(size,kAXValueCGSizeType,&dimensions);
  if(position)CFRelease(position);if(size)CFRelease(size);if(valid)*bounds=(CGRect){point,dimensions};return valid;
}

static int ax_frame(AXUIElementRef element, CGRect *frame) {
  AXValueRef p=(AXValueRef)attribute(element,kAXPositionAttribute),s=(AXValueRef)attribute(element,kAXSizeAttribute);
  CGPoint point=CGPointZero;CGSize size=CGSizeZero;int ok=p&&s&&CFGetTypeID(p)==AXValueGetTypeID()&&CFGetTypeID(s)==AXValueGetTypeID()&&AXValueGetValue(p,kAXValueCGPointType,&point)&&AXValueGetValue(s,kAXValueCGSizeType,&size);
  if(p)CFRelease(p);if(s)CFRelease(s);if(ok)*frame=(CGRect){point,size};return ok;
}

static int same(CGRect a, CGRect b) {return fabs(a.origin.x-b.origin.x)<1&&fabs(a.origin.y-b.origin.y)<1&&fabs(a.size.width-b.size.width)<1&&fabs(a.size.height-b.size.height)<1;}

static int mapping(AXUIElementRef app, CFStringRef title, CGRect bounds, AXUIElementRef retained, int *retained_present) {
  CFArrayRef windows=(CFArrayRef)attribute(app,kAXWindowsAttribute);if(!windows||CFGetTypeID(windows)!=CFArrayGetTypeID()){if(windows)CFRelease(windows);return 0;}
  int matches=0;*retained_present=0;
  for(CFIndex i=0;i<CFArrayGetCount(windows);i++){
    AXUIElementRef item=(AXUIElementRef)CFArrayGetValueAtIndex(windows,i);if(retained&&CFEqual(item,retained))*retained_present=1;
    CFStringRef item_title=(CFStringRef)attribute(item,kAXTitleAttribute);CGRect frame;
    if(item_title&&CFGetTypeID(item_title)==CFStringGetTypeID()&&CFEqual(item_title,title)&&ax_frame(item,&frame)&&same(frame,bounds))matches++;
    if(item_title)CFRelease(item_title);
  }
  CFRelease(windows);return matches;
}

int yonda_work_ref_capture(int32_t pid, uint32_t window_id, void **output, uint64_t *sec, uint64_t *usec) {
  if(!output||!sec||!usec||pid<=0||window_id==0||!AXIsProcessTrusted())return 1;
  uint64_t start_sec=0,start_usec=0;if(!process_start(pid,&start_sec,&start_usec))return 2;
  CFStringRef title=NULL;CGRect bounds;if(!window_info(pid,window_id,&title,&bounds))return 3;
  AXUIElementRef app=AXUIElementCreateApplication(pid),target=NULL;double end=CFAbsoluteTimeGetCurrent()+2;
  do {
    CFArrayRef windows=(CFArrayRef)attribute(app,kAXWindowsAttribute);int matches=0;
    if(windows&&CFGetTypeID(windows)==CFArrayGetTypeID())for(CFIndex i=0;i<CFArrayGetCount(windows);i++){
      AXUIElementRef item=(AXUIElementRef)CFArrayGetValueAtIndex(windows,i);CFStringRef item_title=(CFStringRef)attribute(item,kAXTitleAttribute);CGRect frame;
      if(item_title&&CFGetTypeID(item_title)==CFStringGetTypeID()&&CFEqual(item_title,title)&&ax_frame(item,&frame)&&same(frame,bounds)){matches++;target=item;}
      if(item_title)CFRelease(item_title);
    }
    if(matches==1){CFRetain(target);if(windows)CFRelease(windows);break;} target=NULL;if(windows)CFRelease(windows);usleep(50000);
  } while(CFAbsoluteTimeGetCurrent()<end);
  if(!target){CFRelease(title);CFRelease(app);return 4;}
  YondaWorkRef *reference=calloc(1,sizeof(*reference));if(!reference){CFRelease(target);CFRelease(title);CFRelease(app);return 8;}
  reference->pid=pid;reference->window_id=window_id;reference->start_sec=start_sec;reference->start_usec=start_usec;reference->app=app;reference->target=target;reference->title=title;
  *output=reference;*sec=start_sec;*usec=start_usec;return 0;
}

int yonda_work_ref_focus(void *raw) {
  YondaWorkRef *ref=raw;if(!ref)return 8;if(!AXIsProcessTrusted())return 1;
  uint64_t sec=0,usec=0;if(!process_start(ref->pid,&sec,&usec)||sec!=ref->start_sec||usec!=ref->start_usec)return 2;
  CFStringRef title=NULL;CGRect before;if(!window_info(ref->pid,ref->window_id,&title,&before))return 3;
  int present=0,matches=mapping(ref->app,title,before,ref->target,&present);CFRelease(title);if(!present||matches!=1)return 4;
  CFBooleanRef minimized=(CFBooleanRef)attribute(ref->target,kAXMinimizedAttribute);int was_minimized=minimized&&CFGetTypeID(minimized)==CFBooleanGetTypeID()&&CFBooleanGetValue(minimized);if(minimized)CFRelease(minimized);
  if(was_minimized&&AXUIElementSetAttributeValue(ref->target,kAXMinimizedAttribute,kCFBooleanFalse)!=kAXErrorSuccess)return 5;
  if(AXUIElementSetAttributeValue(ref->target,kAXMainAttribute,kCFBooleanTrue)!=kAXErrorSuccess||AXUIElementPerformAction(ref->target,kAXRaiseAction)!=kAXErrorSuccess||AXUIElementSetAttributeValue(ref->app,kAXFrontmostAttribute,kCFBooleanTrue)!=kAXErrorSuccess)return 5;
  double end=CFAbsoluteTimeGetCurrent()+3;int focused=0;
  do {
    CFBooleanRef front=(CFBooleanRef)attribute(ref->app,kAXFrontmostAttribute),mini=(CFBooleanRef)attribute(ref->target,kAXMinimizedAttribute);AXUIElementRef current=(AXUIElementRef)attribute(ref->app,kAXFocusedWindowAttribute);
    focused=front&&CFBooleanGetValue(front)&&current&&CFEqual(current,ref->target)&&mini&&!CFBooleanGetValue(mini);
    if(front)CFRelease(front);if(mini)CFRelease(mini);if(current)CFRelease(current);if(focused)break;usleep(50000);
  } while(CFAbsoluteTimeGetCurrent()<end);
  if(!focused)return 6;CGRect after;if(!window_info(ref->pid,ref->window_id,NULL,&after))return 3;return same(before,after)?0:7;
}

void yonda_work_ref_release(void *raw) {YondaWorkRef *ref=raw;if(!ref)return;CFRelease(ref->app);CFRelease(ref->target);CFRelease(ref->title);free(ref);}

int yonda_frontmost_work_target(int32_t self_pid,uint32_t *pid,uint32_t *window_id) {
  if(!pid||!window_id||!AXIsProcessTrusted())return 1;
  AXUIElementRef system=AXUIElementCreateSystemWide(),app=(AXUIElementRef)attribute(system,kAXFocusedApplicationAttribute),window=app?(AXUIElementRef)attribute(app,kAXFocusedWindowAttribute):NULL;pid_t focused_pid=0;CGRect focused_bounds_value=CGRectZero;
  if(app)AXUIElementGetPid(app,&focused_pid);int has_focused_bounds=window&&focused_bounds(window,&focused_bounds_value);CFRelease(system);
  CFArrayRef list=CGWindowListCopyWindowInfo(kCGWindowListOptionOnScreenOnly|kCGWindowListExcludeDesktopElements,kCGNullWindowID);if(!list){if(window)CFRelease(window);if(app)CFRelease(app);return 2;}
  int found=0;
  if(focused_pid>0&&focused_pid!=self_pid&&has_focused_bounds)for(CFIndex i=0;i<CFArrayGetCount(list);i++){
    CFDictionaryRef item=CFArrayGetValueAtIndex(list,i);int64_t owner=0,identifier=0,layer=1;CGRect bounds=CGRectZero;CFDictionaryRef raw=CFDictionaryGetValue(item,kCGWindowBounds);
    if(number(item,kCGWindowOwnerPID,&owner)&&number(item,kCGWindowNumber,&identifier)&&number(item,kCGWindowLayer,&layer)&&owner==focused_pid&&identifier>0&&layer==0&&raw&&CGRectMakeWithDictionaryRepresentation(raw,&bounds)&&same(bounds,focused_bounds_value)){*pid=(uint32_t)owner;*window_id=(uint32_t)identifier;found=1;break;}
  }
  if(focused_pid>0&&focused_pid!=self_pid&&!found)for(CFIndex i=0;i<CFArrayGetCount(list);i++){
    CFDictionaryRef item=CFArrayGetValueAtIndex(list,i);int64_t owner=0,identifier=0,layer=1;
    if(number(item,kCGWindowOwnerPID,&owner)&&number(item,kCGWindowNumber,&identifier)&&number(item,kCGWindowLayer,&layer)&&owner==focused_pid&&identifier>0&&layer==0){*pid=(uint32_t)owner;*window_id=(uint32_t)identifier;found=1;break;}
  }
  for(CFIndex i=0;i<CFArrayGetCount(list);i++){
    if(found)break;
    CFDictionaryRef item=CFArrayGetValueAtIndex(list,i);int64_t owner=0,identifier=0,layer=1;CGRect bounds=CGRectZero;
    CFDictionaryRef raw=CFDictionaryGetValue(item,kCGWindowBounds);
    if(number(item,kCGWindowOwnerPID,&owner)&&number(item,kCGWindowNumber,&identifier)&&number(item,kCGWindowLayer,&layer)&&owner>0&&owner!=self_pid&&identifier>0&&layer==0&&raw&&CGRectMakeWithDictionaryRepresentation(raw,&bounds)&&bounds.size.width>1&&bounds.size.height>1){*pid=(uint32_t)owner;*window_id=(uint32_t)identifier;found=1;break;}
  }
  if(window)CFRelease(window);if(app)CFRelease(app);CFRelease(list);return found?0:3;
}

static _Atomic uint64_t hid_generation=0;static _Atomic int hid_monitor_ready=0;static pthread_once_t hid_monitor_once=PTHREAD_ONCE_INIT;
static CGEventRef hid_event(CGEventTapProxy proxy,CGEventType type,CGEventRef event,void *context){(void)proxy;(void)type;(void)context;if(event&&CGEventGetIntegerValueField(event,kCGEventSourceUnixProcessID)==0)atomic_fetch_add(&hid_generation,1);return event;}
static void *hid_monitor(void *unused){(void)unused;CGEventMask mask=CGEventMaskBit(kCGEventKeyDown)|CGEventMaskBit(kCGEventMouseMoved)|CGEventMaskBit(kCGEventLeftMouseDown)|CGEventMaskBit(kCGEventRightMouseDown)|CGEventMaskBit(kCGEventOtherMouseDown)|CGEventMaskBit(kCGEventScrollWheel);CFMachPortRef tap=CGEventTapCreate(kCGHIDEventTap,kCGHeadInsertEventTap,kCGEventTapOptionListenOnly,mask,hid_event,NULL);if(!tap){atomic_store(&hid_monitor_ready,-1);return NULL;}CFRunLoopSourceRef source=CFMachPortCreateRunLoopSource(kCFAllocatorDefault,tap,0);CFRunLoopAddSource(CFRunLoopGetCurrent(),source,kCFRunLoopCommonModes);CGEventTapEnable(tap,true);atomic_store(&hid_monitor_ready,1);CFRunLoopRun();CFRelease(source);CFRelease(tap);return NULL;}
static void start_hid_monitor(void){pthread_t thread;if(pthread_create(&thread,NULL,hid_monitor,NULL)==0)pthread_detach(thread);else atomic_store(&hid_monitor_ready,-1);}
int yonda_hid_generation(uint64_t *value){if(!value)return 0;pthread_once(&hid_monitor_once,start_hid_monitor);for(int i=0;i<100&&atomic_load(&hid_monitor_ready)==0;i++)usleep(10000);if(atomic_load(&hid_monitor_ready)!=1)return 0;*value=atomic_load(&hid_generation);return 1;}

int yonda_accessibility_trusted(void) {return AXIsProcessTrusted()?1:0;}
