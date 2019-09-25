// pub use primitive::{Sphere};

// pub type Ray = ray::Ray3<float>;
// pub type Point = Point<float>;

use super::Float;
use nalgebra as na;
use rayon::prelude::*;
use std::cmp;
use truescad_luascad::implicit3d::Object;

const EPSILON: Float = 0.003;
const APPROX_SLACK: Float = 0.1;

const FOCAL_FACTOR: Float = 36. /* 36 mm film */ / 50.;

#[derive(Copy, Clone, Debug)]
pub struct Ray {
    pub origin: na::Point3<Float>,
    pub dir: na::Vector3<Float>,
}

impl Ray {
    pub fn new(o: na::Point3<Float>, d: na::Vector3<Float>) -> Ray {
        Ray { origin: o, dir: d }
    }
}

#[derive(Clone)]
pub struct Renderer {
    light_dir: na::Vector3<Float>,
    trans: na::Matrix4<Float>,
    object: Option<Box<dyn Object<Float>>>,
    epsilon: Float,
    maxval: Float,
    approx_slack: Float,
}

impl Renderer {
    pub fn new() -> Renderer {
        Renderer {
            light_dir: na::Vector3::new(-2. / 3., 2. / 3., -1. / 3.),
            trans: na::Matrix4::identity(),
            object: None,
            epsilon: EPSILON,
            maxval: 0.,
            approx_slack: APPROX_SLACK,
        }
    }

    pub fn set_object(&mut self, object: Option<Box<dyn Object<Float>>>) {
        self.object = object;
        self.epsilon = self.object_width() * EPSILON;
        self.maxval = self.object_width();
        self.approx_slack = self.object_width() * APPROX_SLACK;
    }

    pub fn rotate_from_screen(&mut self, x: Float, y: Float) {
        let euler = ::na::Rotation::from_euler_angles(y, x, 0.).to_homogeneous();
        self.trans *= euler;
    }

    pub fn translate_from_screen(&mut self, x: Float, y: Float) {
        let v = na::Vector3::new(-x as Float, y as Float, 0.);
        self.trans = self.trans.append_translation(&v);
    }

    fn cast_ray(
        &self,
        obj: &dyn Object<Float>,
        r: &Ray,
        light_dir: &na::Vector3<Float>,
        origin_value: Float,
    ) -> (usize, Float) {
        let mut cr = *r;
        let mut value = origin_value;
        let mut iter: usize = 0;

        loop {
            cr.dir = cr.dir.normalize();
            cr.origin += cr.dir * value;
            value = obj.approx_value(&cr.origin, self.approx_slack);
            iter += 1;
            if value > self.maxval {
                return (iter, 0.);
            }

            if value < self.epsilon {
                break;
            }
        }
        let norm = obj.normal(&cr.origin);
        let dot = norm.dot(light_dir);
        if dot < 0. {
            return (iter, 0.);
        }
        (iter, dot)
    }

    pub fn draw_on_buf(&self, buf: &mut [u8], width: i32, height: i32) {
        if let Some(my_obj) = &self.object {
            let object_width = self.object_width();
            let viewer_dist = FOCAL_FACTOR * object_width * 3.;

            let scale = 1. / Float::from(cmp::min(width, height));
            let w2 = width / 2;
            let h2 = height / 2;

            let dir_front = self.trans.transform_vector(&na::Vector3::new(0., 0., 1.));
            let dir_rl = self
                .trans
                .transform_vector(&na::Vector3::new(FOCAL_FACTOR, 0., 0.));
            let dir_tb = self
                .trans
                .transform_vector(&na::Vector3::new(0., -FOCAL_FACTOR, 0.));
            let light_dir = self.trans.transform_vector(&self.light_dir);
            let ray_origin = self
                .trans
                .transform_point(&na::Point3::new(0., 0., -viewer_dist));
            let ray = Ray::new(ray_origin, dir_front);

            let origin_value = my_obj.approx_value(&ray.origin, self.approx_slack);

            let mut rows: Vec<_> = buf.chunks_mut((width * 4) as usize).enumerate().collect();
            rows.par_iter_mut().for_each(|y_and_buf| {
                let y = y_and_buf.0 as i32;
                let row_buf = &mut y_and_buf.1;
                let dir_row = dir_front + dir_tb * (Float::from(y - h2) * scale);
                let mut row_ray = ray;
                let mut index: usize = 0;

                for x in 0..width {
                    row_ray.dir = dir_row + dir_rl * (Float::from(x - w2) * scale);

                    let (i, v) = self.cast_ray(&**my_obj, &row_ray, &light_dir, origin_value);

                    let b = (255.0 * v * v) as u8;

                    row_buf[index] = i as u8;
                    index += 1;
                    row_buf[index] = b;
                    index += 1;
                    row_buf[index] = b;
                    index += 1;
                    index += 1;
                }
            })
        }
    }

    fn object_width(&self) -> Float {
        if let Some(ref my_obj) = self.object {
            return my_obj
                .bbox()
                .max
                .x
                .abs()
                .max(my_obj.bbox().min.x.abs())
                .max(my_obj.bbox().max.y.abs().max(my_obj.bbox().min.y.abs()))
                .max(my_obj.bbox().max.z.abs().max(my_obj.bbox().min.z.abs()))
                * 2.;
        }
        0.
    }
}

impl Default for Renderer {
    fn default() -> Self {
        Self::new()
    }
}
