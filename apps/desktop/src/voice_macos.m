#import <AVFoundation/AVFoundation.h>
#import <Foundation/Foundation.h>
#import <Speech/Speech.h>

typedef void (*YondaVoiceCallback)(int, const char *);
static AVAudioEngine *engine;
static SFSpeechRecognizer *recognizer;
static SFSpeechAudioBufferRecognitionRequest *request;
static SFSpeechRecognitionTask *task;
static dispatch_source_t timeoutSource;
static YondaVoiceCallback callback;
static long generation;
static BOOL active;
static BOOL recognizing;
static NSString *lastTranscript;

static void emit(int kind, NSString *text) {
    if (callback) callback(kind, (text ?: @"").UTF8String);
}

static void releaseCapture(BOOL cancelTask) {
    if (engine.isRunning) [engine stop];
    @try { [engine.inputNode removeTapOnBus:0]; } @catch (__unused NSException *exception) {}
    if (cancelTask) [task cancel];
    if (timeoutSource) { dispatch_source_cancel(timeoutSource); timeoutSource = nil; }
    request = nil; task = nil; active = NO;
    [lastTranscript release]; lastTranscript = nil;
}

static void beginCapture(long token) {
    if (token != generation) return;
    recognizer = [[SFSpeechRecognizer alloc] initWithLocale:[NSLocale localeWithLocaleIdentifier:@"zh-CN"]];
    if (!recognizer || !recognizer.available) { emit(4, @"中文语音识别当前不可用"); return; }
    engine = [AVAudioEngine new];
    request = [SFSpeechAudioBufferRecognitionRequest new];
    [lastTranscript release]; lastTranscript = nil;
    request.shouldReportPartialResults = YES;
    if (@available(macOS 10.15, *)) request.requiresOnDeviceRecognition = recognizer.supportsOnDeviceRecognition;
    AVAudioInputNode *input = engine.inputNode;
    AVAudioFormat *format = [input outputFormatForBus:0];
    [input installTapOnBus:0 bufferSize:1024 format:format block:^(AVAudioPCMBuffer *buffer, AVAudioTime *when) {
        (void)when; [request appendAudioPCMBuffer:buffer];
    }];
    NSError *error;
    [engine prepare];
    if (![engine startAndReturnError:&error]) { releaseCapture(YES); emit(4, @"麦克风启动失败"); return; }
    active = YES;
    recognizing = YES;
    emit(1, @"");
    task = [recognizer recognitionTaskWithRequest:request resultHandler:^(SFSpeechRecognitionResult *result, NSError *error) {
        dispatch_async(dispatch_get_main_queue(), ^{
            if (token != generation) return;
            NSString *text = result.bestTranscription.formattedString ?: @"";
            if (text.length) { [lastTranscript release]; lastTranscript = [text copy]; }
            if (result) emit(result.final ? 3 : 2, result.final && !text.length ? (lastTranscript ?: @"") : text);
            if (result.final) { recognizing = NO; releaseCapture(NO); }
            else if (error && recognizing) { recognizing = NO; releaseCapture(YES); emit(4, @"没有识别到语音，请重试"); }
        });
    }];
    timeoutSource = dispatch_source_create(DISPATCH_SOURCE_TYPE_TIMER, 0, 0, dispatch_get_main_queue());
    dispatch_source_set_timer(timeoutSource, dispatch_time(DISPATCH_TIME_NOW, 20 * NSEC_PER_SEC), DISPATCH_TIME_FOREVER, 0);
    dispatch_source_set_event_handler(timeoutSource, ^{ if (token == generation && active) { [engine stop]; [engine.inputNode removeTapOnBus:0]; active = NO; [request endAudio]; emit(5, @""); } });
    dispatch_resume(timeoutSource);
}

void yonda_voice_start(YondaVoiceCallback sink) {
    dispatch_async(dispatch_get_main_queue(), ^{
        if (active) return;
        callback = sink; long token = ++generation; emit(0, @"");
        [SFSpeechRecognizer requestAuthorization:^(SFSpeechRecognizerAuthorizationStatus speech) {
            if (speech != SFSpeechRecognizerAuthorizationStatusAuthorized) { dispatch_async(dispatch_get_main_queue(), ^{ if (token == generation) emit(4, @"需要语音识别权限"); }); return; }
            [AVCaptureDevice requestAccessForMediaType:AVMediaTypeAudio completionHandler:^(BOOL microphone) {
                dispatch_async(dispatch_get_main_queue(), ^{ if (token != generation) return; if (microphone) beginCapture(token); else emit(4, @"需要麦克风权限"); });
            }];
        }];
    });
}

void yonda_voice_stop(void) {
    dispatch_async(dispatch_get_main_queue(), ^{ if (active) { [engine stop]; [engine.inputNode removeTapOnBus:0]; active = NO; [request endAudio]; emit(5, @""); } });
}

void yonda_voice_cancel(void) {
    dispatch_async(dispatch_get_main_queue(), ^{ ++generation; recognizing = NO; releaseCapture(YES); emit(6, @""); });
}
