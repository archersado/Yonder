#define WIN32_LEAN_AND_MEAN
#include <Windows.h>
#include <d3d11.h>
#include <dwmapi.h>
#include <dxgi1_2.h>
#include <windows.graphics.capture.interop.h>
#include <windows.graphics.directx.direct3d11.interop.h>

#include <array>
#include <cstdint>
#include <iostream>

#include <winrt/Windows.Foundation.h>
#include <winrt/Windows.Graphics.Capture.h>
#include <winrt/Windows.Graphics.DirectX.h>
#include <winrt/Windows.Graphics.DirectX.Direct3D11.h>

using namespace winrt;
namespace capture = winrt::Windows::Graphics::Capture;
namespace directx = winrt::Windows::Graphics::DirectX;
namespace direct3d = winrt::Windows::Graphics::DirectX::Direct3D11;
namespace direct3d_abi = ::Windows::Graphics::DirectX::Direct3D11;

namespace {
constexpr int logical_width = 240;
constexpr int logical_height = 160;

LRESULT CALLBACK window_proc(HWND window, UINT message, WPARAM wparam, LPARAM lparam) {
  if (message == WM_PAINT) {
    PAINTSTRUCT paint{};
    HDC dc = BeginPaint(window, &paint);
    RECT client{};
    GetClientRect(window, &client);
    const int middle_x = (client.right - client.left) / 2;
    const int middle_y = (client.bottom - client.top) / 2;
    const std::array<COLORREF, 4> colors{
        RGB(255, 0, 0), RGB(0, 255, 0), RGB(0, 0, 255), RGB(255, 255, 0)};
    const std::array<RECT, 4> rectangles{{
        {0, 0, middle_x, middle_y},
        {middle_x, 0, client.right, middle_y},
        {0, middle_y, middle_x, client.bottom},
        {middle_x, middle_y, client.right, client.bottom},
    }};
    for (size_t index = 0; index < rectangles.size(); ++index) {
      HBRUSH brush = CreateSolidBrush(colors[index]);
      FillRect(dc, &rectangles[index], brush);
      DeleteObject(brush);
    }
    EndPaint(window, &paint);
    return 0;
  }
  if (message == WM_ERASEBKGND) return 1;
  return DefWindowProcW(window, message, wparam, lparam);
}

BOOL CALLBACK enumerate_monitor(HMONITOR, HDC, LPRECT, LPARAM value) {
  auto* count = reinterpret_cast<uint32_t*>(value);
  ++*count;
  return true;
}

direct3d::IDirect3DDevice make_direct3d_device(com_ptr<ID3D11Device>& native_device,
                                               com_ptr<ID3D11DeviceContext>& context) {
  UINT flags = D3D11_CREATE_DEVICE_BGRA_SUPPORT;
  D3D_FEATURE_LEVEL feature_level{};
  check_hresult(D3D11CreateDevice(nullptr, D3D_DRIVER_TYPE_HARDWARE, nullptr, flags,
                                  nullptr, 0, D3D11_SDK_VERSION,
                                  native_device.put(), &feature_level, context.put()));
  auto dxgi_device = native_device.as<IDXGIDevice>();
  com_ptr<::IInspectable> inspectable;
  check_hresult(CreateDirect3D11DeviceFromDXGIDevice(dxgi_device.get(), inspectable.put()));
  return inspectable.as<direct3d::IDirect3DDevice>();
}

capture::GraphicsCaptureItem capture_item_for_window(HWND window) {
  capture::GraphicsCaptureItem item{nullptr};
  auto factory = get_activation_factory<capture::GraphicsCaptureItem, IGraphicsCaptureItemInterop>();
  check_hresult(factory->CreateForWindow(window, guid_of<capture::GraphicsCaptureItem>(), put_abi(item)));
  return item;
}

bool channel_near(uint8_t actual, uint8_t expected) {
  const int difference = static_cast<int>(actual) - static_cast<int>(expected);
  return difference >= -35 && difference <= 35;
}
}  // namespace

int main() {
  HINSTANCE instance = GetModuleHandleW(nullptr);
  init_apartment(apartment_type::multi_threaded);
  SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2);

  uint32_t monitor_count = 0;
  EnumDisplayMonitors(nullptr, nullptr, enumerate_monitor,
                      reinterpret_cast<LPARAM>(&monitor_count));
  const bool negative_origin = GetSystemMetrics(SM_XVIRTUALSCREEN) < 0 ||
                               GetSystemMetrics(SM_YVIRTUALSCREEN) < 0;
  UINT32 path_count = 0;
  UINT32 mode_count = 0;
  const LONG buffer_status = GetDisplayConfigBufferSizes(
      QDC_ONLY_ACTIVE_PATHS, &path_count, &mode_count);

  bool capture_supported = capture::GraphicsCaptureSession::IsSupported();
  bool colors_pass = false;
  bool size_pass = false;
  bool pixels_released = false;
  int logical_result_width = logical_width;
  int logical_result_height = logical_height;
  int pixel_width = 0;
  int pixel_height = 0;
  UINT dpi = 96;
  int32_t error_code = 0;
  HWND window = nullptr;

  try {
    if (!capture_supported) throw hresult_error(E_NOTIMPL);

    WNDCLASSW window_class{};
    window_class.hInstance = instance;
    window_class.lpfnWndProc = window_proc;
    window_class.lpszClassName = L"YonderRegionCaptureSpike";
    window_class.hCursor = LoadCursor(nullptr, IDC_ARROW);
    if (!RegisterClassW(&window_class) && GetLastError() != ERROR_CLASS_ALREADY_EXISTS) {
      throw_last_error();
    }

    POINT origin{0, 0};
    HMONITOR monitor = MonitorFromPoint(origin, MONITOR_DEFAULTTOPRIMARY);
    MONITORINFO monitor_info{sizeof(monitor_info)};
    check_bool(GetMonitorInfoW(monitor, &monitor_info));
    dpi = GetDpiForSystem();
    const int physical_width = MulDiv(logical_width, static_cast<int>(dpi), 96);
    const int physical_height = MulDiv(logical_height, static_cast<int>(dpi), 96);
    const int x = monitor_info.rcWork.left +
                  ((monitor_info.rcWork.right - monitor_info.rcWork.left) - physical_width) / 2;
    const int y = monitor_info.rcWork.top +
                  ((monitor_info.rcWork.bottom - monitor_info.rcWork.top) - physical_height) / 2;

    window = CreateWindowExW(WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE,
                             window_class.lpszClassName, L"", WS_POPUP,
                             x, y, physical_width, physical_height,
                             nullptr, nullptr, instance, nullptr);
    if (!window) throw_last_error();
    dpi = GetDpiForWindow(window);
    const int monitor_width = MulDiv(logical_width, static_cast<int>(dpi), 96);
    const int monitor_height = MulDiv(logical_height, static_cast<int>(dpi), 96);
    check_bool(SetWindowPos(window, nullptr,
                            monitor_info.rcWork.left + ((monitor_info.rcWork.right - monitor_info.rcWork.left) - monitor_width) / 2,
                            monitor_info.rcWork.top + ((monitor_info.rcWork.bottom - monitor_info.rcWork.top) - monitor_height) / 2,
                            monitor_width, monitor_height,
                            SWP_NOACTIVATE | SWP_NOZORDER));
    ShowWindow(window, SW_SHOWNOACTIVATE);
    UpdateWindow(window);
    DwmFlush();

    auto item = capture_item_for_window(window);
    auto item_size = item.Size();
    pixel_width = item_size.Width;
    pixel_height = item_size.Height;

    com_ptr<ID3D11Device> native_device;
    com_ptr<ID3D11DeviceContext> context;
    auto device = make_direct3d_device(native_device, context);
    auto frame_pool = capture::Direct3D11CaptureFramePool::CreateFreeThreaded(
        device, directx::DirectXPixelFormat::B8G8R8A8UIntNormalized, 1, item_size);
    auto session = frame_pool.CreateCaptureSession(item);
    session.IsCursorCaptureEnabled(false);

    handle frame_ready{CreateEventW(nullptr, TRUE, FALSE, nullptr)};
    if (!frame_ready) throw_last_error();
    auto token = frame_pool.FrameArrived(
        [&](capture::Direct3D11CaptureFramePool const&,
            winrt::Windows::Foundation::IInspectable const&) {
          SetEvent(frame_ready.get());
        });
    session.StartCapture();
    const DWORD wait_result = WaitForSingleObject(frame_ready.get(), 5000);
    if (wait_result != WAIT_OBJECT_0) throw hresult_error(HRESULT_FROM_WIN32(ERROR_TIMEOUT));

    auto frame = frame_pool.TryGetNextFrame();
    if (!frame) throw hresult_error(E_FAIL);
    auto surface_access = frame.Surface().as<direct3d_abi::IDirect3DDxgiInterfaceAccess>();
    com_ptr<ID3D11Texture2D> source;
    check_hresult(surface_access->GetInterface(__uuidof(ID3D11Texture2D), source.put_void()));

    D3D11_TEXTURE2D_DESC description{};
    source->GetDesc(&description);
    description.Usage = D3D11_USAGE_STAGING;
    description.BindFlags = 0;
    description.CPUAccessFlags = D3D11_CPU_ACCESS_READ;
    description.MiscFlags = 0;
    com_ptr<ID3D11Texture2D> staging;
    check_hresult(native_device->CreateTexture2D(&description, nullptr, staging.put()));
    context->CopyResource(staging.get(), source.get());

    D3D11_MAPPED_SUBRESOURCE mapped{};
    check_hresult(context->Map(staging.get(), 0, D3D11_MAP_READ, 0, &mapped));
    const std::array<std::array<uint8_t, 3>, 4> expected{{
        {0, 0, 255}, {0, 255, 0}, {255, 0, 0}, {0, 255, 255}}};
    const std::array<std::array<UINT, 2>, 4> samples{{
        {description.Width / 4, description.Height / 4},
        {description.Width * 3 / 4, description.Height / 4},
        {description.Width / 4, description.Height * 3 / 4},
        {description.Width * 3 / 4, description.Height * 3 / 4}}};
    colors_pass = true;
    for (size_t index = 0; index < samples.size(); ++index) {
      const auto* pixel = static_cast<const uint8_t*>(mapped.pData) +
                          samples[index][1] * mapped.RowPitch + samples[index][0] * 4;
      colors_pass = colors_pass && channel_near(pixel[0], expected[index][0]) &&
                    channel_near(pixel[1], expected[index][1]) &&
                    channel_near(pixel[2], expected[index][2]);
    }
    context->Unmap(staging.get(), 0);

    const int expected_width = MulDiv(logical_width, static_cast<int>(dpi), 96);
    const int expected_height = MulDiv(logical_height, static_cast<int>(dpi), 96);
    size_pass = pixel_width == expected_width && pixel_height == expected_height &&
                description.Width == static_cast<UINT>(pixel_width) &&
                description.Height == static_cast<UINT>(pixel_height);

    frame_pool.FrameArrived(token);
    frame.Close();
    session.Close();
    frame_pool.Close();
    staging = nullptr;
    source = nullptr;
    context->Flush();
    pixels_released = true;
  } catch (hresult_error const& error) {
    error_code = error.code().value;
  } catch (...) {
    error_code = static_cast<int32_t>(E_UNEXPECTED);
  }

  if (window) DestroyWindow(window);
  const bool passed = error_code == 0 && colors_pass && size_pass && pixels_released;
  const double scale = static_cast<double>(dpi) / 96.0;
  std::cout << "{\"platform\":\"Windows\",\"passed\":" << (passed ? "true" : "false")
            << ",\"capture_api\":\"Windows.Graphics.Capture\""
            << ",\"display_config_available\":" << (buffer_status == ERROR_SUCCESS ? "true" : "false")
            << ",\"display_count\":" << monitor_count
            << ",\"negative_origin_available\":" << (negative_origin ? "true" : "false")
            << ",\"logical_width\":" << logical_result_width
            << ",\"logical_height\":" << logical_result_height
            << ",\"pixels_width\":" << pixel_width
            << ",\"pixels_height\":" << pixel_height
            << ",\"scale\":" << scale
            << ",\"color_checks\":" << (colors_pass ? "true" : "false")
            << ",\"scale_check\":" << (size_pass ? "true" : "false")
            << ",\"pixels_released_after_check\":" << (pixels_released ? "true" : "false")
            << ",\"screenshot_persisted\":false,\"input_listeners_installed\":false"
            << ",\"error_code\":" << error_code << "}\n";
  return passed ? 0 : 2;
}
