#include <atomic>
#include <chrono>
#include <iostream>
#include <string>
#include <thread>
#include <vector>

#include <Windows.h>
#include <appmodel.h>
#include <winrt/Windows.Foundation.h>
#include <winrt/Windows.Globalization.h>
#include <winrt/Windows.Media.Core.h>
#include <winrt/Windows.Media.Playback.h>
#include <winrt/Windows.Media.SpeechRecognition.h>
#include <winrt/Windows.Media.SpeechSynthesis.h>
#include <winrt/Windows.Storage.Streams.h>

using namespace std::chrono_literals;
using namespace winrt;
using namespace Windows::Globalization;
using namespace Windows::Media::Core;
using namespace Windows::Media::Playback;
using namespace Windows::Media::SpeechRecognition;
using namespace Windows::Media::SpeechSynthesis;

template <typename Async>
void wait_for_async(Async const& operation, std::chrono::seconds timeout,
                    char const* stage) {
  auto const deadline = std::chrono::steady_clock::now() + timeout;
  while (operation.Status() == Windows::Foundation::AsyncStatus::Started &&
         std::chrono::steady_clock::now() < deadline) {
    std::this_thread::sleep_for(25ms);
  }
  if (operation.Status() == Windows::Foundation::AsyncStatus::Started) {
    operation.Cancel();
    std::cerr << "stage-timeout=" << stage << std::endl;
    throw hresult_error(HRESULT_FROM_WIN32(ERROR_TIMEOUT));
  }
}

int main(int argc, char** argv) {
  init_apartment(apartment_type::multi_threaded);

  const bool argument_authorized =
      argc == 2 && std::string{argv[1]} == "--authorized-start";
  const bool identity_only =
      argc == 2 && std::string{argv[1]} == "--identity-only";
  UINT32 package_name_length = 0;
  auto package_result =
      GetCurrentPackageFullName(&package_name_length, nullptr);
  bool const package_identity_present =
      package_result == ERROR_INSUFFICIENT_BUFFER;
  if (identity_only) {
    std::cout << "{\"platform\":\"Windows\",\"identity_only\":true"
              << ",\"package_identity_present\":"
              << (package_identity_present ? "true" : "false")
              << ",\"opened_microphone\":false}"
              << '\n';
    return 0;
  }

  std::cout << "VI-S1 Windows explicit voice probe\n"
            << "This test opens the microphone for at most 12 seconds and "
               "plays a public test sentence.\n"
            << "No PCM or transcript is printed or saved.\n";

  std::string confirmation;
  if (argument_authorized) {
    confirmation = "START";
  } else {
    std::cout << "Type START and press Enter to continue: " << std::flush;
    std::getline(std::cin, confirmation);
  }
  if (confirmation != "START") {
    std::cout << R"({"platform":"Windows","authorized":false,"opened_microphone":false,"saved_audio":false,"saved_transcript":false})"
              << '\n';
    return 2;
  }

  bool opened_microphone = false;
  bool stopped = false;
  bool synthesized = false;
  bool played = false;
  bool stop_timed_out = false;
  std::string stop_mode = "not-started";
  std::atomic<uint32_t> partial_count{0};
  std::atomic<uint32_t> final_count{0};
  std::atomic<uint32_t> error_count{0};
  int32_t exit_code = 0;
  int32_t hresult_value = 0;
  std::string failure_stage = "construct-recognizer";

  try {
    std::cerr << "stage=construct-recognizer" << std::endl;
    SpeechRecognizer recognizer{Language{L"zh-CN"}};
    auto session = recognizer.ContinuousRecognitionSession();

    auto hypothesis_token = recognizer.HypothesisGenerated(
        [&](SpeechRecognizer const&, SpeechRecognitionHypothesisGeneratedEventArgs const&) {
          ++partial_count;
        });
    auto result_token = session.ResultGenerated(
        [&](SpeechContinuousRecognitionSession const&, SpeechContinuousRecognitionResultGeneratedEventArgs const& args) {
          if (args.Result().Status() == SpeechRecognitionResultStatus::Success) {
            ++final_count;
          } else {
            ++error_count;
          }
        });
    auto completed_token = session.Completed(
        [&](SpeechContinuousRecognitionSession const&, SpeechContinuousRecognitionCompletedEventArgs const& args) {
          if (args.Status() != SpeechRecognitionResultStatus::Success) {
            ++error_count;
          }
        });

    failure_stage = "compile-constraints";
    std::cerr << "stage=compile-constraints" << std::endl;
    auto compilation_operation = recognizer.CompileConstraintsAsync();
    wait_for_async(compilation_operation, 10s, failure_stage.c_str());
    auto compilation = compilation_operation.get();
    if (compilation.Status() != SpeechRecognitionResultStatus::Success) {
      throw hresult_error(E_FAIL, L"speech constraint compilation failed");
    }

    failure_stage = "start-session";
    std::cerr << "stage=start-session" << std::endl;
    auto start_operation = session.StartAsync();
    wait_for_async(start_operation, 10s, failure_stage.c_str());
    start_operation.get();
    opened_microphone = true;

    failure_stage = "synthesize-public-sentence";
    std::cerr << "stage=synthesize-public-sentence" << std::endl;
    SpeechSynthesizer synthesizer;
    auto synthesis_operation = synthesizer.SynthesizeTextToStreamAsync(
        L"这是 Yonder 语音输入公开测试句。");
    wait_for_async(synthesis_operation, 10s, failure_stage.c_str());
    auto stream = synthesis_operation.get();
    synthesized = true;

    MediaPlayer player;
    player.Source(MediaSource::CreateFromStream(stream, stream.ContentType()));
    player.Play();
    played = true;

    failure_stage = "capture-window";
    std::cerr << "stage=capture-window" << std::endl;
    std::this_thread::sleep_for(12s);

    failure_stage = "stop-session";
    std::cerr << "stage=stop-session" << std::endl;
    auto stop_operation = session.StopAsync();
    auto const stop_deadline = std::chrono::steady_clock::now() + 10s;
    while (stop_operation.Status() ==
               Windows::Foundation::AsyncStatus::Started &&
           std::chrono::steady_clock::now() < stop_deadline) {
      std::this_thread::sleep_for(25ms);
    }
    if (stop_operation.Status() ==
        Windows::Foundation::AsyncStatus::Started) {
      stop_timed_out = true;
      stop_operation.Cancel();
      stop_mode = "cancel-stop-operation-after-timeout";
      std::cerr << "stage=cancel-stop-operation" << std::endl;
      auto const cancel_deadline = std::chrono::steady_clock::now() + 2s;
      while (stop_operation.Status() ==
                 Windows::Foundation::AsyncStatus::Started &&
             std::chrono::steady_clock::now() < cancel_deadline) {
        std::this_thread::sleep_for(25ms);
      }
      throw hresult_error(HRESULT_FROM_WIN32(ERROR_TIMEOUT));
    } else {
      stop_operation.get();
      stop_mode = "graceful";
    }
    stopped = true;
    player.Pause();

    recognizer.HypothesisGenerated(hypothesis_token);
    session.ResultGenerated(result_token);
    session.Completed(completed_token);
    player.Source(nullptr);
    stream.Close();
    synthesizer.Close();
    recognizer.Close();
    if (partial_count.load() == 0 && final_count.load() == 0) {
      exit_code = 3;
      failure_stage = "no-recognition-result";
    } else {
      failure_stage = "complete";
    }
  } catch (hresult_error const& error) {
    exit_code = 1;
    hresult_value = error.code().value;
  } catch (...) {
    exit_code = 1;
    hresult_value = static_cast<int32_t>(E_UNEXPECTED);
  }

  std::cout << "{\"platform\":\"Windows\",\"authorized\":true"
            << ",\"authorization_source\":\""
            << (argument_authorized ? "command-line" : "console") << "\""
            << ",\"package_identity_present\":"
            << (package_identity_present ? "true" : "false")
            << ",\"opened_microphone\":" << (opened_microphone ? "true" : "false")
            << ",\"synthesized_public_sentence\":" << (synthesized ? "true" : "false")
            << ",\"played_public_sentence\":" << (played ? "true" : "false")
            << ",\"partial_count\":" << partial_count.load()
            << ",\"final_count\":" << final_count.load()
            << ",\"error_count\":" << error_count.load()
            << ",\"stopped\":" << (stopped ? "true" : "false")
            << ",\"stop_timed_out\":"
            << (stop_timed_out ? "true" : "false")
            << ",\"stop_mode\":\"" << stop_mode << "\""
            << ",\"saved_audio\":false,\"saved_transcript\":false"
            << ",\"failure_stage\":\"" << failure_stage << "\""
            << ",\"hresult\":" << hresult_value << "}\n";
  return exit_code;
}
