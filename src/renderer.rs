#![allow(unused_macros)]
macro_rules! CUBE_VERT {
    () => (vec![
        V4::new(-0.5, -0.5, -0.5, -0.5),
        V4::new( 0.5, -0.5, -0.5, -0.5),
        V4::new( 0.5, -0.5,  0.5, -0.5),
        V4::new(-0.5, -0.5,  0.5, -0.5),

        V4::new(-0.5,  0.5, -0.5, -0.5),
        V4::new( 0.5,  0.5, -0.5, -0.5),
        V4::new( 0.5,  0.5,  0.5, -0.5),
        V4::new(-0.5,  0.5,  0.5, -0.5),

        V4::new(-0.5, -0.5, -0.5,  0.5),
        V4::new( 0.5, -0.5, -0.5,  0.5),
        V4::new( 0.5, -0.5,  0.5,  0.5),
        V4::new(-0.5, -0.5,  0.5,  0.5),

        V4::new(-0.5,  0.5, -0.5,  0.5),
        V4::new( 0.5,  0.5, -0.5,  0.5),
        V4::new( 0.5,  0.5,  0.5,  0.5),
        V4::new(-0.5,  0.5,  0.5,  0.5),
    ])
}

macro_rules! CUBE_EDGE {
    () => (vec![
        Edge::new( 0,  1),
        Edge::new( 1,  2),
        Edge::new( 2,  3),
        Edge::new( 3,  0),
        Edge::new( 0,  4),
        Edge::new( 1,  5),
        Edge::new( 2,  6),
        Edge::new( 3,  7),
        Edge::new( 4,  5),
        Edge::new( 5,  6),
        Edge::new( 6,  7),
        Edge::new( 7,  4),

        Edge::new( 8,  9),
        Edge::new( 9, 10),
        Edge::new(10, 11),
        Edge::new(11,  8),
        Edge::new( 8, 12),
        Edge::new( 9, 13),
        Edge::new(10, 14),
        Edge::new(11, 15),
        Edge::new(12, 13),
        Edge::new(13, 14),
        Edge::new(14, 15),
        Edge::new(15, 12),

        Edge::new( 0,  8),
        Edge::new( 1,  9),
        Edge::new( 2, 10),
        Edge::new( 3, 11),
        Edge::new( 4, 12),
        Edge::new( 5, 13),
        Edge::new( 6, 14),
        Edge::new( 7, 15),
    ])
}

pub struct V4 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

pub struct V3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

pub struct Edge {
    pub a: usize,
    pub b: usize,
}

pub struct State {
    // renderer
    pub v: Vec<V4>,
    pub e: Vec<Edge>,
    pub f: f32,
    pub r: f32,
    pub p: V4,

    // controls
    pub toggle_rotate: bool
}

impl V4 {
    pub fn new(x: f32, y: f32, z: f32, w: f32) -> V4 {
        V4 { x, y, z, w }
    }
}

impl V3 {
    pub fn new(x: f32, y: f32, z: f32) -> V3 {
        V3 { x, y, z }
    }
}

impl Edge {
    pub fn new(a: usize, b: usize) -> Edge {
        Edge { a, b }
    }
}

impl Default for State {
    fn default() -> Self {
        Self {
            v: CUBE_VERT!(),
            e: CUBE_EDGE!(),
            f: 2.5,
            r: 0.0,
            p: V4::new(0.0, 0.0, 0.0, 0.0),

            toggle_rotate: false,
        }
    }
}

pub fn fov_to_fl(fov: f32) -> f32 {
    2.0 / (2.0 * (fov / 2.0).tan())
}

pub fn render(state: &mut State, size: usize) -> Vec<Vec<u8>> {
    struct ProjectedPoint { x: f32, y: f32, d: f32 }
    struct ScreenPoint { x: isize, y: isize, d: f32 }

    let mut screen = vec![vec![0; size]; size];
    let mut verts = Vec::with_capacity(state.v.len());
    let sin = state.r.sin();
    let cos = state.r.cos();

    for v in state.v.iter() {
        // 4d to 3d
        let x = v.x * sin + v.w * cos - state.p.x;
        let y = v.y - state.p.y;
        let z = v.z - state.p.z;
        let w = v.x * cos - v.w * sin - state.p.w;
        let d = w + state.f;

        let px = x * state.f / d;
        let py = y * state.f / d;
        let pz = z * state.f / d;

        // 3d to 2d
        let d = pz + state.f;
        verts.push(ProjectedPoint {
            x:  (px * state.f) / d,
            y: -(py * state.f) / d,
            d,
        });
    }
    let mut sverts = Vec::with_capacity(verts.len());
    for v in verts {
        sverts.push(ScreenPoint {
            x: (size as f32 * (v.x*0.5+0.5)) as isize,
            y: (size as f32 * (v.y*0.5+0.5)) as isize,
            d: v.d,
        });
    }
    for edge in state.e.iter() {
        let v1 = &sverts[edge.a];
        let v2 = &sverts[edge.b];

        if v1.d <= 0.0 || v2.d <= 0.0 {
            continue;
        }

        // Line drawing
        let mut x = v1.x;
        let mut y = v1.y;
        let mut dx = (v2.x-v1.x).abs();
        let mut dy = (v2.y-v1.y).abs();
        let s1 = (v2.x-v1.x).signum();
        let s2 = (v2.y-v1.y).signum();
        let interchange = if dy > dx {
            let t = dx;
            dx = dy;
            dy = t;
            true
        } else {
            false
        };
        let mut e = 2 * dy - dx;
        let a = 2 * dy;
        let b = 2 * dy - 2 * dx;
        for i in 0..dx {
            if e < 0 {
                if interchange {
                    y += s2;
                } else {
                    x += s1;
                }
                e += a;
            } else {
                y += s2;
                x += s1;
                e += b;
            }

            let i = i as f32 / dx as f32;
            let l = v1.d * (1.0 - i) + v2.d * i;
            for i in 0..(size as isize).div_ceil(100) {
                if interchange {
                    plot(&mut screen, size, x+i, y, ((1.0-(l / 10.0).min(0.5)) * 255.0) as u8);
                } else {
                    plot(&mut screen, size, x, y+i, ((1.0-(l / 10.0).min(0.5)) * 255.0) as u8);
                }
            }
        }
    }
    screen
}

fn plot(screen: &mut Vec<Vec<u8>>, size: usize, x: isize, y: isize, val: u8) {
    if  x >= 0 && x < size as isize &&
        y >= 0 && y < size as isize {
        if screen[y as usize][x as usize] < val {
            screen[y as usize][x as usize] = val
        }
    }
}
