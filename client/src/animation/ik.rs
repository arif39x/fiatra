use crate::core::skeleton::{Pose, Skeleton};

#[allow(dead_code)]
pub struct FABRIKSolver {
    pub tolerance: f32,
    pub max_iterations: u32,
}

impl FABRIKSolver {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self {
            tolerance: 0.01,
            max_iterations: 20,
        }
    }

    #[allow(dead_code)]
    pub fn solve(
        &self,
        _skeleton: &Skeleton,
        pose: &Pose,
        _target: (f32, f32, f32),
        _chain: &[usize],
    ) -> Pose {
        pose.clone()
    }
}
