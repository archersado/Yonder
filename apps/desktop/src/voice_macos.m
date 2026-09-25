#import <AVFoundation/AVFoundation.h>
#import <Foundation/Foundation.h>
#import <Speech/Speech.h>
#import <math.h>

typedef void (*YondaVoiceCallback)(long long, int, const char *);
typedef struct {
    double voicedDuration;
    double silentDuration;
    BOOL heardSpeech;
} YondaVoiceActivityState;

static const float voiceLevelThreshold = 0.015f;
static const double minimumVoiceDuration = 0.24;
static const double trailingSilenceDuration = 0.90;
static AVAudioEngine *engine;
static SFSpeechRecognizer *recognizer;
static SFSpeechAudioBufferRecognitionRequest *request;
static SFSpeechRecognitionTask *task;
static dispatch_source_t timeoutSource;
static YondaVoiceCallback callback;
static long generation;
static long long activeSessionIdentifier;
static BOOL active;
static BOOL recognizing;
static BOOL finalizationRequested;
static BOOL terminalDelivered;
static NSString *lastTranscript;
static id configurationObserver;
static YondaVoiceActivityState activityState;

static void emit(long long sessionIdentifier, int kind, NSString *text) {
    if (callback) callback(sessionIdentifier, kind, (text ?: @"").UTF8String);
}

static BOOL observeVoiceActivity(YondaVoiceActivityState *state, float level, double duration) {
    if (level >= voiceLevelThreshold) {
        state->voicedDuration += duration;
        state->silentDuration = 0;
        if (state->voicedDuration >= minimumVoiceDuration) state->heardSpeech = YES;
    } else if (state->heardSpeech) {
        state->silentDuration += duration;
    } else {
        state->voicedDuration = 0;
    }
    return state->heardSpeech && state->silentDuration + 1e-9 >= trailingSilenceDuration;
}

static BOOL claimFinalization(BOOL *requested) {
    if (*requested) return NO;
    *requested = YES;
    return YES;
}

int yonda_voice_vad_test(const float *levels, const double *durations, int count) {
    YondaVoiceActivityState state = {0};
    for (int index = 0; index < count; index++) {
        if (observeVoiceActivity(&state, levels[index], durations[index])) return index + 1;
    }
    return 0;
}

int yonda_voice_finish_gate_test(int signalCount) {
    BOOL requested = NO;
    int accepted = 0;
    for (int index = 0; index < signalCount; index++) if (claimFinalization(&requested)) accepted++;
    return accepted;
}

static float rootMeanSquare(AVAudioPCMBuffer *buffer) {
    float *const *channels = buffer.floatChannelData;
    AVAudioFrameCount frames = buffer.frameLength;
    AVAudioChannelCount channelCount = buffer.format.channelCount;
    if (!channels || frames == 0 || channelCount == 0) return 0;
    double sum = 0;
    for (AVAudioChannelCount channel = 0; channel < channelCount; channel++) {
        for (AVAudioFrameCount frame = 0; frame < frames; frame++) {
            float sample = channels[channel][frame];
            sum += sample * sample;
        }
    }
    return (float)sqrt(sum / ((double)frames * channelCount));
}

static void releaseCapture(BOOL cancelTask) {
    if (configurationObserver) {
        [[NSNotificationCenter defaultCenter] removeObserver:configurationObserver];
        configurationObserver = nil;
    }
    if (engine.isRunning) [engine stop];
    @try { [engine.inputNode removeTapOnBus:0]; } @catch (__unused NSException *exception) {}
    if (cancelTask) [task cancel];
    if (timeoutSource) { dispatch_source_cancel(timeoutSource); timeoutSource = nil; }
    [request release]; request = nil;
    task = nil;
    [engine release]; engine = nil;
    [recognizer release]; recognizer = nil;
    active = NO;
    [lastTranscript release]; lastTranscript = nil;
}

static void requestFinalization(long token, long long sessionIdentifier) {
    if (token != generation || !active || !claimFinalization(&finalizationRequested)) return;
    if (engine.isRunning) [engine stop];
    @try { [engine.inputNode removeTapOnBus:0]; } @catch (__unused NSException *exception) {}
    active = NO;
    if (timeoutSource) { dispatch_source_cancel(timeoutSource); timeoutSource = nil; }
    [request endAudio];
    emit(sessionIdentifier, 5, @"");
}

static void failCapture(long token, long long sessionIdentifier, NSString *message) {
    if (token != generation || terminalDelivered) return;
    terminalDelivered = YES;
    recognizing = NO;
    releaseCapture(YES);
    emit(sessionIdentifier, 4, message);
}

static NSString *speechErrorMessage(NSError *error) {
    if ([error.domain isEqualToString:@"kLSRErrorDomain"]) {
        switch (error.code) {
            case 102: return @"语音识别组件缺失";
            case 201: return @"语音识别服务已关闭";
            case 300: return @"语音识别器初始化失败";
        }
    } else if ([error.domain isEqualToString:@"kAFAssistantErrorDomain"]) {
        switch (error.code) {
            case 1100: return @"语音识别任务冲突";
            case 1101:
            case 1107: return @"语音服务中断，请重试";
            case 1110: return @"没有识别到语音，请重试";
            case 1700: return @"语音识别权限已撤销";
            case 203: return @"语音识别失败，请重试";
        }
    }
    return @"语音输入失败，请重试";
}

static void beginCapture(long token, long long sessionIdentifier) {
    if (token != generation) return;
    recognizer = [[SFSpeechRecognizer alloc] initWithLocale:[NSLocale localeWithLocaleIdentifier:@"zh-CN"]];
    if (!recognizer || !recognizer.available) {
        [recognizer release]; recognizer = nil;
        emit(sessionIdentifier, 4, @"中文语音识别当前不可用"); return;
    }
    engine = [AVAudioEngine new];
    request = [SFSpeechAudioBufferRecognitionRequest new];
    [lastTranscript release]; lastTranscript = nil;
    activeSessionIdentifier = sessionIdentifier;
    finalizationRequested = NO;
    terminalDelivered = NO;
    activityState = (YondaVoiceActivityState){0};
    request.shouldReportPartialResults = YES;
    if (@available(macOS 10.15, *)) request.requiresOnDeviceRecognition = recognizer.supportsOnDeviceRecognition;
    AVAudioInputNode *input = engine.inputNode;
    AVAudioFormat *format = [input outputFormatForBus:0];
    [input installTapOnBus:0 bufferSize:1024 format:format block:^(AVAudioPCMBuffer *buffer, AVAudioTime *when) {
        (void)when;
        [request appendAudioPCMBuffer:buffer];
        float level = rootMeanSquare(buffer);
        double duration = format.sampleRate > 0 ? buffer.frameLength / format.sampleRate : 0;
        dispatch_async(dispatch_get_main_queue(), ^{
            if (token != generation || sessionIdentifier != activeSessionIdentifier || !active || finalizationRequested) return;
            if (observeVoiceActivity(&activityState, level, duration)) requestFinalization(token, sessionIdentifier);
        });
    }];
    NSError *error;
    [engine prepare];
    if (![engine startAndReturnError:&error]) { releaseCapture(YES); emit(sessionIdentifier, 4, @"麦克风启动失败"); return; }
    active = YES;
    recognizing = YES;
    emit(sessionIdentifier, 1, @"");
    configurationObserver = [[NSNotificationCenter defaultCenter] addObserverForName:AVAudioEngineConfigurationChangeNotification
        object:engine
        queue:[NSOperationQueue mainQueue]
        usingBlock:^(__unused NSNotification *notification) {
            if (token != generation || !active) return;
            failCapture(token, sessionIdentifier, @"音频设备变化，请重试");
        }];
    task = [recognizer recognitionTaskWithRequest:request resultHandler:^(SFSpeechRecognitionResult *result, NSError *error) {
        dispatch_async(dispatch_get_main_queue(), ^{
            if (token != generation || terminalDelivered) return;
            NSString *text = result.bestTranscription.formattedString ?: @"";
            if (text.length) { [lastTranscript release]; lastTranscript = [text copy]; }
            if (result.final) {
                terminalDelivered = YES;
                NSString *finalText = activityState.heardSpeech ? (text.length ? text : (lastTranscript ?: @"")) : @"";
                [finalText retain];
                recognizing = NO;
                releaseCapture(NO);
                emit(sessionIdentifier, 3, finalText);
                [finalText release];
            } else if (result && !finalizationRequested) {
                emit(sessionIdentifier, 2, text);
            } else if (error && recognizing) {
                failCapture(token, sessionIdentifier, speechErrorMessage(error));
            }
        });
    }];
    timeoutSource = dispatch_source_create(DISPATCH_SOURCE_TYPE_TIMER, 0, 0, dispatch_get_main_queue());
    dispatch_source_set_timer(timeoutSource, dispatch_time(DISPATCH_TIME_NOW, 20 * NSEC_PER_SEC), DISPATCH_TIME_FOREVER, 0);
    dispatch_source_set_event_handler(timeoutSource, ^{ requestFinalization(token, sessionIdentifier); });
    dispatch_resume(timeoutSource);
}

void yonda_voice_start(long long sessionIdentifier, YondaVoiceCallback sink) {
    dispatch_async(dispatch_get_main_queue(), ^{
        if (active) return;
        callback = sink; long token = ++generation; emit(sessionIdentifier, 0, @"");
        [SFSpeechRecognizer requestAuthorization:^(SFSpeechRecognizerAuthorizationStatus speech) {
            if (speech != SFSpeechRecognizerAuthorizationStatusAuthorized) { dispatch_async(dispatch_get_main_queue(), ^{ if (token == generation) emit(sessionIdentifier, 4, @"需要语音识别权限"); }); return; }
            [AVCaptureDevice requestAccessForMediaType:AVMediaTypeAudio completionHandler:^(BOOL microphone) {
                dispatch_async(dispatch_get_main_queue(), ^{ if (token != generation) return; if (microphone) beginCapture(token, sessionIdentifier); else emit(sessionIdentifier, 4, @"需要麦克风权限"); });
            }];
        }];
    });
}

void yonda_voice_stop(void) {
    dispatch_async(dispatch_get_main_queue(), ^{ requestFinalization(generation, activeSessionIdentifier); });
}

void yonda_voice_cancel(void) {
    dispatch_async(dispatch_get_main_queue(), ^{ long long sessionIdentifier = activeSessionIdentifier; ++generation; recognizing = NO; releaseCapture(YES); emit(sessionIdentifier, 6, @""); });
}
