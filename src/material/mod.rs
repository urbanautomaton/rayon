use rand::Rng;

use crate::ray::Ray;
use crate::vector::Vec;

pub trait Material<R: Rng> {
    fn scatter(&self, ray_in: &Ray, intersection: &Vec, normal: &Vec, rng: &mut R) -> Option<Ray>;
}

pub struct ReflectiveMaterial {}

impl<R: Rng> Material<R> for ReflectiveMaterial {
    fn scatter(&self, ray_in: &Ray, intersection: &Vec, normal: &Vec, _rng: &mut R) -> Option<Ray> {
        let dot = ray_in.direction.dot(*normal);
        let reflection_direction = ray_in.direction - *normal * (2.0 * dot);

        Some(Ray::new(*intersection, reflection_direction))
    }
}

pub struct FuzzyReflectiveMaterial {
    pub fuzz: f64,
}

impl<R: Rng> Material<R> for FuzzyReflectiveMaterial {
    fn scatter(&self, ray_in: &Ray, intersection: &Vec, normal: &Vec, rng: &mut R) -> Option<Ray> {
        let dot = ray_in.direction.dot(*normal);
        let reflection_direction = ray_in.direction - *normal * (2.0 * dot);

        let coords: [f64; 3] = rng.gen();
        let fuzz_vector = Vec::from(coords) * self.fuzz;
        let scattered = reflection_direction + fuzz_vector;

        if scattered.dot(*normal) > 0.0 {
            Some(Ray::new(*intersection, scattered.normalize()))
        } else {
            None
        }
    }
}

pub struct LambertianMaterial {}

impl LambertianMaterial {
    fn random_in_unit_sphere<R: Rng>(rng: &mut R) -> Vec {
        let mut vec;

        loop {
            let coords: [f64; 3] = rng.gen();

            vec = Vec::from(coords);

            if vec.length() <= 1.0 {
                break vec;
            }
        }
    }
}

impl<R: Rng> Material<R> for LambertianMaterial {
    fn scatter(&self, _ray_in: &Ray, intersection: &Vec, normal: &Vec, rng: &mut R) -> Option<Ray> {
        let direction = Self::random_in_unit_sphere(rng) + *normal;

        Some(Ray::new(*intersection, direction))
    }
}

pub struct DielectricMaterial {
    pub refractive_index: f64,
}

impl DielectricMaterial {
    fn refract(direction: &Vec, normal: &Vec, ni_over_nt: f64) -> Option<Vec> {
        let uv = direction.normalize();
        let dt = uv.dot(*normal);
        let discriminant = 1.0 - ni_over_nt.powi(2) * (1.0 - dt.powi(2));

        if discriminant > 0.0 {
            let refracted = (uv - *normal * dt) * ni_over_nt - *normal * discriminant.sqrt();

            Some(refracted)
        } else {
            None
        }
    }

    fn reflect(ray_in: &Ray, intersection: &Vec, normal: &Vec) -> Option<Ray> {
        let dot = ray_in.direction.dot(*normal);
        let direction = ray_in.direction - *normal * (2.0 * dot);

        Some(Ray::new(*intersection, direction))
    }

    fn schlick(&self, cosine: f64) -> f64 {
        let r0 = ((1.0 - self.refractive_index) / (1.0 + self.refractive_index)).powi(2);

        r0 + (1.0 - r0) * (1.0 - cosine).powi(5)
    }
}

impl<R: Rng> Material<R> for DielectricMaterial {
    fn scatter(&self, ray_in: &Ray, intersection: &Vec, normal: &Vec, rng: &mut R) -> Option<Ray> {
        let outward_normal;
        let ni_over_nt;
        let cosine;
        let rdotn = ray_in.direction.dot(*normal);

        if rdotn > 0.0 {
            outward_normal = *normal * -1.0;
            ni_over_nt = self.refractive_index;
            cosine = self.refractive_index * rdotn;
        } else {
            outward_normal = *normal;
            ni_over_nt = 1.0 / self.refractive_index;
            cosine = -rdotn;
        }

        let reflect_prob = self.schlick(cosine);

        if let Some(refracted) = Self::refract(&ray_in.direction, &outward_normal, ni_over_nt) {
            let val: f64 = rng.gen();

            if val < reflect_prob {
                Self::reflect(ray_in, intersection, normal)
            } else {
                Some(Ray::new(*intersection, refracted))
            }
        } else {
            Self::reflect(ray_in, intersection, normal)
        }
    }
}
