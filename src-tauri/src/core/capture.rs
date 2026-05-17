use windows::core::*;
use windows::Win32::Foundation::*;
use windows::Win32::Graphics::Direct3D::*;
use windows::Win32::Graphics::Direct3D11::*;
use windows::Win32::Graphics::Dxgi::*;
use windows::Graphics::Capture::*;
use windows::Graphics::DirectX::Direct3D11::*;
use windows::Win32::System::WinRT::*;
use windows::Win32::System::WinRT::Direct3D11::*;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct WgcGrabber {
    _device: ID3D11Device,
    _context: ID3D11DeviceContext,
    frame_pool: Direct3D11CaptureFramePool,
    session: GraphicsCaptureSession,
    latest_frame: Arc<Mutex<Option<Vec<u8>>>>,
    width: u32,
    height: u32,
}

impl WgcGrabber {
    pub fn clone_instance(&self) -> Self {
        self.clone()
    }
    
    pub fn new(hwnd: HWND) -> Result<Self> {
        unsafe {
            let mut device: Option<ID3D11Device> = None;
            let mut context: Option<ID3D11DeviceContext> = None;
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
            )?;
            let device = device.unwrap();
            let context = context.unwrap();

            let interop = factory::<GraphicsCaptureItem, IGraphicsCaptureItemInterop>()?;
            let item: GraphicsCaptureItem = interop.CreateForWindow(hwnd)?;
            let size = item.Size()?;
            let width = size.Width as u32;
            let height = size.Height as u32;

            let dxgi_device: IDXGIDevice = device.cast()?;
            let d3d_winrt_device: IDirect3DDevice = CreateDirect3D11DeviceFromDXGIDevice(&dxgi_device)?.cast()?;

            let frame_pool = Direct3D11CaptureFramePool::CreateFreeThreaded(
                &d3d_winrt_device,
                windows::Graphics::DirectX::DirectXPixelFormat::B8G8R8A8UIntNormalized,
                2,
                size,
            )?;
            let session = frame_pool.CreateCaptureSession(&item)?;

            let latest_frame = Arc::new(Mutex::new(None));
            let latest_frame_clone = latest_frame.clone();
            
            let cb_context = context.clone();

            let staging_desc = D3D11_TEXTURE2D_DESC {
                Width: width,
                Height: height,
                MipLevels: 1,
                ArraySize: 1,
                Format: windows::Win32::Graphics::Dxgi::Common::DXGI_FORMAT_B8G8R8A8_UNORM,
                SampleDesc: windows::Win32::Graphics::Dxgi::Common::DXGI_SAMPLE_DESC { Count: 1, Quality: 0 },
                Usage: D3D11_USAGE_STAGING,
                BindFlags: 0,
                CPUAccessFlags: D3D11_CPU_ACCESS_READ.0 as u32,
                MiscFlags: 0,
            };
            let mut staging_texture: Option<ID3D11Texture2D> = None;
            device.CreateTexture2D(&staging_desc, None, Some(&mut staging_texture))?;
            let staging_texture = staging_texture.unwrap();

            frame_pool.FrameArrived(&windows::Foundation::TypedEventHandler::<Direct3D11CaptureFramePool, IInspectable>::new(
                move |pool, _| {
                    if let Some(pool_ref) = (*pool).as_ref() {
                        if let Ok(frame) = pool_ref.TryGetNextFrame() {
                                if let Ok(surface) = frame.Surface() {
                                    let access: IDirect3DDxgiInterfaceAccess = surface.cast().unwrap();
                                    if let Ok(texture) = access.GetInterface::<ID3D11Texture2D>() {
                                        cb_context.CopyResource(&staging_texture, &texture);
                                        
                                        let mut mapped = D3D11_MAPPED_SUBRESOURCE::default();
                                        if cb_context.Map(&staging_texture, 0, D3D11_MAP_READ, 0, Some(&mut mapped)).is_ok() {
                                            let row_pitch = mapped.RowPitch as usize;
                                            
                                            let mut lock = latest_frame_clone.lock().unwrap();
                                            let data = lock.get_or_insert_with(|| Vec::with_capacity((width * height * 4) as usize));
                                            data.clear();

                                            let src_ptr = mapped.pData as *const u8;
                                            for y in 0..height as usize {
                                                data.extend_from_slice(std::slice::from_raw_parts(src_ptr.add(y * row_pitch), (width * 4) as usize));
                                            }
                                            cb_context.Unmap(&staging_texture, 0);
                                        }
                                    }
                                }
                        }
                    }
                    Ok(())
                },
            ))?;

            session.StartCapture()?;

            Ok(Self {
                _device: device,
                _context: context,
                frame_pool,
                session,
                latest_frame,
                width,
                height,
            })
        }
    }

    pub fn capture_frame(&self) -> Result<Vec<u8>> {
        {
            let lock = self.latest_frame.lock().unwrap();
            if let Some(data) = lock.as_ref() {
                return Ok(data.clone());
            }
        }

        for _ in 0..20 {
            std::thread::sleep(std::time::Duration::from_millis(50));
            let lock = self.latest_frame.lock().unwrap();
            if let Some(data) = lock.as_ref() {
                return Ok(data.clone());
            }
        }
        Err(Error::new(HRESULT(0x80004005u32 as i32), "Timeout: Không nhận được khung hình từ GPU."))
    }

    pub fn get_resolution(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    pub fn close(&self) {
        if let Ok(closable) = self.session.cast::<windows::Foundation::IClosable>() {
            let _ = closable.Close();
        }
        if let Ok(closable) = self.frame_pool.cast::<windows::Foundation::IClosable>() {
            let _ = closable.Close();
        }
    }
}

fn factory<T: RuntimeName, I: Interface>() -> Result<I> {
    unsafe {
        let hstr = HSTRING::from(T::NAME);
        RoGetActivationFactory::<I>(&hstr)
    }
}

#[repr(transparent)]
#[derive(Clone)]
#[allow(non_snake_case)]
pub struct IGraphicsCaptureItemInterop(IUnknown);

#[allow(non_snake_case)]
impl IGraphicsCaptureItemInterop {
    pub unsafe fn CreateForWindow<T: Interface>(&self, window: HWND) -> Result<T> {
        let mut result = std::ptr::null_mut();
        (Interface::vtable(self).CreateForWindow)(std::mem::transmute_copy(self), window, &T::IID, &mut result).ok()?;
        Ok(T::from_raw(result))
    }
}
unsafe impl Interface for IGraphicsCaptureItemInterop {
    type Vtable = IGraphicsCaptureItemInterop_Vtbl;
    const IID: GUID = GUID::from_u128(0x3628e81b_3cac_4c60_b7f4_23ce0e0c3356);
}
#[repr(C)]
#[allow(non_snake_case)]
pub struct IGraphicsCaptureItemInterop_Vtbl {
    pub base: IUnknown_Vtbl,
    pub CreateForWindow: unsafe extern "system" fn(
        this: *mut std::ffi::c_void,
        window: HWND,
        riid: *const GUID,
        result: *mut *mut std::ffi::c_void,
    ) -> HRESULT,
}
