use std::ffi::{c_char, c_int, c_void};
use std::sync::OnceLock;

unsafe extern "C" {
    fn dlopen(file: *const c_char, flags: c_int) -> *mut c_void;
    fn dlsym(handle: *mut c_void, symbol: *const c_char) -> *mut c_void;
}

const RTLD_NEXT: *mut c_void = -1isize as *mut c_void;
const RTLD_LAZY: c_int = 1;

pub struct Orig {
    pub glCallList: unsafe extern "C" fn(u32),
}

pub struct Statefns {
    pub glPushAttrib: unsafe extern "C" fn(u32),
    pub glPopAttrib: unsafe extern "C" fn(),
    pub glDisable: unsafe extern "C" fn(u32),
    pub glDepthFunc: unsafe extern "C" fn(u32),
    pub glDepthMask: unsafe extern "C" fn(u8),
    pub glColor3f: unsafe extern "C" fn(f32, f32, f32),
    pub glGetIntegerv: unsafe extern "C" fn(u32, *mut i32),
}

pub struct Meshfns {
    pub glGenLists: unsafe extern "C" fn(i32) -> u32,
    pub glNewList: unsafe extern "C" fn(u32, u32),
    pub glEndList: unsafe extern "C" fn(),
    pub glBegin: unsafe extern "C" fn(u32),
    pub glEnd: unsafe extern "C" fn(),
    pub glVertex3f: unsafe extern "C" fn(f32, f32, f32),
    pub glNormal3f: unsafe extern "C" fn(f32, f32, f32),
    pub glPushMatrix: unsafe extern "C" fn(),
    pub glPopMatrix: unsafe extern "C" fn(),
    pub glScalef: unsafe extern "C" fn(f32, f32, f32),
}

static libgl: OnceLock<usize> = OnceLock::new();
static orig: OnceLock<Option<Orig>> = OnceLock::new();
static statefns: OnceLock<Option<Statefns>> = OnceLock::new();
static meshfns: OnceLock<Option<Meshfns>> = OnceLock::new();

fn handle() -> *mut c_void {
    *libgl.get_or_init(|| unsafe { dlopen(c"libGL.so.1".as_ptr(), RTLD_LAZY) as usize })
        as *mut c_void
}

fn find(name: &std::ffi::CStr) -> *mut c_void {
    unsafe {
        let direct = dlsym(RTLD_NEXT, name.as_ptr());
        if !direct.is_null() {
            return direct;
        }
        let h = handle();
        if h.is_null() {
            return std::ptr::null_mut();
        }
        dlsym(h, name.as_ptr())
    }
}

macro_rules! grab {
    ($name:literal) => {{
        let cname = match std::ffi::CStr::from_bytes_with_nul(concat!($name, "\0").as_bytes()) {
            Ok(c) => c,
            Err(_) => return None,
        };
        let p = find(cname);
        if p.is_null() {
            return None;
        }
        unsafe { std::mem::transmute(p) }
    }};
}

pub fn chain() -> Option<&'static Orig> {
    orig.get_or_init(|| Some(Orig { glCallList: grab!("glCallList") })).as_ref()
}

pub fn state() -> Option<&'static Statefns> {
    statefns
        .get_or_init(|| {
            Some(Statefns {
                glPushAttrib: grab!("glPushAttrib"),
                glPopAttrib: grab!("glPopAttrib"),
                glDisable: grab!("glDisable"),
                glDepthFunc: grab!("glDepthFunc"),
                glDepthMask: grab!("glDepthMask"),
                glColor3f: grab!("glColor3f"),
                glGetIntegerv: grab!("glGetIntegerv"),
            })
        })
        .as_ref()
}

pub fn mesh() -> Option<&'static Meshfns> {
    meshfns
        .get_or_init(|| {
            Some(Meshfns {
                glGenLists: grab!("glGenLists"),
                glNewList: grab!("glNewList"),
                glEndList: grab!("glEndList"),
                glBegin: grab!("glBegin"),
                glEnd: grab!("glEnd"),
                glVertex3f: grab!("glVertex3f"),
                glNormal3f: grab!("glNormal3f"),
                glPushMatrix: grab!("glPushMatrix"),
                glPopMatrix: grab!("glPopMatrix"),
                glScalef: grab!("glScalef"),
            })
        })
        .as_ref()
}
