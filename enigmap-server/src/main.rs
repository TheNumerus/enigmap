#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use] extern crate rocket;
use rocket::Request;
use rocket::response::status::BadRequest;
use rocket_contrib::json::Json;

use enigmap::HexMap;
use enigmap::generators::{Circle, MapGen, Islands, Inland};

#[catch(404)]
fn not_found(req: &Request) -> String {
    format!("Sorry, '{}' is not a valid path.", req.uri())
}

#[get("/circle?<x>&<y>&<seed>")]
fn circle(x: Option<u32>, y: Option<u32>, seed: Option<u32>) -> Result<Json<HexMap>, BadRequest<String>> {
    let size_x = x.unwrap_or(100);
    let size_y = y.unwrap_or(75);
    // dont generate large maps
    if size_x > 1000 || size_y > 1000 {
        return Err(BadRequest(Some("Map size too large".into())));
    }

    let mut map = HexMap::new(size_x, size_y);

    let mut gen = Circle::new_optimized(&map);

    if let Some(seed) = seed {
        gen.set_seed(seed);
    }

    gen.generate(&mut map);
    Ok(Json(map))
}

#[get("/islands?<x>&<y>&<seed>")]
fn island(x: Option<u32>, y: Option<u32>, seed: Option<u32>) -> Result<Json<HexMap>, BadRequest<String>> {
    let size_x = x.unwrap_or(100);
    let size_y = y.unwrap_or(75);
    // dont generate large maps
    if size_x > 1000 || size_y > 1000 {
        return Err(BadRequest(Some("Map size too large".into())));
    }

    let mut map = HexMap::new(size_x, size_y);

    let mut gen = Islands::default();

    if let Some(seed) = seed {
        gen.set_seed(seed);
    }

    gen.generate(&mut map);
    Ok(Json(map))
}

#[get("/inland?<x>&<y>&<seed>")]
fn inland(x: Option<u32>, y: Option<u32>, seed: Option<u32>) -> Result<Json<HexMap>, BadRequest<String>> {
    let size_x = x.unwrap_or(100);
    let size_y = y.unwrap_or(75);
    // dont generate large maps
    if size_x > 1000 || size_y > 1000 {
        return Err(BadRequest(Some("Map size too large".into())));
    }

    let mut map = HexMap::new(size_x, size_y);

    let mut gen = Inland::default();

    if let Some(seed) = seed {
        gen.set_seed(seed);
    }

    gen.generate(&mut map);
    Ok(Json(map))
}

fn main() {
    rocket::ignite()
        .mount("/", routes![circle, island, inland])
        .register(catchers![not_found])
        .launch();
}