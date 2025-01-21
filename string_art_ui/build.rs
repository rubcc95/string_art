fn main() {
    // Solo compilar los recursos en Windows
    #[cfg(target_os = "windows")]
    {
        let mut res = winres::WindowsResource::new();
        res.set_icon("assets/icon.ico");
        res.compile().expect("Fallo al compilar los recursos");
    }
}