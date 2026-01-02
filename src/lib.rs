pub mod library {
    pub fn dot(vec1: [f32; 2], vec2: [f32; 2]) -> f32 {
        vec1[0] * vec2[0] + vec1[1] * vec2[1]
    }

    pub fn get_magnitude(vec: [f32; 2]) -> f32 {
        (vec[0].powf(2.0) + vec[1].powf(2.0)).sqrt()
    }

    pub fn normalize(vec: [f32; 2]) -> [f32; 2] {
        let mag = get_magnitude(vec);
        if mag == 0.0 {
            return [0.0, 0.0];
        }
        [vec[0] / mag, vec[1] / mag]
    }

    pub fn find_vector(x: [f32; 2], y: [f32; 2]) -> [f32; 2] {
        [y[0] - x[0], y[1] - x[1]]
    }

    pub fn find_normal(x: [f32; 2], y: [f32; 2]) -> [f32; 2] {
        let vec = normalize(find_vector(x, y));
        [-vec[1], vec[0]]
    }
}
