use {BoundingBox, Object};
use truescad_types::{Float, Point, Vector};

#[derive(Clone, Debug, PartialEq)]
pub struct Sphere {
    radius: Float,
    bbox: BoundingBox,
}

impl Sphere {
    pub fn new(r: Float) -> Box<Sphere> {
        Box::new(Sphere {
            radius: r,
            bbox: BoundingBox::new(Point::new(-r, -r, -r), Point::new(r, r, r)),
        })
    }
}

impl Object for Sphere {
    fn approx_value(&self, p: Point, slack: Float) -> Float {
        let approx = self.bbox.value(p);
        if approx <= slack {
            return Vector::new(p.x, p.y, p.z).norm() - self.radius;
        } else {
            approx
        }
    }
    fn bbox(&self) -> &BoundingBox {
        &self.bbox
    }
    fn normal(&self, p: Point) -> Vector {
        return Vector::new(p.x, p.y, p.z).normalize();
    }
}
