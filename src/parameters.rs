use smart_default::SmartDefault;
use typed_builder::TypedBuilder;

#[derive(Debug, Clone, Copy, PartialEq, TypedBuilder)]
pub struct Parameters {
    /// Number of ticks of the simulation to target per second.
    #[builder(default = 60.0)]
    pub target_ticks_per_second: f32,

    /// Number of agents the buffer is initialized with
    #[builder(default = 500_000)]
    pub number_of_agents: u32,

    #[builder(default)]
    pub initial_conditions: InitialConditions,

    pub shader_parameters: ShaderParameters,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, TypedBuilder, bytemuck::Zeroable, bytemuck::NoUninit)]
pub struct ShaderParameters {
    #[builder(default = 1.0)]
    pub agent_speed: f32,

    #[builder(default = 1)]
    pub bool_enable_agent_bounce: u32,

    #[builder(default = 1)]
    pub bool_enable_agent_deposit: u32,

    #[builder(default = 1)]
    pub bool_enable_agent_rotate: u32,

    #[builder(default = 1)]
    pub bool_enable_agent_rotate_left: u32,

    #[builder(default = 1)]
    pub bool_enable_agent_rotate_randomly: u32,

    #[builder(default = 1)]
    pub bool_enable_agent_rotate_right: u32,

    #[builder(default = 1)]
    pub bool_enable_color: u32,

    #[builder(default = 1)]
    pub bool_enable_decay: u32,

    #[builder(default = 1)]
    pub bool_enable_diffuse: u32,

    #[builder(default = 1)]
    pub bool_enable_render_trail_map: u32,

    #[builder(default = 0)]
    pub bool_enable_high_density_dispersion: u32,

    pub canvas_width: u32,

    pub canvas_height: u32,

    #[builder(default = 10_000_000)]
    pub number_of_active_agents: u32,

    #[builder(default = 1.0)]
    pub vertex_stretch: f32,

    #[builder(default = 0.32)]
    pub decay_strength: f32,

    /// Angle for left and right sensors.
    #[builder(default = 24.2)]
    pub sensor_angle_degrees: f32,

    /// Max angle to turn when left or right sensor dictates turn direction.
    #[builder(default = 29.15)]
    pub max_turn_angle_degrees: f32,

    /// Max angle to turn when turning in a random direction.
    #[builder(default = 1.38)]
    pub max_rand_turn_angle_degrees: f32,

    /// Threshold for forced turn due to high agent density (as measured indirectly by deposit strength)
    #[builder(default = 0.56)]
    pub high_density_threshold: f32,

    /// Speed boost for agents in areas of high agent density (as measured indirectly by deposit strength)
    #[builder(default = 0.85)]
    pub high_density_speed_boost: f32,

    #[builder(default = 0.03)]
    pub deposit_strength: f32,

    #[builder(default = 33.8)]
    pub sensor_distance: f32,
}

impl ShaderParameters {
    pub fn randomize(&mut self) {
        use rand::Rng as _;
        let mut rng = rand::thread_rng();

        self.decay_strength = rng.gen_range(0.001..0.5);
        self.sensor_angle_degrees = rng.gen_range(1.0..120.0);
        self.max_turn_angle_degrees = rng.gen_range(1.0..60.0);
        self.max_rand_turn_angle_degrees = rng.gen_range(0.0..10.0);
        self.high_density_speed_boost = rng.gen_range(0.0..20.0);
        self.high_density_threshold = rng.gen_range(0.50..1.0);
        self.deposit_strength = rng.gen_range(0.001..0.03);
        self.sensor_distance = rng.gen_range(5.0..14.0);
    }
}

#[derive(Debug, Clone, Copy, SmartDefault, PartialEq)]
pub struct InitialConditions {
    /// Radius of circle in which agents are initially distributed
    #[default = 500.0]
    pub initial_circle_radius: f32,

    /// Initial agent direction
    pub initial_heading: InitialHeading,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum InitialHeading {
    Inward,
    Outward,
    #[default]
    Random,
}
