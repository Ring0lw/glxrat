use crate::glapi::{Orig, Statefns};
use crate::ratmesh::Plan;

const GL_CURRENT_BIT: u32 = 0x0000_0001;
const GL_LIGHTING_BIT: u32 = 0x0000_0040;
const GL_DEPTH_BUFFER_BIT: u32 = 0x0000_0100;
const GL_ENABLE_BIT: u32 = 0x0000_2000;

const GL_LESS: u32 = 0x0201;
const GL_GREATER: u32 = 0x0204;

const GL_LIST_INDEX: u32 = 0x0B33;
const GL_CULL_FACE: u32 = 0x0B44;
const GL_LIGHTING: u32 = 0x0B50;
const GL_COLOR_MATERIAL: u32 = 0x0B57;
const GL_FOG: u32 = 0x0B60;
const GL_DEPTH_FUNC: u32 = 0x0B74;
const GL_TEXTURE_2D: u32 = 0x0DE1;

const GL_FALSE: u8 = 0;
const GL_TRUE: u8 = 1;

const hiddencolor: [f32; 3] = [0.90, 0.08, 0.08];
const visiblecolor: [f32; 3] = [0.10, 1.00, 0.20];

pub unsafe fn iscompiling(s: &Statefns) -> bool {
    let mut list: i32 = 0;
    unsafe { (s.glGetIntegerv)(GL_LIST_INDEX, &mut list) };
    list != 0
}

unsafe fn emit(o: &Orig, p: &Plan) {
    unsafe {
        match p.mesh {
            Some(m) => {
                (m.glPushMatrix)();
                (m.glScalef)(p.scale, p.scale, p.scale);
                (o.glCallList)(p.list);
                (m.glPopMatrix)();
            }
            None => (o.glCallList)(p.list),
        }
    }
}

pub unsafe fn drawlist(o: &Orig, s: &Statefns, p: &Plan) {
    unsafe {
        let mut depthfunc: i32 = GL_LESS as i32;
        (s.glGetIntegerv)(GL_DEPTH_FUNC, &mut depthfunc);

        (s.glPushAttrib)(GL_ENABLE_BIT | GL_DEPTH_BUFFER_BIT | GL_CURRENT_BIT | GL_LIGHTING_BIT);
        (s.glDisable)(GL_LIGHTING);
        (s.glDisable)(GL_COLOR_MATERIAL);
        (s.glDisable)(GL_TEXTURE_2D);
        (s.glDisable)(GL_FOG);
        (s.glDisable)(GL_CULL_FACE);

        (s.glDepthFunc)(GL_GREATER);
        (s.glDepthMask)(GL_FALSE);
        (s.glColor3f)(hiddencolor[0], hiddencolor[1], hiddencolor[2]);
        emit(o, p);

        (s.glDepthFunc)(depthfunc as u32);
        (s.glDepthMask)(GL_TRUE);
        (s.glColor3f)(visiblecolor[0], visiblecolor[1], visiblecolor[2]);
        emit(o, p);

        (s.glPopAttrib)();
    }
}
