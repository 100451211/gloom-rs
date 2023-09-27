extern crate nalgebra_glm as glm;

const PI:f32 = std::f32::consts::PI;
const limit: f32 = PI/2.0;

pub struct Camera {
    pub position: glm::TVec3<f32>,
    rotation: glm::TVec3<f32>,
    matrix: glm::TMat4<f32>,
}

// Intialise a new camera looking down the z axis placed in position (0, 0, 0)
impl Camera {
    pub fn new() -> Camera {
        Camera {
            position: glm::vec3(0.0, 0.0, 0.0),
            rotation: glm::vec3(0.0, 0.0, 0.0),
            matrix: glm::identity(),
        }
    }

    // Return the view matrix for the camera
    pub fn view(&self) -> glm::TMat4<f32> {
        let mut view = glm::identity();
        view = glm::rotate_x(&view, self.rotation.x);
        view = glm::rotate_y(&view, self.rotation.y);
        view = glm::rotate_z(&view, self.rotation.z);
        view = glm::translate(&view, &(-self.position));
        view
    }

    // Return matrix that represents how  the camera sees the world
    pub fn projection(&self, aspect_ratio: f32) -> glm::TMat4<f32> {
        glm::perspective(aspect_ratio, 45.0, 0.1, 100.0)
    }

    // Move the camera by the given vector : camera.translate(glm::vec3(0.0, 0.1, 0.0)); - move by 0.1 in the y axis
    pub fn translate(&mut self, translation: glm::TVec3<f32>) {
        self.position += translation;
        self.view();
    }

    // Rotate the camera by the given vector : camera.rotate(glm::vec3(0.0, 0.1, 0.0)); - rotate by 0.1 in the y axis
    pub fn rotate(&mut self, rotation: glm::TVec3<f32>) {
        self.rotation += rotation;
    }

    /// Set position and rotation of camera back to default.
    pub fn reset(&mut self) {
        self.position = glm::vec3(0.0, 0.0, 0.0);
        self.rotation = glm::vec3(0.0, 0.0, 0.0);
        self.matrix = glm::identity();
    }

    pub fn get_direction(&self) -> glm::TVec3<f32> {
        let angle = self.rotation[0];
        let yaw = self.rotation[1] + (std::f32::consts::PI / 2.0);

        let xz_len = angle.cos();
        let x = xz_len * yaw.cos();
        let y = angle.sin();
        let z = xz_len * yaw.sin();

        let mut direction = glm::vec3(x, y, z);
        direction = glm::normalize(&direction);
        return direction;
    }

    fn move_position(
        &mut self,
        mut direction: glm::TVec3<f32>,
        distance: f32,
    ) {
        direction = glm::normalize(&mut direction);
        self.position.x += direction.x * distance;
        self.position.y += direction.y * distance;
        self.position.z += direction.z * distance;
    }

    pub fn up( &mut self, distance: f32,) {
        let direction: glm::TVec3<f32> = glm::vec3(0.0, 1.0, 0.0);
        self.move_position(direction, distance);
    }

    /// Move camera striaght down (in -y-direction)
    pub fn down( &mut self, distance: f32,) {
        self.up(-distance);
    }

    pub fn forward( &mut self, distance: f32,) {
        self.move_position(self.get_direction(), -distance);
    }

    pub fn backward( &mut self, distance: f32,) {
        self.forward(-distance);
    }

    pub fn left( &mut self, distance: f32,) {
        let mut direction = self.get_direction();
        direction = glm::normalize(&glm::cross(&direction, &glm::vec3(0.0, 1.0, 0.0)));
        self.move_position(direction, distance);
    }

    pub fn right( &mut self, distance: f32,) {
        self.left(-distance);
    }

  
    pub fn update_yaw(&mut self,delta: f32,) {
        let mut new_yaw = self.rotation[0] + delta;
        if new_yaw < 0.0 {
            new_yaw = -(-new_yaw % (2.0 * std::f32::consts::PI));
        } else {
            new_yaw = new_yaw % (2.0 * std::f32::consts::PI);
        }
        self.rotation[0] = new_yaw;
    }


    pub fn update_angle(&mut self,delta: f32,) {
        let mut new_pitch = self.rotation[1] - delta;
        if new_pitch > limit {
            new_pitch = limit;
        } else if new_pitch <  limit {
            new_pitch =  limit;
        }
        self.rotation[1] = new_pitch;
    }

}
