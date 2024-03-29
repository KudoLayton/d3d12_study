use windows::{
    core::*, 
    Win32::{
        Foundation::*,
        UI::WindowsAndMessaging::*,
        Graphics::{
            Direct3D::*,
            Direct3D12::*,
            Dxgi::{*, Common::*},
            Gdi::*
        },
    },
};
use std::sync::RwLock;
use const_default::ConstDefault;

pub static GRAPHICS: RwLock<Graphics> = RwLock::new(Graphics::DEFAULT);
static BACK_BUFFER_FORMAT: DXGI_FORMAT = DXGI_FORMAT_R8G8B8A8_UNORM;

pub unsafe extern "system" fn wnd_graphics_proc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    match msg {
        WM_DESTROY => {
            PostQuitMessage(0);
            LRESULT::default()
        },
        WM_PAINT => {
            let mut paint_struct = PAINTSTRUCT::default();
            let hdc = BeginPaint(hwnd, &mut paint_struct);
            let rect = paint_struct.rcPaint;
            let graphics = GRAPHICS.write().unwrap();
            EndPaint(hwnd, &paint_struct);
            LRESULT::default()
        },
        _ => {
            DefWindowProcW(hwnd, msg, wparam, lparam)
        }
    }
}

//set default value of struct
#[derive(ConstDefault)]
pub struct Graphics {
    width: u32,
    height: u32,
    debug_controller: Option<ID3D12Debug>,
    dxgi_factory: Option<IDXGIFactory4>,
    d3d_device: Option<ID3D12Device>,
    descriptor_sizes: Option<DescriptorSizes>,
    fence: Option<ID3D12Fence>,
    quality_4x_msaa: u32,
    state_4x_msaa: bool,
    command_queue: Option<ID3D12CommandQueue>,
    direct_cmd_list_alloc: Option<ID3D12CommandAllocator>,
    command_list: Option<ID3D12GraphicsCommandList>,
    swapchain: Option<IDXGISwapChain>,
    rtv_heap: Option<ID3D12DescriptorHeap>,
    dsv_heap: Option<ID3D12DescriptorHeap>,
}

struct DescriptorSizes {
    cbv_srv_uav: u32,
    rtv: u32,
    dsv: u32,
}

impl Graphics {
    pub fn init(&mut self, hwnd:HWND, width: u32, height: u32) -> Result<()> {
        unsafe {
            self.width = width;
            self.height = height;
            self.create_device()?;
            self.create_fence_and_get_descriptor_sizes()?;
            self.check_4x_msaa()?;
            self.create_command_objects()?;
            self.create_swapchain(hwnd)?;
            self.create_rtv_and_dsv_descriptor_heaps()?;
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

    unsafe fn check_4x_msaa(&mut self) -> Result<()> {
        let ms_quality_levels = D3D12_FEATURE_DATA_MULTISAMPLE_QUALITY_LEVELS {
            Format: BACK_BUFFER_FORMAT,
            SampleCount: 4,
            Flags: D3D12_MULTISAMPLE_QUALITY_LEVELS_FLAG_NONE,
            NumQualityLevels: 0,
        };

        if let Some(device) = self.d3d_device.as_ref() { 
            device.CheckFeatureSupport(
                D3D12_FEATURE_MULTISAMPLE_QUALITY_LEVELS,
                &ms_quality_levels as *const _ as *mut _,
                std::mem::size_of::<D3D12_FEATURE_DATA_MULTISAMPLE_QUALITY_LEVELS>() as u32,
            )?;
            self.quality_4x_msaa = ms_quality_levels.NumQualityLevels;
            assert!(self.quality_4x_msaa > 0, "Unexpected MSAA quality level.");
        }
        Ok(())
    }

    unsafe fn create_command_objects(&mut self) -> Result<()> {
        if let Some(device) = self.d3d_device.as_ref() {
            let queue_desc = D3D12_COMMAND_QUEUE_DESC {
                Type: D3D12_COMMAND_LIST_TYPE_DIRECT,
                Flags: D3D12_COMMAND_QUEUE_FLAG_NONE,
                ..Default::default()
            };
            self.command_queue = Some(device.CreateCommandQueue(&queue_desc)?);
            self.direct_cmd_list_alloc = Some(device.CreateCommandAllocator(D3D12_COMMAND_LIST_TYPE_DIRECT)?);
            self.command_list = Some(device.CreateCommandList(
                0,
                D3D12_COMMAND_LIST_TYPE_DIRECT,
                self.direct_cmd_list_alloc.as_ref().unwrap(),
                None,
            )?);
            // 처음에는 닫아야 한다. 최초 reset을 진행하려면 닫혀 있어야 한다.
            self.command_list.as_ref().unwrap().Close()?;
        }
        Ok(())
    }

    unsafe fn create_swapchain(&mut self, hwnd: HWND) -> Result<()> {
        let buffer_desc = DXGI_MODE_DESC {
            Width: self.width,
            Height: self.height,
            RefreshRate: DXGI_RATIONAL {
                Numerator: 60,
                Denominator: 1,
            },
            Format: BACK_BUFFER_FORMAT,
            ScanlineOrdering: DXGI_MODE_SCANLINE_ORDER_UNSPECIFIED,
            Scaling: DXGI_MODE_SCALING_UNSPECIFIED,
            ..Default::default()
        };

        let sample_desc = match self.state_4x_msaa {
            true => DXGI_SAMPLE_DESC {
                Count: 4,
                Quality: self.quality_4x_msaa - 1,
            },
            false => DXGI_SAMPLE_DESC {
                Count: 1,
                Quality: 0,
            },
        };

        let swapchain_desc = DXGI_SWAP_CHAIN_DESC {
            BufferDesc: buffer_desc,
            SampleDesc: sample_desc,
            BufferUsage: DXGI_USAGE_RENDER_TARGET_OUTPUT,
            BufferCount: 2,
            OutputWindow: hwnd,
            Windowed: TRUE,
            SwapEffect: DXGI_SWAP_EFFECT_FLIP_DISCARD,
            Flags: DXGI_SWAP_CHAIN_FLAG_ALLOW_MODE_SWITCH.0 as u32,
        };

        if let Some(dxgi_factory) = self.dxgi_factory.as_ref() {
            dxgi_factory.CreateSwapChain(self.command_queue.as_ref().expect("no command queue when create swapchain"), &swapchain_desc as *const _, &self.swapchain as *const _ as *mut _).ok()?;
        }
        Ok(())
    }

    unsafe fn create_rtv_and_dsv_descriptor_heaps(&mut self) -> Result<()> {
        let rtv_heap_desc = D3D12_DESCRIPTOR_HEAP_DESC {
            Type: D3D12_DESCRIPTOR_HEAP_TYPE_RTV,
            NumDescriptors: 2,
            Flags: D3D12_DESCRIPTOR_HEAP_FLAG_NONE,
            ..Default::default()
        };

        let dsv_heap_desc = D3D12_DESCRIPTOR_HEAP_DESC {
            Type: D3D12_DESCRIPTOR_HEAP_TYPE_DSV,
            NumDescriptors: 1,
            Flags: D3D12_DESCRIPTOR_HEAP_FLAG_NONE,
            ..Default::default()
        };

        if let Some(device) = self.d3d_device.as_ref(){
            self.rtv_heap = Some(device.CreateDescriptorHeap::<ID3D12DescriptorHeap>(&rtv_heap_desc as *const _)?);
            self.dsv_heap = Some(device.CreateDescriptorHeap::<ID3D12DescriptorHeap>(&dsv_heap_desc as *const _)?);
        };
        Ok(())
    }
}
