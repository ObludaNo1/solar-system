use std::{
    f32::consts::PI,
    time::{Duration, Instant},
};

use cgmath::{InnerSpace, Matrix3, Matrix4, Point3, Rad, SquareMatrix, Vector3};

pub const UP: Vector3<f32> = Vector3::new(0.0, 1.0, 0.0);
// 5 pixels of movements results in 1 degree of rotation
const ROTATION_MULTIPLIER: f32 = PI / 180.0 / 5.0;

#[derive(Debug, Clone, Copy, PartialEq, Default)]
enum Change {
    #[default]
    None,
    Positive {
        init: Instant,
        last: Instant,
    },
    Negative {
        init: Instant,
        last: Instant,
    },
}

impl Change {
    fn positive(now: Instant) -> Self {
        Change::Positive {
            init: now,
            last: now,
        }
    }

    fn negative(now: Instant) -> Self {
        Change::Negative {
            init: now,
            last: now,
        }
    }

    fn take(&mut self, now: Instant, mapper: impl Fn(Duration) -> f32) -> f32 {
        match self {
            Change::None => 0.0,
            Change::Positive { init, last } => {
                // the result is difference between mapped duration of current movement and previous movement
                let result = mapper(now - *init) - mapper(*last - *init);
                *last = now; // reset start time
                result
            }
            Change::Negative { init, last } => {
                let result = mapper(now - *init) - mapper(*last - *init);
                *last = now; // reset start time
                -result
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
struct Movements {
    forward: Change,
    right: Change,
    up: Change,
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum MovementDirection {
    #[default]
    None,
    Positive,
    Negative,
}

impl MovementDirection {
    pub fn positive(pressed: bool) -> Self {
        if pressed {
            MovementDirection::Positive
        } else {
            MovementDirection::None
        }
    }

    pub fn negative(pressed: bool) -> Self {
        if pressed {
            MovementDirection::Negative
        } else {
            MovementDirection::None
        }
    }
}

fn ms_map(duration: Duration) -> f32 {
    // map duration to seconds
    duration.as_secs_f32().powf(5.0)
}

/// Camera control struct.
///
/// Movements are time based. When movements begin, its timestamp is remembered. Each time the
/// camera is materialized into a view matrix, movement is computed based on elapsed time since the
/// movement began.
///
/// Camera view direction is based on azimuth and zenith. Since those are controlled by mouse
/// movements, where the same logic as for the movements does not apply and even if it's controlled
/// by keyboard, computing rotation with movement together is non-trivial. Therefore camera is
/// rotation is instantaneous.
#[derive(Debug, Clone, PartialEq)]
pub struct CameraControl {
    // current world position
    position: Point3<f32>,
    // current view direction
    view_direction: Vector3<f32>,
    // relative to camera view vector
    movements: Movements,
}

impl CameraControl {
    /// Integrates all movement changes based on current time and returns the resulting view matrix.
    pub fn snapshot(&mut self, now: Instant) -> Matrix4<f32> {
        self.materialize_movements(now);
        Matrix4::look_to_rh(self.position, self.view_direction, UP)
    }

    /// Forward is positive, backwards is negative
    pub fn move_forw_backw(&mut self, now: Instant, direction: MovementDirection) {
        self.materialize_movements(now);
        match direction {
            MovementDirection::None => self.movements.forward = Change::None,
            MovementDirection::Positive => self.movements.forward = Change::positive(now),
            MovementDirection::Negative => self.movements.forward = Change::negative(now),
        }
    }

    /// Right is positive, left is negative
    pub fn move_sideways(&mut self, now: Instant, direction: MovementDirection) {
        self.materialize_movements(now);
        match direction {
            MovementDirection::None => self.movements.right = Change::None,
            MovementDirection::Positive => self.movements.right = Change::positive(now),
            MovementDirection::Negative => self.movements.right = Change::negative(now),
        }
    }

    // Up is positive, down is negative
    pub fn move_vertical(&mut self, now: Instant, direction: MovementDirection) {
        self.materialize_movements(now);
        match direction {
            MovementDirection::None => self.movements.up = Change::None,
            MovementDirection::Positive => self.movements.up = Change::positive(now),
            MovementDirection::Negative => self.movements.up = Change::negative(now),
        }
    }

    pub fn rotate(&mut self, now: Instant, delta_x: f32, delta_y: f32) {
        self.materialize_movements(now);

        let delta_x = delta_x * ROTATION_MULTIPLIER;
        let delta_y = delta_y * ROTATION_MULTIPLIER;

        let right = self.view_direction.cross(UP).normalize();

        // zenith needs special treatment since it cannot exceed bounds
        let current_zen = PI * 0.5 - self.view_direction.y.acos();
        let new_zen = (current_zen + delta_y).clamp(-PI * 0.49, PI * 0.49);
        let zen_change = new_zen - current_zen;

        self.view_direction = Matrix3::identity()
            * Matrix3::from_angle_y(Rad(delta_x))
            * Matrix3::from_axis_angle(right, Rad(zen_change))
            * self.view_direction;
    }

    /// updates self position based on current movements and their durations
    fn materialize_movements(&mut self, now: Instant) {
        let right = self.view_direction.cross(UP).normalize();
        self.position = self.position
            + self.view_direction * self.movements.forward.take(now, ms_map)
            + right * self.movements.right.take(now, ms_map)
            + UP * self.movements.up.take(now, ms_map);
    }
}

impl Default for CameraControl {
    fn default() -> Self {
        Self {
            position: Point3::new(0.0, 0.0, 0.0),
            view_direction: Vector3::new(0.0, 0.0, -1.0),
            movements: Movements::default(),
        }
    }
}
