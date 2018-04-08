use {Object, PrimitiveParameters, BoundingBox};
use alga::linear::Transformation;
use truescad_types::{Float, Transform, Point, Vector};

#[derive(Clone, Debug)]
pub struct AffineTransformer {
    object: Box<Object>,
    transform: Transform,
    scale_min: Float,
    bbox: BoundingBox,
}

impl Object for AffineTransformer {
    fn approx_value(&self, p: Point, slack: Float) -> Float {
        let approx = self.bbox.value(p);
        if approx <= slack {
            self.object
                .approx_value(self.transform.transform_point(&p), slack / self.scale_min) *
            self.scale_min
        } else {
            approx
        }
    }
    fn bbox(&self) -> &BoundingBox {
        &self.bbox
    }
    fn set_parameters(&mut self, p: &PrimitiveParameters) {
        self.object.set_parameters(p);
    }
    fn normal(&self, p: Point) -> Vector {
        self.transform
            .transform_vector(&self.object.normal(self.transform.transform_point(&p)))
            .normalize()
    }
    fn translate(&self, v: Vector) -> Box<Object> {
        let new_trans = self.transform.append_translation(&-v);
        AffineTransformer::new_with_scaler(self.object.clone(), new_trans, self.scale_min)
    }
    fn rotate(&self, r: Vector) -> Box<Object> {
        let euler = ::na::Rotation::from_euler_angles(r.x, r.y, r.z).to_homogeneous();
        let new_trans = self.transform * euler;
        AffineTransformer::new_with_scaler(self.object.clone(), new_trans, self.scale_min)
    }
    fn scale(&self, s: Vector) -> Box<Object> {
        let new_trans = self.transform
            .append_nonuniform_scaling(&Vector::new(1. / s.x, 1. / s.y, 1. / s.z));
        AffineTransformer::new_with_scaler(self.object.clone(),
                                           new_trans,
                                           self.scale_min * s.x.min(s.y.min(s.z)))
    }
}

impl AffineTransformer {
    fn identity(o: Box<Object>) -> Box<Object> {
        AffineTransformer::new(o, Transform::identity())
    }
    fn new(o: Box<Object>, t: Transform) -> Box<AffineTransformer> {
        AffineTransformer::new_with_scaler(o, t, 1.)
    }
    fn new_with_scaler(o: Box<Object>, t: Transform, scale_min: Float) -> Box<AffineTransformer> {
        // TODO: Calculate scale_min from t.
        // This should be something similar to
        // 1./Vector::new(t.x.x, t.y.x, t.z.x).magnitude().min(
        // 1./Vector::new(t.x.y, t.y.y, t.z.y).magnitude().min(
        // 1./Vector::new(t.x.z, t.y.z, t.z.z).magnitude()))

        match t.try_inverse() {
            None => panic!("Failed to invert {:?}", t),
            Some(t_inv) => {
                let bbox = o.bbox().transform(&t_inv);
                Box::new(AffineTransformer {
                             object: o,
                             transform: t,
                             scale_min: scale_min,
                             bbox: bbox,
                         })
            }
        }
    }
    pub fn new_translate(o: Box<Object>, v: Vector) -> Box<Object> {
        AffineTransformer::identity(o).translate(v)
    }
    pub fn new_rotate(o: Box<Object>, r: Vector) -> Box<Object> {
        AffineTransformer::identity(o).rotate(r)
    }
    pub fn new_scale(o: Box<Object>, s: Vector) -> Box<Object> {
        AffineTransformer::identity(o).scale(s)
    }
}


#[cfg(test)]
mod test {
    use super::*;

    #[derive(Clone, Debug, PartialEq)]
    pub struct MockObject {
        value: Float,
        normal: Vector,
    }

    impl MockObject {
        pub fn new(value: Float, normal: Vector) -> Box<MockObject> {
            Box::new(MockObject {
                         value: value,
                         normal: normal,
                     })
        }
    }

    impl Object for MockObject {
        fn approx_value(&self, _: Point, _: Float) -> Float {
            self.value
        }
        fn normal(&self, _: Point) -> Vector {
            self.normal.clone()
        }
    }

    #[test]
    fn translate() {
        let mock_object = MockObject::new(1.0, Vector::new(1.0, 0.0, 0.0));
        let translated = mock_object.translate(Vector::new(0.0001, 0.0, 0.0));
        let p = Point::new(1.0, 0.0, 0.0);
        assert_eq!(mock_object.normal(p), translated.normal(p));
    }
}
