pub mod winapi_helper;
pub mod graphics;

use windows::core::*;
use winapi_helper::*;
use graphics::*;

fn main() -> Result<()> {
    unsafe{
        let class_name = w!("d3d12Sample");
        let window_title = w!("d3d12 Sample");
        create_wnd_class(&class_name, Some(graphics::wnd_graphics_proc))?;
        GRAPHICS.write().unwrap().init()?;
        let hwnd = create_wnd(&class_name, &window_title, 800, 600)?;
        run_wnd(&hwnd)?;
    }
    Ok(())
}
