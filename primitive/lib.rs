extern crate alga;
#[cfg(not(test))]
extern crate approx;
#[cfg(test)]
#[macro_use]
extern crate approx;
extern crate nalgebra as na;
extern crate stl_io;
extern crate truescad_bbox;
extern crate truescad_types;
use std::fmt::Debug;
use truescad_bbox::BoundingBox;
pub use truescad_types::{Float, Point, Vector, EPSILON_X, EPSILON_Y, EPSILON_Z};

mod transformer;
pub use self::transformer::AffineTransformer;

mod twister;
pub use self::twister::Twister;

mod bender;
pub use self::bender::Bender;

mod boolean;
pub use self::boolean::{Intersection, Union};

mod sphere;
pub use self::sphere::Sphere;

mod cylinder;
pub use self::cylinder::{Cone, Cylinder};

mod slab;
pub use self::slab::{SlabX, SlabY, SlabZ};

mod mesh;
pub use self::mesh::Mesh;

pub struct PrimitiveParameters {
    pub fade_range: Float,
    pub r_multiplier: Float,
}

pub const ALWAYS_PRECISE: Float = 1.;

pub fn normal_from_object(f: &Object, p: Point) -> Vector {
    let center = f.approx_value(p, ALWAYS_PRECISE);
    let dx = f.approx_value(&p + *EPSILON_X, ALWAYS_PRECISE) - center;
    let dy = f.approx_value(&p + *EPSILON_Y, ALWAYS_PRECISE) - center;
    let dz = f.approx_value(&p + *EPSILON_Z, ALWAYS_PRECISE) - center;
    Vector::new(dx, dy, dz).normalize()
}

pub trait Object: ObjectClone + Debug + Sync + Send {
    fn bbox(&self) -> &BoundingBox<Float>;
    fn set_bbox(&mut self, _: BoundingBox<Float>) {
        unimplemented!();
    }
    fn set_parameters(&mut self, _: &PrimitiveParameters) {}
    // Value is 0 on object surfaces, negative inside and positive outside of objects.
    // If positive, value is guarateed to be the minimum distance to the object surface.
    // return some approximation (which is always larger then the proper value).
    // Only do a proper calculation, for values smaller then slack.
    fn approx_value(&self, _: Point, _: Float) -> Float {
        unimplemented!();
    }
    fn normal(&self, _: Point) -> Vector {
        unimplemented!();
    }
    fn translate(&self, v: Vector) -> Box<Object> {
        AffineTransformer::new_translate(self.clone_box(), v)
    }
    fn rotate(&self, r: Vector) -> Box<Object> {
        AffineTransformer::new_rotate(self.clone_box(), r)
    }
    fn scale(&self, s: Vector) -> Box<Object> {
        AffineTransformer::new_scale(self.clone_box(), s)
    }
}

pub trait ObjectClone {
    fn clone_box(&self) -> Box<Object>;
}

impl<T> ObjectClone for T
where
    T: 'static + Object + Clone,
{
    fn clone_box(&self) -> Box<Object> {
        Box::new(self.clone())
    }
}

// We can now implement Clone manually by forwarding to clone_box.
impl Clone for Box<Object> {
    fn clone(&self) -> Box<Object> {
        self.clone_box()
    }
}

// Objects never equal each other
impl PartialEq for Box<Object> {
    fn eq(&self, _: &Box<Object>) -> bool {
        false
    }
}

// Objects are never ordered
impl PartialOrd for Box<Object> {
    fn partial_cmp(&self, _: &Box<Object>) -> Option<::std::cmp::Ordering> {
        None
    }
}
