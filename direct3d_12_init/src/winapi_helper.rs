use windows::{
    core::*, 
    Win32::{
        Foundation::*,
        UI::WindowsAndMessaging::*,
        System::LibraryLoader::*,
        Graphics::Gdi::*,
    },
};

#[cfg(test)]
mod tests {
    use windows::core::*;
    #[test]
    fn test_pcwstr_empty_check() {
        let empty = w!("");
        unsafe{
            assert!(empty.as_wide().len() <= 0);
        }
    }
}

pub unsafe extern "system" fn wnd_proc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
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

pub unsafe fn create_wnd_class(class_name: &PCWSTR, wnd_proc: WNDPROC) -> Result<()>{
    let hinstance = GetModuleHandleA(None)?;
    let icon = LoadIconW(None, IDI_APPLICATION)?;
    let cursor = LoadCursorW(None, IDC_ARROW)?;
    let class = WNDCLASSEXW{
        cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
        style: CS_HREDRAW | CS_VREDRAW,
        hInstance: hinstance.into(),
        hIcon: icon,
        hIconSm: icon,
        hCursor: cursor,
        lpszClassName: *class_name,
        lpfnWndProc: wnd_proc,
        ..Default::default()
    };

    if RegisterClassExW(&class) == 0 {
        GetLastError()?;
    }
    Ok(())
}

pub unsafe fn create_wnd(class_name: &PCWSTR, window_title: &PCWSTR, width: i32, height: i32) -> Result<HWND>{
    let hinstance = GetModuleHandleA(None)?;
    let hwnd = CreateWindowExW(
        WS_EX_RIGHTSCROLLBAR,
        *class_name,
        *window_title,
        WS_OVERLAPPEDWINDOW,
        CW_USEDEFAULT, CW_USEDEFAULT,
        width, height,
        None,
        None,
        hinstance,
        None
    );

    if hwnd.0 == 0 {
        GetLastError()?;
    }
    Ok(hwnd)
}

pub unsafe fn run_wnd(hwnd: &HWND) -> Result<()> {
    ShowWindow(*hwnd, SW_SHOWDEFAULT);
    UpdateWindow(*hwnd);

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
    Ok(())
}
