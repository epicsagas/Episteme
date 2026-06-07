// Guards against: Switch Statements FP.
// Rust `match` on enum variants is exhaustive and idiomatic — this is not a
// classic "switch statement smell" because the compiler enforces completeness.

use std::f64::consts::PI;

/// Geometric shape variants for area calculation.
pub enum Shape {
    Circle(f64),
    Rectangle(f64, f64),
    Triangle(f64, f64, f64),
    Square(f64),
    Ellipse(f64, f64),
    Parallelogram(f64, f64),
    Trapezoid(f64, f64, f64),
    Rhombus(f64, f64),
}

/// Compute the area of any supported shape.
///
/// Each arm is a single expression — the match is proportional to the number
/// of variants and cannot be shortened without losing coverage.
pub fn area(shape: &Shape) -> f64 {
    match shape {
        Shape::Circle(radius) => PI * radius * radius,
        Shape::Rectangle(width, height) => width * height,
        Shape::Triangle(base, height, _hypotenuse) => 0.5 * base * height,
        Shape::Square(side) => side * side,
        Shape::Ellipse(semi_major, semi_minor) => PI * semi_major * semi_minor,
        Shape::Parallelogram(base, height) => base * height,
        Shape::Trapezoid(base_a, base_b, height) => 0.5 * (base_a + base_b) * height,
        Shape::Rhombus(diagonal_a, diagonal_b) => 0.5 * diagonal_a * diagonal_b,
    }
}

/// Return a human-readable name for the shape type.
pub fn shape_name(shape: &Shape) -> &'static str {
    match shape {
        Shape::Circle(_) => "circle",
        Shape::Rectangle(_, _) => "rectangle",
        Shape::Triangle(_, _, _) => "triangle",
        Shape::Square(_) => "square",
        Shape::Ellipse(_, _) => "ellipse",
        Shape::Parallelogram(_, _) => "parallelogram",
        Shape::Trapezoid(_, _, _) => "trapezoid",
        Shape::Rhombus(_, _) => "rhombus",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn circle_area() {
        let result = area(&Shape::Circle(1.0));
        assert!((result - PI).abs() < 1e-10);
    }

    #[test]
    fn rectangle_area() {
        assert_eq!(area(&Shape::Rectangle(3.0, 4.0)), 12.0);
    }

    #[test]
    fn trapezoid_area() {
        assert_eq!(area(&Shape::Trapezoid(6.0, 4.0, 5.0)), 25.0);
    }

    #[test]
    fn all_shapes_have_names() {
        let shapes = [
            Shape::Circle(1.0),
            Shape::Rectangle(1.0, 1.0),
            Shape::Triangle(3.0, 4.0, 5.0),
            Shape::Square(2.0),
            Shape::Ellipse(2.0, 1.0),
            Shape::Parallelogram(3.0, 4.0),
            Shape::Trapezoid(2.0, 4.0, 3.0),
            Shape::Rhombus(3.0, 4.0),
        ];
        for shape in &shapes {
            assert!(!shape_name(shape).is_empty());
        }
    }
}
