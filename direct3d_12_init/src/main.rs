pub mod winapi_helper;
pub mod graphics;

use windows::core::*;
use winapi_helper::*;
use graphics::*;

static WIDTH: u32 = 800;
static HEIGHT: u32 = 600;

fn main() -> Result<()> {
    unsafe{
        let class_name = w!("d3d12Sample");
        let window_title = w!("d3d12 Sample");
        create_wnd_class(&class_name, Some(graphics::wnd_graphics_proc))?;
        let hwnd = create_wnd(&class_name, &window_title, WIDTH as i32, HEIGHT as i32)?;
        GRAPHICS.write().unwrap().init(hwnd, WIDTH, HEIGHT)?;
        run_wnd(&hwnd)?;
    }
    Ok(())
}
