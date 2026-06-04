/// Minimal FFI bindings to gtk4-layer-shell loaded dynamically at runtime via dlopen.
/// No Rust wrapper crate is used because the available crates conflict with
/// our gtk4 0.9 / libadwaita 0.7 dependency versions.
use gtk4::glib::translate::ToGlibPtr;
use std::ffi::{c_void, CString};
use std::os::raw::{c_char, c_int, c_uint};
use std::sync::OnceLock;

type GtkWindow = c_void;

// Function pointer signatures for gtk4-layer-shell
type GtkLayerIsSupported = unsafe extern "C" fn() -> c_int;
type GtkLayerInitForWindow = unsafe extern "C" fn(window: *mut GtkWindow);
type GtkLayerSetLayer = unsafe extern "C" fn(window: *mut GtkWindow, layer: c_uint);
type GtkLayerSetAnchor = unsafe extern "C" fn(window: *mut GtkWindow, edge: c_uint, anchor_to_edge: c_int);
type GtkLayerSetExclusiveZone = unsafe extern "C" fn(window: *mut GtkWindow, exclusive_zone: c_int);
type GtkLayerSetKeyboardMode = unsafe extern "C" fn(window: *mut GtkWindow, mode: c_uint);
type GtkLayerSetMargin = unsafe extern "C" fn(window: *mut GtkWindow, edge: c_uint, margin_size: c_int);

struct LibGtk4LayerShell {
    _handle: *mut c_void,
    is_supported: GtkLayerIsSupported,
    init_for_window: GtkLayerInitForWindow,
    set_layer: GtkLayerSetLayer,
    set_anchor: GtkLayerSetAnchor,
    set_exclusive_zone: GtkLayerSetExclusiveZone,
    set_keyboard_mode: GtkLayerSetKeyboardMode,
    set_margin: GtkLayerSetMargin,
}

unsafe impl Send for LibGtk4LayerShell {}
unsafe impl Sync for LibGtk4LayerShell {}

extern "C" {
    fn dlopen(filename: *const c_char, flag: c_int) -> *mut c_void;
    fn dlsym(handle: *mut c_void, symbol: *const c_char) -> *mut c_void;
    #[allow(dead_code)]
    fn dlclose(handle: *mut c_void) -> c_int;
}

const RTLD_LAZY: c_int = 1;

fn get_library() -> Option<&'static LibGtk4LayerShell> {
    static LIB: OnceLock<Option<LibGtk4LayerShell>> = OnceLock::new();
    LIB.get_or_init(|| {
        unsafe {
            let lib_name = CString::new("libgtk4-layer-shell.so.0").ok()?;
            let handle = dlopen(lib_name.as_ptr(), RTLD_LAZY);
            if handle.is_null() {
                return None;
            }

            let load_sym = |name: &str| -> Option<*mut c_void> {
                let sym_name = CString::new(name).ok()?;
                let sym = dlsym(handle, sym_name.as_ptr());
                if sym.is_null() {
                    None
                } else {
                    Some(sym)
                }
            };

            let is_supported: GtkLayerIsSupported = std::mem::transmute(load_sym("gtk_layer_is_supported")?);
            let init_for_window: GtkLayerInitForWindow = std::mem::transmute(load_sym("gtk_layer_init_for_window")?);
            let set_layer: GtkLayerSetLayer = std::mem::transmute(load_sym("gtk_layer_set_layer")?);
            let set_anchor: GtkLayerSetAnchor = std::mem::transmute(load_sym("gtk_layer_set_anchor")?);
            let set_exclusive_zone: GtkLayerSetExclusiveZone = std::mem::transmute(load_sym("gtk_layer_set_exclusive_zone")?);
            let set_keyboard_mode: GtkLayerSetKeyboardMode = std::mem::transmute(load_sym("gtk_layer_set_keyboard_mode")?);
            let set_margin: GtkLayerSetMargin = std::mem::transmute(load_sym("gtk_layer_set_margin")?);

            Some(LibGtk4LayerShell {
                _handle: handle,
                is_supported,
                init_for_window,
                set_layer,
                set_anchor,
                set_exclusive_zone,
                set_keyboard_mode,
                set_margin,
            })
        }
    }).as_ref()
}

/// Initialise the window as a Wayland layer-shell overlay surface.
///
/// - Anchors to the TOP edge only (no left/right) → compositor centres it horizontally
/// - 60 px top margin places it just below the GNOME top panel
/// - Keyboard mode ON_DEMAND so focus is captured only while the window is visible
/// - Falls back silently on X11 or compositors without wlr-layer-shell support
pub fn setup_layer_shell(window: &libadwaita::ApplicationWindow) {
    let lib = match get_library() {
        Some(l) => l,
        None => {
            eprintln!("[spear] libgtk4-layer-shell.so.0 not found — falling back to normal window");
            return;
        }
    };

    let supported = unsafe { (lib.is_supported)() };
    if supported == 0 {
        eprintln!("[spear] gtk4-layer-shell not supported — falling back to normal window");
        return;
    }

    // AdwApplicationWindow is a subclass of GtkWindow, so the pointer cast is safe.
    let raw: *mut GtkWindow = {
        use gtk4::glib::object::Cast;
        let gtk_win = window.upcast_ref::<gtk4::Window>();
        <gtk4::Window as ToGlibPtr<'_, *mut gtk4::ffi::GtkWindow>>::to_glib_none(gtk_win).0
            as *mut GtkWindow
    };

    unsafe {
        (lib.init_for_window)(raw);

        // TOP layer: above normal windows, below lock-screen / screensaver
        (lib.set_layer)(raw, 3); // GTK_LAYER_SHELL_LAYER_TOP = 3

        // Anchor top only → centred horizontally by compositor
        (lib.set_anchor)(raw, 2, 1); // GTK_LAYER_SHELL_EDGE_TOP = 2, anchor = 1
        (lib.set_anchor)(raw, 3, 0); // GTK_LAYER_SHELL_EDGE_BOTTOM = 3, anchor = 0
        (lib.set_anchor)(raw, 0, 0); // GTK_LAYER_SHELL_EDGE_LEFT = 0, anchor = 0
        (lib.set_anchor)(raw, 1, 0); // GTK_LAYER_SHELL_EDGE_RIGHT = 1, anchor = 0

        // No exclusive zone — other windows are not pushed away
        (lib.set_exclusive_zone)(raw, 0);

        // Grab keyboard on demand
        (lib.set_keyboard_mode)(raw, 2); // GTK_LAYER_SHELL_KEYBOARD_MODE_ON_DEMAND = 2

        // 60 px gap from the top so we sit just below the GNOME panel
        (lib.set_margin)(raw, 2, 60); // GTK_LAYER_SHELL_EDGE_TOP = 2, margin = 60
    }
}
