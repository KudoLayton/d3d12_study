use windows::{
    core::*, 
    Win32::{
        Foundation::*,
        UI::WindowsAndMessaging::*,
        Graphics::{
            Direct3D::*,
            Direct3D12::*,
            Dxgi::*,
            Gdi::*
        },
    },
};
use std::sync::RwLock;
use const_default::ConstDefault;

pub static GRAPHICS: RwLock<Graphics> = RwLock::new(Graphics::DEFAULT);

pub unsafe extern "system" fn wnd_graphics_proc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    match msg {
        WM_DESTROY => {
            unsafe{
                PostQuitMessage(0);
                LRESULT::default()
            }
        },
        WM_PAINT => {
            unsafe{
                let mut paint_struct = PAINTSTRUCT::default();
                let hdc = BeginPaint(hwnd, &mut paint_struct);
                let rect = paint_struct.rcPaint;
                GRAPHICS.write().unwrap();
                EndPaint(hwnd, &paint_struct);
                LRESULT::default()
            }
        },
        _ => {
            unsafe{
                DefWindowProcW(hwnd, msg, wparam, lparam)
            }
        }
    }
}

//set default value of struct
#[derive(ConstDefault)]
pub struct Graphics {
    debug_controller: Option<ID3D12Debug>,
    dxgi_factory: Option<IDXGIFactory4>,
    d3d_device: Option<ID3D12Device>,
    descriptor_sizes: Option<DescriptorSizes>,
    fence: Option<ID3D12Fence>,
}

struct DescriptorSizes {
    cbv_srv_uav: u32,
    rtv: u32,
    dsv: u32,
}

impl Graphics {
    pub fn init(&mut self) -> Result<()> {
        unsafe {
            self.create_device()?;
            self.create_fence_and_get_descriptor_sizes()?;
        }
        Ok(())
    }

    unsafe fn create_device(&mut self) -> Result<()> {
        // Create debug layer
        if cfg!(debug_assertions){
            D3D12GetDebugInterface(&mut self.debug_controller)?;
            self.debug_controller.as_ref().unwrap().EnableDebugLayer();
        }

        self.dxgi_factory = Some(CreateDXGIFactory1()?);

        if let Err(_) = D3D12CreateDevice(
            None,
            D3D_FEATURE_LEVEL_11_0,
            &mut self.d3d_device,
        ) {
            // If failed, try to create WARP device (software renderer)
            let warp_adapter: Option<IDXGIAdapter> = Some(self.dxgi_factory.as_ref().unwrap().EnumWarpAdapter()?);
            D3D12CreateDevice(
                warp_adapter.as_ref().unwrap(),
                D3D_FEATURE_LEVEL_11_0,
                &mut self.d3d_device,
            )?;
        }

        Ok(())
    }

    unsafe fn create_fence_and_get_descriptor_sizes(&mut self) -> Result<()> {
        if let Some(device) = self.d3d_device.as_ref() {
            self.fence = Some(device.CreateFence(0, D3D12_FENCE_FLAG_NONE)?);
            self.descriptor_sizes = Some(
                DescriptorSizes {
                    cbv_srv_uav: device.GetDescriptorHandleIncrementSize(D3D12_DESCRIPTOR_HEAP_TYPE_CBV_SRV_UAV),
                    rtv: device.GetDescriptorHandleIncrementSize(D3D12_DESCRIPTOR_HEAP_TYPE_RTV),
                    dsv: device.GetDescriptorHandleIncrementSize(D3D12_DESCRIPTOR_HEAP_TYPE_DSV),
                }
            );
                    
        }
        Ok(())
    }
}
