use windows::{
    core::*, 
    Win32::{
        Foundation::*,
        UI::WindowsAndMessaging::*,
        System::LibraryLoader::*,
        Graphics::Gdi::*,
    },
};

unsafe extern "system" fn wnd_proc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
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

fn main() -> Result<()> {
    unsafe{
        let hinstance = GetModuleHandleA(None)?;
        let icon = LoadIconW(None, IDI_APPLICATION)?;
        let cursor = LoadCursorW(None, IDC_ARROW)?;
        let class_name = w!("d3d12Sample");
        let class = WNDCLASSEXW{
            cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
            style: CS_HREDRAW | CS_VREDRAW,
            hInstance: hinstance.into(),
            hIcon: icon,
            hIconSm: icon,
            hCursor: cursor,
            lpszClassName: class_name,
            lpfnWndProc: Some(wnd_proc),
            ..Default::default()
        };

        if RegisterClassExW(&class) == 0 {
            GetLastError()?;
        }

        let hwnd = CreateWindowExW(
            WS_EX_RIGHTSCROLLBAR,
            class_name,
            w!("d3d12 Sample"),
            WS_OVERLAPPEDWINDOW,
            CW_USEDEFAULT, CW_USEDEFAULT, 
            800, 600,
            None,
            None,
            hinstance,
            None
        );

        if hwnd.0 == 0 {
            GetLastError()?;
        }

        ShowWindow(hwnd, SW_SHOWDEFAULT);
        UpdateWindow(hwnd);

        loop {
            let mut msg = MSG::default();
            if PeekMessageW(&mut msg, None, 0, 0, PM_REMOVE).into() {
                match msg.message {
                    WM_QUIT => break,
                    _ => {
                        TranslateMessage(&msg);
                        DispatchMessageW(&msg);
                    }
                }
            } 
        }
    }
    Ok(())
}
