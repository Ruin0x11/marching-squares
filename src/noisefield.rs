use std::cell::RefCell;
use std::ops::Add;

use noise::{Point2, NoiseModule, Perlin, Worley, Fbm, MultiFractal, Seedable};
use hlua::{self, Lua};

thread_local! {
    pub static NOISE: RefCell<NoiseField> = RefCell::new(init());
}

fn init() -> NoiseField {
    let mut nf = NoiseField::new(1);
    nf.add_noise(NoiseType::Perlin.make_noise_module(1));
    nf
}

pub type MyNoiseModule = Box<NoiseModule<Point2<f32>, Output=f32>>;

// 2d noise field
pub struct NoiseField {
    pub seed: usize,
    noise: Vec<MyNoiseModule>
}

impl NoiseField {
    pub fn new(seed: usize) -> NoiseField {
        NoiseField {
            seed: seed,
            noise: Vec::new(),
        }
    }

    pub fn add_noise(&mut self, noise: MyNoiseModule) {
        self.noise.push(noise);
    }

    pub fn sample(&self, pt: [f32; 2]) -> f32 {
        let mut res = 0.0;
        for f in self.noise.iter() {
            res = res.add(f.get(pt));
        }
        res
    }
}

fn lua_noise_reset(seed: u32) {
    NOISE.with(|n| *n.borrow_mut() = NoiseField::new(seed as usize));
}

fn lua_noise_add(noise_type: NoiseType) {
    NOISE.with(|n| {
        let module = noise_type.make_noise_module(n.borrow().seed);
        n.borrow_mut().add_noise(module);
    });
}

fn lua_noise_sample(x: i32, y: i32) -> f32 {
    NOISE.with(|n| n.borrow().sample([x as f32, y as f32]))
}

#[derive(Clone)]
pub enum NoiseType {
    Perlin,
    Brownian(usize, f32, f32, f32),
    Worley(f32, f32)
}

impl NoiseType {
    pub fn make_noise_module(&self, seed: usize) -> MyNoiseModule {
        match *self {
            NoiseType::Perlin => {
                Box::new(Perlin::new().set_seed(seed))
            },
            NoiseType::Brownian(oct, freq, lac, pers) => {
                Box::new(Fbm::new().set_seed(seed).set_octaves(oct).set_frequency(freq).set_lacunarity(lac).set_persistence(pers))
            },
            NoiseType::Worley(freq, disp) => {
                Box::new(Worley::new().set_seed(seed).set_frequency(freq).set_displacement(disp))
            }
        }
    }
}

fn lua_noisetype_perlin() -> NoiseType {
    NoiseType::Perlin
}

fn lua_noisetype_brownian(octaves: u32, frequency: f32, lacunarity: f32, persistence: f32) -> NoiseType {
    NoiseType::Brownian(octaves as usize, frequency, lacunarity, persistence)
}

fn lua_noisetype_worley(frequency: f32, displacement: f32) -> NoiseType {
    NoiseType::Worley(frequency, displacement)
}

implement_lua_push!(NoiseType, |mut metatable| {
    let mut index = metatable.empty_array("__index");
});

implement_lua_read!(NoiseType);

impl<'lua, L> hlua::LuaRead<L> for NoiseType
    where L: hlua::AsMutLua<'lua>
{
    fn lua_read_at_position(lua: L, index: i32) -> Result<NoiseType, L> {
        let val: Result<hlua::UserdataOnStack<NoiseType, _>, _> =
            hlua::LuaRead::lua_read_at_position(lua, index);
        val.map(|d| d.clone())
    }
}

pub fn add_lua_interop(lua: &mut Lua) {
    {
        let mut noise_namespace = lua.empty_array("Noise");

        noise_namespace.set("reset", hlua::function1(lua_noise_reset));

        noise_namespace.set("add", hlua::function1(lua_noise_add));

        noise_namespace.set("sample", hlua::function2(lua_noise_sample));

    }

    let mut noisetype_namespace = lua.empty_array("NoiseType");

    noisetype_namespace.set("perlin", hlua::function0(lua_noisetype_perlin));
    noisetype_namespace.set("brownian", hlua::function4(lua_noisetype_brownian));
    noisetype_namespace.set("worley", hlua::function2(lua_noisetype_worley));
}
