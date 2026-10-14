#import <AVFoundation/AVFoundation.h>
#import <Foundation/Foundation.h>
#import <objc/runtime.h>
#include <stdint.h>

typedef void (*YondaVoiceReplyCallback)(uint64_t generation, uint64_t sequence, int32_t event);

static AVSpeechSynthesizer *voiceSynthesizer;
static NSObject<AVSpeechSynthesizerDelegate> *voiceDelegate;
static YondaVoiceReplyCallback voiceCallback;
static NSUInteger pendingUtterances;
static char generationKey;
static char sequenceKey;
static NSString *const enabledKey = @"yonda.voiceReply.enabled";

static AVSpeechSynthesisVoice *localChineseVoice(void) {
    for (AVSpeechSynthesisVoice *voice in [AVSpeechSynthesisVoice speechVoices]) {
        if ([[voice.language lowercaseString] hasPrefix:@"zh"]) return voice;
    }
    return nil;
}

static uint64_t metadata(AVSpeechUtterance *utterance, const void *key) {
    NSNumber *number = objc_getAssociatedObject(utterance, key);
    return number ? number.unsignedLongLongValue : 0;
}

@interface YondaVoiceReplyDelegate : NSObject <AVSpeechSynthesizerDelegate>
@end

@implementation YondaVoiceReplyDelegate
- (void)speechSynthesizer:(AVSpeechSynthesizer *)synthesizer didStartSpeechUtterance:(AVSpeechUtterance *)utterance {
    if (voiceCallback) voiceCallback(metadata(utterance, &generationKey), metadata(utterance, &sequenceKey), 1);
}
- (void)speechSynthesizer:(AVSpeechSynthesizer *)synthesizer didFinishSpeechUtterance:(AVSpeechUtterance *)utterance {
    uint64_t generation = metadata(utterance, &generationKey);
    uint64_t sequence = metadata(utterance, &sequenceKey);
    if (pendingUtterances > 0) pendingUtterances--;
    if (voiceCallback) voiceCallback(generation, sequence, 2);
    if (pendingUtterances == 0 && voiceCallback) voiceCallback(generation, sequence, 4);
}
- (void)speechSynthesizer:(AVSpeechSynthesizer *)synthesizer didCancelSpeechUtterance:(AVSpeechUtterance *)utterance {
    uint64_t generation = metadata(utterance, &generationKey);
    uint64_t sequence = metadata(utterance, &sequenceKey);
    if (pendingUtterances > 0) pendingUtterances--;
    if (voiceCallback) voiceCallback(generation, sequence, 3);
}
@end

static void onMain(void (^work)(void)) {
    if ([NSThread isMainThread]) work();
    else dispatch_async(dispatch_get_main_queue(), work);
}

bool yonda_voice_reply_available(void) {
    __block bool available = false;
    void (^check)(void) = ^{ available = localChineseVoice() != nil; };
    if ([NSThread isMainThread]) check();
    else dispatch_sync(dispatch_get_main_queue(), check);
    return available;
}

bool yonda_voice_reply_enabled(void) {
    return [[NSUserDefaults standardUserDefaults] boolForKey:enabledKey];
}

void yonda_voice_reply_set_enabled(bool enabled) {
    [[NSUserDefaults standardUserDefaults] setBool:enabled forKey:enabledKey];
}

void yonda_voice_reply_speak(uint64_t generation, uint64_t sequence, const char *text, YondaVoiceReplyCallback callback) {
    NSString *value = text ? [NSString stringWithUTF8String:text] : @"";
    onMain(^{
        AVSpeechSynthesisVoice *voice = localChineseVoice();
        voiceCallback = callback;
        if (!voice || value.length == 0) {
            if (voiceCallback) voiceCallback(generation, sequence, 5);
            return;
        }
        if (!voiceSynthesizer) {
            voiceSynthesizer = [AVSpeechSynthesizer new];
            voiceDelegate = [YondaVoiceReplyDelegate new];
            voiceSynthesizer.delegate = voiceDelegate;
        }
        AVSpeechUtterance *utterance = [AVSpeechUtterance speechUtteranceWithString:value];
        utterance.voice = voice;
        objc_setAssociatedObject(utterance, &generationKey, @(generation), OBJC_ASSOCIATION_RETAIN_NONATOMIC);
        objc_setAssociatedObject(utterance, &sequenceKey, @(sequence), OBJC_ASSOCIATION_RETAIN_NONATOMIC);
        pendingUtterances++;
        [voiceSynthesizer speakUtterance:utterance];
    });
}

void yonda_voice_reply_stop(void) {
    onMain(^{
        pendingUtterances = 0;
        [voiceSynthesizer stopSpeakingAtBoundary:AVSpeechBoundaryImmediate];
    });
}

void yonda_voice_reply_release(void) {
    onMain(^{
        pendingUtterances = 0;
        [voiceSynthesizer stopSpeakingAtBoundary:AVSpeechBoundaryImmediate];
        voiceSynthesizer.delegate = nil;
        [voiceSynthesizer release];
        [voiceDelegate release];
        voiceSynthesizer = nil;
        voiceDelegate = nil;
        voiceCallback = nil;
    });
}
