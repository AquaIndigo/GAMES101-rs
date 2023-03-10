use std::rc::Rc;
use super::bounds3::Bounds3;
use super::bvh::BVHAccel;
use super::intersection::Intersection;
use super::material::Material;
use super::object::{Object};
use super::ray::Ray;
use super::renderer::EPSILON;
use super::utils::load_triangles;
use super::vector::{cross, dot, lerp, normalize, Vector2f, Vector3f};

#[derive(Default, Debug, Clone)]
pub struct Triangle {
    pub v0: Vector3f,
    pub v1: Vector3f,
    pub v2: Vector3f,
    pub e1: Vector3f,
    pub e2: Vector3f,
    pub normal: Vector3f,
    m: Option<Rc<Material>>,
}

impl Triangle {
    pub fn new(v0: Vector3f, v1: Vector3f, v2: Vector3f, m: Option<Rc<Material>>) -> Self {
        let e1 = &v1 - &v0;
        let e2 = &v2 - &v0;
        let normal = normalize(&cross(&e1, &e2));
        Self { v0, v1, v2, e1, e2, normal, m }
    }
}

impl Object for Triangle {
    fn get_intersection(&self, ray: Ray) -> Intersection {
        let mut inter = Intersection::new();
        if dot(&ray.direction, &self.normal) > 0.0 {
            return inter;
        }
        let (u, v, t_tmp): (f64, f64, f64);
        let pvec = cross(&ray.direction, &self.e2);
        let det = dot(&self.e1, &pvec) as f64;
        if det.abs() < EPSILON as f64 { return inter; }
        let det_inv = 1.0 / det;
        let tvec = &ray.origin - &self.v0;
        u = dot(&tvec, &pvec) as f64 * det_inv;
        if u < 0.0 || u > 1.0 { return inter; }
        let qvec = cross(&tvec, &self.e1);
        v = dot(&ray.direction, &qvec) as f64 * det_inv;
        if v < 0.0 || u + v > 1.0 { return inter; }
        t_tmp = dot(&self.e2, &qvec) as f64 * det_inv;
        if t_tmp < 0.0 { return inter; }
        inter.happened = true;
        inter.obj = Some(Rc::new(self.clone()));
        inter.normal = self.normal.clone();
        inter.coords = ray.at(t_tmp);
        inter.m = self.m.clone();
        inter.distance = t_tmp;
        inter
    }

    fn get_surface_properties(&self, _p: &Vector3f, _q: &Vector3f, _index: usize, _uv: Vector2f, _st: &mut Vector2f) -> Vector3f {
        self.normal.clone()
    }

    fn eval_diffuse_color(&self, _v: &Vector2f) -> Vector3f {
        Vector3f::same(0.5)
    }

    fn get_bounds(&self) -> Bounds3 {
        Bounds3::union_point(&Bounds3::new(self.v0.clone(), self.v1.clone()), &self.v2)
    }
}

pub struct MeshTriangle {
    pub bounding_box: Bounds3,
    pub bvh: Option<Rc<BVHAccel>>,
    pub m: Option<Rc<Material>>,
}

impl MeshTriangle {
    pub fn from_obj(filename: &str) -> Self {
        let (bounding_box, triangles) =  load_triangles(filename) ;
        let ptrs: Vec<Rc<dyn Object>> = triangles.into_iter().map(|t| Rc::new(t) as Rc<dyn Object>).collect();
        let bvh = BVHAccel::default(ptrs);
        Self {
            bounding_box,
            bvh: Some(Rc::new(bvh)),
            m: None,
        }
    }
}

impl Object for MeshTriangle {
    fn get_intersection(&self, ray: Ray) -> Intersection {
        if self.bvh.is_some() {
            let bvh = self.bvh.as_ref().unwrap();
            bvh.intersect(&ray)
        } else { Intersection::new() }
    }

    fn get_surface_properties(&self, _p: &Vector3f, _q: &Vector3f,
                              _index: usize, _uv: Vector2f, _st: &mut Vector2f) -> Vector3f {
        Vector3f::zeros()
    }

    fn eval_diffuse_color(&self, st: &Vector2f) -> Vector3f {
        let scale = 5.0;
        let pattern = (((st.x * scale) % 1.0) > 0.5) ^ (((st.y * scale) % 1.0) > 0.5);
        let pattern = pattern as i32 as f32;
        lerp(&Vector3f::new(0.815, 0.235, 0.031), &Vector3f::new(0.937, 0.937, 0.231), pattern)
    }

    fn get_bounds(&self) -> Bounds3 {
        self.bounding_box.clone()
    }
}
