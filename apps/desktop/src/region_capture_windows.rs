//! Windows Graphics Capture 区域捕获。调用方必须先隐藏圈选层。

use std::{sync::mpsc, time::Duration};

use windows::{
    core::{factory, Interface},
    Foundation::TypedEventHandler,
    Graphics::{
        Capture::{Direct3D11CaptureFramePool, GraphicsCaptureItem, GraphicsCaptureSession},
        DirectX::{Direct3D11::IDirect3DDevice, DirectXPixelFormat},
    },
    Win32::{
        Foundation::{HMODULE, RECT},
        Graphics::{
            Direct3D::D3D_DRIVER_TYPE_HARDWARE,
            Direct3D11::{
                D3D11CreateDevice, ID3D11Device, ID3D11DeviceContext, ID3D11Resource,
                ID3D11Texture2D, D3D11_BOX, D3D11_CPU_ACCESS_READ,
                D3D11_CREATE_DEVICE_BGRA_SUPPORT, D3D11_MAP_READ, D3D11_MAPPED_SUBRESOURCE,
                D3D11_SDK_VERSION, D3D11_TEXTURE2D_DESC, D3D11_USAGE_STAGING,
            },
            Dxgi::{Common::{DXGI_FORMAT_B8G8R8A8_UNORM, DXGI_SAMPLE_DESC}, IDXGIDevice},
            Gdi::{GetMonitorInfoW, MonitorFromRect, HMONITOR, MONITORINFO, MONITOR_DEFAULTTONULL},
        },
        System::WinRT::{
            Direct3D11::{CreateDirect3D11DeviceFromDXGIDevice, IDirect3DDxgiInterfaceAccess},
            Graphics::Capture::IGraphicsCaptureItemInterop,
            RoInitialize, RoUninitialize, RO_INIT_MULTITHREADED,
        },
        UI::HiDpi::{
            GetDpiForMonitor, SetThreadDpiAwarenessContext,
            DPI_AWARENESS_CONTEXT, DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2, MDT_EFFECTIVE_DPI,
        },
    },
};

const MAX_PIXELS: u64 = 16_777_216;
const FRAME_TIMEOUT: Duration = Duration::from_secs(2);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CaptureError {
    Unsupported,
    UnsupportedScale,
    OutOfBounds,
    TooLarge,
    Timeout,
    Unavailable,
}

impl CaptureError {
    pub fn code(self) -> &'static str {
        match self {
            Self::Unsupported => "capture-unsupported",
            Self::UnsupportedScale => "capture-unsupported-scale",
            Self::OutOfBounds => "capture-out-of-bounds",
            Self::TooLarge => "capture-too-large",
            Self::Timeout => "capture-timeout",
            Self::Unavailable => "capture-failed",
        }
    }
}

pub struct CapturedRegion {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
}

struct MonitorSelection {
    handle: HMONITOR,
    crop_left: u32,
    crop_top: u32,
    width: u32,
    height: u32,
}

pub fn capture_region(x: i32, y: i32, width: u32, height: u32) -> Result<CapturedRegion, CaptureError> {
    let previous_dpi_context = unsafe { SetThreadDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2) };
    if previous_dpi_context.0.is_null() { return Err(CaptureError::Unavailable); }
    let _dpi_context = DpiContext(previous_dpi_context);
    if width == 0 || height == 0 || u64::from(width) * u64::from(height) > MAX_PIXELS {
        return Err(CaptureError::TooLarge);
    }
    let selection = monitor_selection(x, y, width, height)?;
    capture_monitor_region(selection)
}

struct DpiContext(DPI_AWARENESS_CONTEXT);

impl Drop for DpiContext {
    fn drop(&mut self) { unsafe { SetThreadDpiAwarenessContext(self.0); } }
}

pub fn encode_png(capture: &CapturedRegion) -> Result<Vec<u8>, CaptureError> {
    let mut output = Vec::new();
    {
        let mut encoder = png::Encoder::new(&mut output, capture.width, capture.height);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        let mut writer = encoder.write_header().map_err(|_| CaptureError::Unavailable)?;
        writer.write_image_data(&capture.rgba).map_err(|_| CaptureError::Unavailable)?;
    }
    Ok(output)
}

fn monitor_selection(x: i32, y: i32, width: u32, height: u32) -> Result<MonitorSelection, CaptureError> {
    let right = x.checked_add(i32::try_from(width).map_err(|_| CaptureError::OutOfBounds)?)
        .ok_or(CaptureError::OutOfBounds)?;
    let bottom = y.checked_add(i32::try_from(height).map_err(|_| CaptureError::OutOfBounds)?)
        .ok_or(CaptureError::OutOfBounds)?;
    let rect = RECT { left: x, top: y, right, bottom };
    let handle = unsafe { MonitorFromRect(&rect, MONITOR_DEFAULTTONULL) };
    if handle.is_invalid() { return Err(CaptureError::OutOfBounds); }

    let mut info = MONITORINFO { cbSize: size_of::<MONITORINFO>() as u32, ..Default::default() };
    if !unsafe { GetMonitorInfoW(handle, &mut info) }.as_bool() { return Err(CaptureError::Unavailable); }
    let bounds = info.rcMonitor;
    if x < bounds.left || y < bounds.top || right > bounds.right || bottom > bounds.bottom {
        return Err(CaptureError::OutOfBounds);
    }

    let mut dpi_x = 0;
    let mut dpi_y = 0;
    unsafe { GetDpiForMonitor(handle, MDT_EFFECTIVE_DPI, &mut dpi_x, &mut dpi_y) }
        .map_err(|_| CaptureError::Unavailable)?;
    if !supported_dpi(dpi_x, dpi_y) { return Err(CaptureError::UnsupportedScale); }

    Ok(MonitorSelection {
        handle,
        crop_left: u32::try_from(x - bounds.left).map_err(|_| CaptureError::OutOfBounds)?,
        crop_top: u32::try_from(y - bounds.top).map_err(|_| CaptureError::OutOfBounds)?,
        width,
        height,
    })
}

fn supported_dpi(dpi_x: u32, dpi_y: u32) -> bool {
    dpi_x == dpi_y && matches!(dpi_x, 96 | 192)
}

fn capture_monitor_region(selection: MonitorSelection) -> Result<CapturedRegion, CaptureError> {
    unsafe { RoInitialize(RO_INIT_MULTITHREADED) }.map_err(|_| CaptureError::Unavailable)?;
    let _apartment = WinRtApartment;
    if !GraphicsCaptureSession::IsSupported().map_err(|_| CaptureError::Unsupported)? {
        return Err(CaptureError::Unsupported);
    }

    let interop = factory::<GraphicsCaptureItem, IGraphicsCaptureItemInterop>()
        .map_err(|_| CaptureError::Unavailable)?;
    let item: GraphicsCaptureItem = unsafe { interop.CreateForMonitor(selection.handle) }
        .map_err(|_| CaptureError::Unavailable)?;
    let item_size = item.Size().map_err(|_| CaptureError::Unavailable)?;
    if item_size.Width <= 0 || item_size.Height <= 0 { return Err(CaptureError::Unavailable); }

    let (device, context, winrt_device) = create_device()?;
    let pool = Direct3D11CaptureFramePool::CreateFreeThreaded(
        &winrt_device,
        DirectXPixelFormat::B8G8R8A8UIntNormalized,
        1,
        item_size,
    ).map_err(|_| CaptureError::Unavailable)?;
    let session = pool.CreateCaptureSession(&item).map_err(|_| CaptureError::Unavailable)?;
    let mut resources = CaptureResources { session, pool, frame_arrived_token: None };
    resources.session.SetIsCursorCaptureEnabled(false).map_err(|_| CaptureError::Unavailable)?;
    let (sender, receiver) = mpsc::sync_channel(1);
    let event_pool = resources.pool.clone();
    let handler = TypedEventHandler::new(move |_, _| {
        let frame = event_pool.TryGetNextFrame().map_err(|_| CaptureError::Unavailable);
        let _ = sender.try_send(frame);
        Ok(())
    });
    resources.frame_arrived_token = Some(resources.pool.FrameArrived(&handler).map_err(|_| CaptureError::Unavailable)?);
    resources.session.StartCapture().map_err(|_| CaptureError::Unavailable)?;
    let frame = receiver.recv_timeout(FRAME_TIMEOUT).map_err(|_| CaptureError::Timeout)??;

    let result = read_frame(&device, &context, &frame, &selection);
    let _ = frame.Close();
    result
}

struct CaptureResources {
    session: windows::Graphics::Capture::GraphicsCaptureSession,
    pool: Direct3D11CaptureFramePool,
    frame_arrived_token: Option<i64>,
}

impl Drop for CaptureResources {
    fn drop(&mut self) {
        if let Some(token) = self.frame_arrived_token.take() { let _ = self.pool.RemoveFrameArrived(token); }
        let _ = self.session.Close();
        let _ = self.pool.Close();
    }
}

struct WinRtApartment;

impl Drop for WinRtApartment {
    fn drop(&mut self) { unsafe { RoUninitialize(); } }
}

fn create_device() -> Result<(ID3D11Device, ID3D11DeviceContext, IDirect3DDevice), CaptureError> {
    let mut device = None;
    let mut context = None;
    unsafe {
        D3D11CreateDevice(
            None,
            D3D_DRIVER_TYPE_HARDWARE,
            HMODULE::default(),
            D3D11_CREATE_DEVICE_BGRA_SUPPORT,
            None,
            D3D11_SDK_VERSION,
            Some(&mut device),
            None,
            Some(&mut context),
        )
    }.map_err(|_| CaptureError::Unavailable)?;
    let device = device.ok_or(CaptureError::Unavailable)?;
    let context = context.ok_or(CaptureError::Unavailable)?;
    let dxgi: IDXGIDevice = device.cast().map_err(|_| CaptureError::Unavailable)?;
    let inspectable = unsafe { CreateDirect3D11DeviceFromDXGIDevice(&dxgi) }
        .map_err(|_| CaptureError::Unavailable)?;
    let winrt_device: IDirect3DDevice = inspectable.cast().map_err(|_| CaptureError::Unavailable)?;
    Ok((device, context, winrt_device))
}

fn read_frame(
    device: &ID3D11Device,
    context: &ID3D11DeviceContext,
    frame: &windows::Graphics::Capture::Direct3D11CaptureFrame,
    selection: &MonitorSelection,
) -> Result<CapturedRegion, CaptureError> {
    let surface = frame.Surface().map_err(|_| CaptureError::Unavailable)?;
    let access: IDirect3DDxgiInterfaceAccess = surface.cast().map_err(|_| CaptureError::Unavailable)?;
    let texture: ID3D11Texture2D = unsafe { access.GetInterface() }.map_err(|_| CaptureError::Unavailable)?;
    let mut source_desc = D3D11_TEXTURE2D_DESC::default();
    unsafe { texture.GetDesc(&mut source_desc); }

    let source_left = selection.crop_left;
    let source_top = selection.crop_top;
    if source_left.checked_add(selection.width).is_none_or(|right| right > source_desc.Width)
        || source_top.checked_add(selection.height).is_none_or(|bottom| bottom > source_desc.Height) {
        return Err(CaptureError::OutOfBounds);
    }

    let desc = D3D11_TEXTURE2D_DESC {
        Width: selection.width,
        Height: selection.height,
        MipLevels: 1,
        ArraySize: 1,
        Format: DXGI_FORMAT_B8G8R8A8_UNORM,
        SampleDesc: DXGI_SAMPLE_DESC { Count: 1, Quality: 0 },
        Usage: D3D11_USAGE_STAGING,
        BindFlags: 0,
        CPUAccessFlags: D3D11_CPU_ACCESS_READ.0 as u32,
        MiscFlags: 0,
    };
    let mut staging = None;
    unsafe { device.CreateTexture2D(&desc, None, Some(&mut staging)) }
        .map_err(|_| CaptureError::Unavailable)?;
    let staging = staging.ok_or(CaptureError::Unavailable)?;
    let source_box = D3D11_BOX {
        left: source_left,
        top: source_top,
        front: 0,
        right: source_left + selection.width,
        bottom: source_top + selection.height,
        back: 1,
    };
    let destination: ID3D11Resource = staging.cast().map_err(|_| CaptureError::Unavailable)?;
    let source: ID3D11Resource = texture.cast().map_err(|_| CaptureError::Unavailable)?;
    unsafe { context.CopySubresourceRegion(&destination, 0, 0, 0, 0, &source, 0, Some(&source_box)); }

    let mut mapped = D3D11_MAPPED_SUBRESOURCE::default();
    unsafe { context.Map(&destination, 0, D3D11_MAP_READ, 0, Some(&mut mapped)) }
        .map_err(|_| CaptureError::Unavailable)?;
    let rgba = copy_bgra_to_rgba(mapped, selection.width, selection.height);
    unsafe { context.Unmap(&destination, 0); }
    Ok(CapturedRegion { width: selection.width, height: selection.height, rgba })
}

fn copy_bgra_to_rgba(mapped: D3D11_MAPPED_SUBRESOURCE, width: u32, height: u32) -> Vec<u8> {
    let mut rgba = vec![0; width as usize * height as usize * 4];
    for row in 0..height as usize {
        let source = unsafe {
            std::slice::from_raw_parts((mapped.pData as *const u8).add(row * mapped.RowPitch as usize), width as usize * 4)
        };
        let destination = &mut rgba[row * width as usize * 4..(row + 1) * width as usize * 4];
        for (bgra, rgba_pixel) in source.chunks_exact(4).zip(destination.chunks_exact_mut(4)) {
            rgba_pixel.copy_from_slice(&[bgra[2], bgra[1], bgra[0], bgra[3]]);
        }
    }
    rgba
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bgra_rows_respect_pitch_and_become_rgba() {
        let bytes = [1_u8, 2, 3, 4, 9, 9, 9, 9, 5, 6, 7, 8, 9, 9, 9, 9];
        let mapped = D3D11_MAPPED_SUBRESOURCE { pData: bytes.as_ptr().cast_mut().cast(), RowPitch: 8, DepthPitch: 16 };
        assert_eq!(copy_bgra_to_rgba(mapped, 1, 2), [3, 2, 1, 4, 7, 6, 5, 8]);
    }

    #[test]
    fn only_verified_dpi_values_are_enabled() {
        assert!(supported_dpi(96, 96));
        assert!(supported_dpi(192, 192));
        assert!(!supported_dpi(120, 120));
        assert!(!supported_dpi(144, 144));
        assert!(!supported_dpi(168, 168));
        assert!(!supported_dpi(192, 96));
    }

    #[test]
    fn oversized_region_is_rejected_before_capture() {
        assert!(matches!(capture_region(0, 0, 4097, 4097), Err(CaptureError::TooLarge)));
    }

    #[test]
    fn png_is_encoded_in_memory() {
        let capture = CapturedRegion { width: 1, height: 1, rgba: vec![255, 0, 0, 255] };
        let bytes = encode_png(&capture).unwrap();
        assert_eq!(&bytes[..8], b"\x89PNG\r\n\x1a\n");
    }
}
