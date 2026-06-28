#[derive(Clone)]
pub struct Template {
    pub name: &'static str,
    pub equation: &'static str,
    pub tag: &'static str,
    pub description: &'static str,
}

pub const TEMPLATES: &[Template] = &[
    Template {
        name: "Sphere",
        equation: "sqrt(x*x + y*y + z*z) - 10.0",
        tag: "geometry",
        description: "Basic signed distance sphere",
    },
    Template {
        name: "Box",
        equation: "Max(abs(x) - 8.0, Max(abs(y) - 6.0, abs(z) - 6.0))",
        tag: "geometry",
        description: "Axis-aligned rectangular box using Max of axis distances",
    },
    Template {
        name: "Infinite Cylinder",
        equation: "sqrt(x*x + y*y) - 5.0",
        tag: "geometry",
        description: "Endless tube along the Z axis",
    },
    Template {
        name: "Plane",
        equation: "y + 5.0",
        tag: "geometry",
        description: "Infinite ground plane at y = -5",
    },
    Template {
        name: "Torus",
        equation: "sqrt((sqrt(x*x + z*z) - 8.0)*(sqrt(x*x + z*z) - 8.0) + y*y) - 3.0",
        tag: "geometry",
        description: "Ring doughnut shape — a circle swept around another circle",
    },
    Template {
        name: "Capsule",
        equation: "Min(sqrt(x*x + (y-6.0)*(y-6.0) + z*z), sqrt(x*x + (y+6.0)*(y+6.0) + z*z)) - 3.0",
        tag: "geometry",
        description: "Two spheres blended by a Min — approximates a pill shape",
    },
    Template {
        name: "Sphere with Hole",
        equation: "Max(sqrt(x*x + y*y + z*z) - 10.0, -(sqrt(x*x + y*y) - 4.0))",
        tag: "geometry",
        description: "Sphere carved by a cylinder using Max for intersection",
    },
    Template {
        name: "Octahedron",
        equation: "(abs(x) + abs(y) + abs(z)) - 8.0",
        tag: "advanced",
        description: "Diamond shape using Manhattan distance (L1 norm)",
    },
    Template {
        name: "Tetrahedron",
        equation: "Max(abs(x + y + z) + x - y - z, Max(abs(x - y + z) - x + y - z, abs(-x - y + z) + x + y - z)) * 0.3 - 4.0",
        tag: "advanced",
        description: "Four-faced solid using plane combinations",
    },
    Template {
        name: "Gyroid",
        equation: "sin(x)*cos(y) + sin(y)*cos(z) + sin(z)*cos(x)",
        tag: "advanced",
        description: "Triply periodic minimal surface — no straight lines, zero mean curvature",
    },
    Template {
        name: "Infinity Mirror",
        equation: "sin(x)*sin(y)*sin(z) - 0.3",
        tag: "advanced",
        description: "Repeating cell pattern — SDF approximation of a Schwarz P surface",
    },
    Template {
        name: "Fractal Box",
        equation: "Max(abs(x) - 5.0, Max(abs(y) - 5.0, abs(z) - 5.0)) - 2.0*sin(z*0.5)*cos(x*0.5+y*0.3)",
        tag: "advanced",
        description: "Box with wavy deformation demonstrating displacement mapping",
    },
    Template {
        name: "Twisted Column",
        equation: "sqrt(x*x + y*y) - (4.0 + sin(z*0.5)*2.0)",
        tag: "advanced",
        description: "Cylinder with sinusoidal radius variation along Z — corkscrew effect",
    },
    Template {
        name: "Mandelbulb Slice",
        equation: "sqrt((sqrt(x*x + y*y) - 3.0)*(sqrt(x*x + y*y) - 3.0) + z*z) - 2.0*sin(x)*sin(y)",
        tag: "advanced",
        description: "Torus deformed by 2D sine waves — fractal-inspired organic shape",
    },
    Template {
        name: "Orbital Tracking",
        equation: "sqrt((x - state.x)*(x - state.x) + (y - state.y)*(y - state.y) + (z - state.z)*(z - state.z)) - 5.0",
        tag: "physics",
        description: "Follows a moving entity with a spherical probe",
    },
];
