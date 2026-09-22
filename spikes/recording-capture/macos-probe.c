#include <ApplicationServices/ApplicationServices.h>
#import <AppKit/AppKit.h>
#include <Carbon/Carbon.h>
#include <CoreFoundation/CoreFoundation.h>
#include <IOKit/hid/IOHIDManager.h>
#include <IOKit/hid/IOHIDUsageTables.h>
#include <IOKit/hidsystem/IOHIDLib.h>
#include <mach/mach_time.h>
#include <stdint.h>
#include <sys/types.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

enum { CAPACITY = 64, YONDER_INJECTED_TAG = 0x594f4e44 };
typedef struct { CGEventType type; int64_t source_pid, source_tag; } Sample;
typedef struct { uint64_t timestamp; unsigned kind; } Mark;
typedef struct {
  Sample samples[CAPACITY]; Mark events[CAPACITY], hid[CAPACITY * 4];
  size_t count, event_count, hid_count; unsigned gaps, known_injected_rejected, excluded_secure_input, excluded_app; int recording, lease_active, excluded_pid;
} Recorder;

enum { KIND_KEY = 1, KIND_BUTTON = 2, KIND_SCROLL = 3 };
static unsigned event_kind(CGEventType type) {
  if (type == kCGEventKeyDown || type == kCGEventFlagsChanged) return KIND_KEY;
  if (type == kCGEventLeftMouseDown || type == kCGEventRightMouseDown) return KIND_BUTTON;
  return type == kCGEventScrollWheel ? KIND_SCROLL : 0;
}
static int frontmost_window_pid(void) {
  return (int)NSWorkspace.sharedWorkspace.frontmostApplication.processIdentifier;
}

static CGEventRef capture(CGEventTapProxy proxy, CGEventType type, CGEventRef event, void *context) {
  (void)proxy;
  Recorder *recorder = context;
  if (type == kCGEventTapDisabledByTimeout || type == kCGEventTapDisabledByUserInput) {
    recorder->gaps++; recorder->recording = 0; return event;
  }
  if (!recorder->recording || !recorder->lease_active) return event;
  if (recorder->excluded_pid > 0 && frontmost_window_pid() == recorder->excluded_pid) { recorder->excluded_app++; return event; }
  if (IsSecureEventInputEnabled()) { recorder->excluded_secure_input++; return event; }
  if (CGEventGetIntegerValueField(event, kCGEventSourceUserData) == YONDER_INJECTED_TAG) { recorder->known_injected_rejected++; return event; }
  if (recorder->count == CAPACITY || recorder->event_count == CAPACITY) { recorder->gaps++; recorder->recording = 0; return event; }
  recorder->samples[recorder->count++] = (Sample){type,
    CGEventGetIntegerValueField(event, kCGEventSourceUnixProcessID),
    CGEventGetIntegerValueField(event, kCGEventSourceUserData)};
  recorder->events[recorder->event_count++] = (Mark){CGEventGetTimestamp(event), event_kind(type)};
  return event;
}

static void hid_value(void *context, IOReturn result, void *sender, IOHIDValueRef value) {
  (void)sender;
  Recorder *recorder = context;
  if (!recorder->recording || !recorder->lease_active || result != kIOReturnSuccess || recorder->hid_count == CAPACITY * 4 || IOHIDValueGetIntegerValue(value) == 0) return;
  IOHIDElementRef element = IOHIDValueGetElement(value);
  uint32_t page = IOHIDElementGetUsagePage(element), usage = IOHIDElementGetUsage(element); unsigned kind = 0;
  if (page == kHIDPage_KeyboardOrKeypad) kind = KIND_KEY;
  else if (page == kHIDPage_Button) kind = KIND_BUTTON;
  else if (page == kHIDPage_GenericDesktop && usage == kHIDUsage_GD_Wheel) kind = KIND_SCROLL;
  if (kind) recorder->hid[recorder->hid_count++] = (Mark){IOHIDValueGetTimeStamp(value), kind};
}

static int user_candidate(Sample sample) { return sample.source_pid == 0 && sample.source_tag == 0; }
static int known_yonder_injection(Sample sample) { return sample.source_tag == YONDER_INJECTED_TAG; }

static int self_check(void) {
  Recorder recorder = {.recording = 1, .lease_active = 1};
  Sample user = {kCGEventLeftMouseDown, 0, 0}, injected = {kCGEventKeyDown, 42, 0}, replay = {kCGEventKeyDown, 42, YONDER_INJECTED_TAG};
  if (!user_candidate(user) || user_candidate(injected) || user_candidate(replay) || !known_yonder_injection(replay) || known_yonder_injection(user)) return 1;
  recorder.count = CAPACITY; recorder.gaps = 1; recorder.recording = 0;
  if (recorder.count != CAPACITY || recorder.gaps != 1 || recorder.recording) return 2;
  recorder.lease_active = 0;
  if (recorder.lease_active) return 3;
  puts("{\"controlled_session_boundary\":true,\"known_injected_rejected\":true,\"bounded_queue\":true,\"stop_boundary\":true,\"payload_recorded\":false,\"passed\":true}");
  return 0;
}

int main(int argc, char **argv) {
  if (argc == 2 && strcmp(argv[1], "--self-check") == 0) return self_check();
  int hid_mode = argc == 4 && strcmp(argv[1], "--hid-listen") == 0;
  int excluded_mode = argc == 4 && strcmp(argv[1], "--exclude-pid") == 0;
  int stop_check_mode = argc == 3 && strcmp(argv[1], "--stop-check") == 0;
  int controlled_mode = argc == 3 && (strcmp(argv[1], "--controlled") == 0 || strcmp(argv[1], "--outside") == 0);
  int known_injection_mode = argc == 3 && strcmp(argv[1], "--known-injection") == 0;
  if (argc != 2 && !hid_mode && !controlled_mode && !stop_check_mode && !excluded_mode && !known_injection_mode) { fputs("usage: macos-probe --self-check | seconds | --controlled seconds | --outside seconds | --stop-check seconds | --known-injection seconds | --exclude-pid PID seconds | --hid-listen seconds window_ms\n", stderr); return 2; }
  char *end = NULL; double seconds = strtod((controlled_mode || stop_check_mode || excluded_mode || known_injection_mode) ? argv[2 + excluded_mode] : argv[1], &end);
  double window_ms = 0;
  if (hid_mode) { seconds = strtod(argv[2], &end); if (!end || *end) return 2; window_ms = strtod(argv[3], &end); }
  if (!end || *end || seconds <= 0 || seconds > 30) return 2;
  Recorder recorder = {.recording = 1, .lease_active = known_injection_mode || !controlled_mode || strcmp(argv[1], "--controlled") == 0};
  if (excluded_mode) { recorder.excluded_pid = (int)strtol(argv[2], &end, 10); if (!end || *end || recorder.excluded_pid <= 0) return 2; }
  CGEventMask mask = CGEventMaskBit(kCGEventKeyDown) | CGEventMaskBit(kCGEventFlagsChanged) |
    CGEventMaskBit(kCGEventLeftMouseDown) | CGEventMaskBit(kCGEventRightMouseDown) | CGEventMaskBit(kCGEventScrollWheel);
  CFMachPortRef tap = CGEventTapCreate(kCGSessionEventTap, kCGHeadInsertEventTap, kCGEventTapOptionListenOnly, mask, capture, &recorder);
  if (!tap) { puts("{\"available\":false,\"passed\":false}"); return 3; }
  CFRunLoopSourceRef source = CFMachPortCreateRunLoopSource(kCFAllocatorDefault, tap, 0);
  CFRunLoopAddSource(CFRunLoopGetCurrent(), source, kCFRunLoopCommonModes);
  IOHIDManagerRef manager = NULL;
  IOHIDAccessType access = kIOHIDAccessTypeUnknown;
  if (hid_mode) {
    if (window_ms <= 0 || window_ms > 100) return 2;
    access = IOHIDCheckAccess(kIOHIDRequestTypeListenEvent);
    manager = IOHIDManagerCreate(kCFAllocatorDefault, kIOHIDOptionsTypeNone);
    IOHIDManagerSetDeviceMatching(manager, NULL);
    IOHIDManagerRegisterInputValueCallback(manager, hid_value, &recorder);
    IOHIDManagerScheduleWithRunLoop(manager, CFRunLoopGetCurrent(), kCFRunLoopCommonModes);
    if (IOHIDManagerOpen(manager, kIOHIDOptionsTypeNone) != kIOReturnSuccess) { CFRelease(manager); manager = NULL; }
  }
  CFAbsoluteTime started = CFAbsoluteTimeGetCurrent(), deadline = started + seconds;
  size_t stopped_count = 0;
  int injection_posted = 0;
  while (CFAbsoluteTimeGetCurrent() < deadline) {
    if (stop_check_mode && recorder.recording && CFAbsoluteTimeGetCurrent() >= started + seconds / 2) { recorder.recording = 0; stopped_count = recorder.count; }
    if (known_injection_mode && !injection_posted && CFAbsoluteTimeGetCurrent() >= started + 0.1) {
      CGEventSourceRef source = CGEventSourceCreate(kCGEventSourceStatePrivate);
      CGEventRef event = CGEventCreateScrollWheelEvent(source, kCGScrollEventUnitPixel, 2, 0, 0);
      if (source && event) {
        CGEventSetIntegerValueField(event, kCGEventSourceUserData, YONDER_INJECTED_TAG);
        CGEventPost(kCGHIDEventTap, event);
      }
      if (event) CFRelease(event);
      if (source) CFRelease(source);
      injection_posted = 1;
    }
    CFRunLoopRunInMode(kCFRunLoopDefaultMode, 0.05, false);
  }
  recorder.recording = 0;
  unsigned external = 0;
  for (size_t i = 0; i < recorder.count; i++) if (!user_candidate(recorder.samples[i])) external++;
  unsigned correlated = 0;
  double min_delta_ms = -1;
  if (hid_mode) {
    mach_timebase_info_data_t scale; mach_timebase_info(&scale);
    uint64_t tolerance = (uint64_t)(window_ms * 1000000.0 * scale.denom / scale.numer);
    for (size_t i = 0; i < recorder.event_count; i++) for (size_t j = 0; j < recorder.hid_count; j++) {
      uint64_t a = recorder.events[i].timestamp, b = recorder.hid[j].timestamp, delta = a > b ? a - b : b - a;
      if (recorder.events[i].kind == recorder.hid[j].kind) { double ms = (double)delta * scale.numer / scale.denom / 1000000.0; if (min_delta_ms < 0 || ms < min_delta_ms) min_delta_ms = ms; }
      if (recorder.events[i].kind == recorder.hid[j].kind && delta <= tolerance) { correlated++; break; }
    }
  }
  printf("{\"available\":true,\"controlled_session_inputs\":%zu,\"post_stop_inputs\":%zu,\"external_unknown\":%u,\"known_injected_rejected\":%u,\"excluded_secure_input\":%u,\"excluded_app\":%u,\"secure_input_active\":%s,\"lease_active\":%s,\"hid_access\":%d,\"hid_marks\":%zu,\"hardware_correlated\":%u,\"window_ms\":%.1f,\"min_matching_delta_ms\":%.3f,\"gaps\":%u,\"payload_recorded\":false}\n",
    recorder.count, stop_check_mode ? recorder.count - stopped_count : 0, external, recorder.known_injected_rejected, recorder.excluded_secure_input, recorder.excluded_app, IsSecureEventInputEnabled() ? "true" : "false", recorder.lease_active ? "true" : "false", access, recorder.hid_count, correlated, window_ms, min_delta_ms, recorder.gaps);
  if (manager) { IOHIDManagerUnscheduleFromRunLoop(manager, CFRunLoopGetCurrent(), kCFRunLoopCommonModes); IOHIDManagerClose(manager, kIOHIDOptionsTypeNone); CFRelease(manager); }
  CFRunLoopRemoveSource(CFRunLoopGetCurrent(), source, kCFRunLoopCommonModes);
  CFRelease(source); CFRelease(tap); return recorder.gaps ? 4 : 0;
}
