use std::f32::consts::PI;
use std::sync::OnceLock;
use std::sync::atomic::{AtomicU32, Ordering};

use crate::glapi::{self, Meshfns};

const GL_QUAD_STRIP: u32 = 0x0008;
const GL_COMPILE: u32 = 0x1300;

const slices: u32 = 14;
const stacks: u32 = 10;
const taillinks: u32 = 22;

struct Blob {
    c: [f32; 3],
    r: [f32; 3],
}

const parts: [Blob; 15] = [
    Blob { c: [0.00, 0.00, 0.00], r: [1.00, 0.46, 0.42] },
    Blob { c: [0.62, 0.03, 0.00], r: [0.56, 0.43, 0.39] },
    Blob { c: [1.16, 0.05, 0.00], r: [0.42, 0.34, 0.32] },
    Blob { c: [1.52, -0.07, 0.00], r: [0.26, 0.19, 0.18] },
    Blob { c: [1.73, -0.09, 0.00], r: [0.09, 0.09, 0.09] },
    Blob { c: [1.06, 0.44, 0.21], r: [0.24, 0.27, 0.07] },
    Blob { c: [1.06, 0.44, -0.21], r: [0.24, 0.27, 0.07] },
    Blob { c: [0.64, -0.40, 0.27], r: [0.13, 0.28, 0.13] },
    Blob { c: [0.64, -0.40, -0.27], r: [0.13, 0.28, 0.13] },
    Blob { c: [0.70, -0.63, 0.27], r: [0.17, 0.08, 0.11] },
    Blob { c: [0.70, -0.63, -0.27], r: [0.17, 0.08, 0.11] },
    Blob { c: [-0.48, -0.36, 0.33], r: [0.24, 0.32, 0.19] },
    Blob { c: [-0.48, -0.36, -0.33], r: [0.24, 0.32, 0.19] },
    Blob { c: [-0.40, -0.65, 0.33], r: [0.21, 0.08, 0.12] },
    Blob { c: [-0.40, -0.65, -0.33], r: [0.21, 0.08, 0.12] },
];

const gearscales: [f32; 3] = [1.80, 0.90, 0.90];

static ratlist: AtomicU32 = AtomicU32::new(0);
static slots: [AtomicU32; 3] = [AtomicU32::new(0), AtomicU32::new(0), AtomicU32::new(0)];

pub struct Plan {
    pub list: u32,
    pub scale: f32,
    pub mesh: Option<&'static Meshfns>,
}

fn normalize(v: [f32; 3]) -> [f32; 3] {
    let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if len <= f32::EPSILON {
        return [0.0, 1.0, 0.0];
    }
    [v[0] / len, v[1] / len, v[2] / len]
}

unsafe fn blob(m: &Meshfns, b: &Blob) {
    for i in 0..stacks {
        let lo = (i as f32 / stacks as f32 - 0.5) * PI;
        let hi = ((i + 1) as f32 / stacks as f32 - 0.5) * PI;
        unsafe { (m.glBegin)(GL_QUAD_STRIP) };
        for j in 0..=slices {
            let phi = j as f32 / slices as f32 * PI * 2.0;
            for ring in [hi, lo] {
                let d = [ring.cos() * phi.cos(), ring.sin(), ring.cos() * phi.sin()];
                let n = normalize([d[0] / b.r[0], d[1] / b.r[1], d[2] / b.r[2]]);
                unsafe {
                    (m.glNormal3f)(n[0], n[1], n[2]);
                    (m.glVertex3f)(
                        b.c[0] + b.r[0] * d[0],
                        b.c[1] + b.r[1] * d[1],
                        b.c[2] + b.r[2] * d[2],
                    );
                }
            }
        }
        unsafe { (m.glEnd)() };
    }
}

fn tailblob(link: u32) -> Blob {
    let t = link as f32 / (taillinks - 1) as f32;
    Blob {
        c: [-1.02 - 1.55 * t, 0.12 + 0.80 * t * t, 0.0],
        r: [0.13 - 0.08 * t, 0.13 - 0.08 * t, 0.13 - 0.08 * t],
    }
}

unsafe fn bake(m: &Meshfns) -> u32 {
    let list = unsafe { (m.glGenLists)(1) };
    if list == 0 {
        return 0;
    }
    unsafe {
        (m.glNewList)(list, GL_COMPILE);
        for part in &parts {
            blob(m, part);
        }
        for link in 0..taillinks {
            blob(m, &tailblob(link));
        }
        (m.glEndList)();
    }
    list
}

fn slotfor(list: u32) -> usize {
    for (i, slot) in slots.iter().enumerate() {
        if slot.load(Ordering::Relaxed) == list {
            return i;
        }
    }
    for (i, slot) in slots.iter().enumerate() {
        if slot.compare_exchange(0, list, Ordering::Relaxed, Ordering::Relaxed).is_ok() {
            return i;
        }
    }
    slots.len() - 1
}

fn wanted() -> bool {
    static want: OnceLock<bool> = OnceLock::new();
    *want.get_or_init(|| std::env::var("GLXRAT_GEARS").map(|v| v == "0").unwrap_or(true))
}

pub unsafe fn plan(list: u32) -> Plan {
    let gears = Plan { list, scale: 1.0, mesh: None };
    if !wanted() {
        return gears;
    }
    let Some(m) = glapi::mesh() else { return gears };

    let mut rat = ratlist.load(Ordering::Relaxed);
    if rat == 0 {
        rat = unsafe { bake(m) };
        ratlist.store(rat, Ordering::Relaxed);
    }
    if rat == 0 {
        return gears;
    }

    Plan { list: rat, scale: gearscales[slotfor(list)], mesh: Some(m) }
}
