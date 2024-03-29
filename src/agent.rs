use crate::parameters::{InitialHeading, Parameters};

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Agent {
    pub position: [f32; 2],
    pub velocity: [f32; 2],
}
impl Agent {
    pub fn new_with_random_start_position(params: &Parameters) -> Self {
        let middle = [
            (params.shader_parameters.canvas_width / 2) as f32,
            (params.shader_parameters.canvas_height / 2) as f32,
        ];

        let in_circle = random_point_in_circle(params.initial_conditions.initial_circle_radius);

        let position = [middle[0] + in_circle[0], middle[1] + in_circle[1]];

        let dir = match params.initial_conditions.initial_heading {
            InitialHeading::Inward => normalize(vector_from_a_to_b(position, middle)),
            InitialHeading::Outward => normalize(vector_from_a_to_b(middle, position)),
            InitialHeading::Random => random_normalized_vector(),
        };

        let velocity = dir;

        Agent { position, velocity }
    }
}

pub fn initial_agent_distribution(params: &Parameters) -> Vec<Agent> {
    (0..params.number_of_agents)
        .map(|_| Agent::new_with_random_start_position(params))
        .collect()
}

fn random_point_in_circle(radius: f32) -> [f32; 2] {
    use rand::Rng as _;
    let mut rng = rand::thread_rng();

    // Randomly pick an angle between 0 and 2π.
    use std::f32::consts::PI;
    let theta: f32 = rng.gen_range(0.0..2.0 * PI);

    // Generate a uniformly distributed random radius inside the circle.
    let r: f32 = (rng.gen::<f32>()).sqrt() * radius;

    // Convert to Cartesian coordinates.
    [r * theta.cos(), r * theta.sin()]
}

fn normalize(vec: [f32; 2]) -> [f32; 2] {
    let magnitude = (vec[0].powi(2) + vec[1].powi(2)).sqrt();

    // Check for zero magnitude to prevent division by zero
    if magnitude == 0.0 {
        return [0.0; 2];
    }

    [vec[0] / magnitude, vec[1] / magnitude]
}

fn vector_from_a_to_b(a: [f32; 2], b: [f32; 2]) -> [f32; 2] {
    [b[0] - a[0], b[1] - a[1]]
}

fn random_normalized_vector() -> [f32; 2] {
    use rand::Rng as _;
    let mut rng = rand::thread_rng();

    // Randomly pick an angle between 0 and 2π.
    use std::f32::consts::PI;
    let theta: f32 = rng.gen_range(0.0..2.0 * PI);

    // Convert the angle to x and y coordinates.
    [theta.cos(), theta.sin()]
}
