/// A library module containing 2D vector mathematics utilities.
///
/// This module provides fundamental vector operations used throughout the physics engine,
/// including dot products, magnitude calculations, normalization, and normal vector computation.
pub mod library {
    /// Computes the dot product of two 2D vectors.
    ///
    /// The dot product is calculated as: `vec1.x * vec2.x + vec1.y * vec2.y`
    ///
    /// # Arguments
    ///
    /// * `vec1` - The first 2D vector as [x, y]
    /// * `vec2` - The second 2D vector as [x, y]
    ///
    /// # Returns
    ///
    /// The scalar dot product of the two vectors
    pub fn dot(vec1: [f32; 2], vec2: [f32; 2]) -> f32 {
        vec1[0] * vec2[0] + vec1[1] * vec2[1]
    }

    /// Calculates the magnitude (length) of a 2D vector.
    ///
    /// Uses the Pythagorean theorem: `sqrt(x² + y²)`
    ///
    /// # Arguments
    ///
    /// * `vec` - The 2D vector as [x, y]
    ///
    /// # Returns
    ///
    /// The magnitude of the vector
    pub fn get_magnitude(vec: [f32; 2]) -> f32 {
        (vec[0].powf(2.0) + vec[1].powf(2.0)).sqrt()
    }

    /// Normalizes a 2D vector to unit length.
    ///
    /// Creates a vector with the same direction but magnitude of 1.
    /// Returns [0.0, 0.0] if the input vector has zero magnitude to avoid division by zero.
    ///
    /// # Arguments
    ///
    /// * `vec` - The 2D vector to normalize as [x, y]
    ///
    /// # Returns
    ///
    /// A normalized vector with magnitude 1, or [0.0, 0.0] if input magnitude is 0
    pub fn normalize(vec: [f32; 2]) -> [f32; 2] {
        let mag = get_magnitude(vec);
        if mag == 0.0 {
            return [0.0, 0.0];
        }
        [vec[0] / mag, vec[1] / mag]
    }

    /// Finds the vector from point x to point y.
    ///
    /// Computes the directional vector by subtracting the start point from the end point.
    ///
    /// # Arguments
    ///
    /// * `x` - The starting point as [x, y]
    /// * `y` - The ending point as [x, y]
    ///
    /// # Returns
    ///
    /// The vector from x to y as [dx, dy]
    pub fn find_vector(x: [f32; 2], y: [f32; 2]) -> [f32; 2] {
        [y[0] - x[0], y[1] - x[1]]
    }

    /// Finds the normal (perpendicular) vector to the line segment from x to y.
    ///
    /// Computes a normalized perpendicular vector by rotating the direction vector 90 degrees.
    /// The normal vector is rotated counter-clockwise.
    ///
    /// # Arguments
    ///
    /// * `x` - The starting point of the line segment as [x, y]
    /// * `y` - The ending point of the line segment as [x, y]
    ///
    /// # Returns
    ///
    /// A unit normal vector perpendicular to the line segment
    pub fn find_normal(x: [f32; 2], y: [f32; 2]) -> [f32; 2] {
        let vec = normalize(find_vector(x, y));
        // Rotate 90 degrees counter-clockwise: (x, y) -> (-y, x)
        [-vec[1], vec[0]]
    }
}
