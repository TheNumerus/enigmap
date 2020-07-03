#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use] extern crate rocket;
use rocket::Request;
use rocket_contrib::json::Json;

use enigmap::HexMap;
use enigmap::generators::{Circle, MapGen};

#[catch(404)]
fn not_found(req: &Request) -> String {
    format!("Sorry, '{}' is not a valid path.", req.uri())
}

#[get("/circle?<x>&<y>")]
fn circle(x: Option<u32>, y: Option<u32>) -> Json<HexMap> {
    let size_x = x.unwrap_or(100);
    let size_y = y.unwrap_or(75);

    let mut map = HexMap::new(size_x, size_y);

    let gen = Circle::new_optimized(&map);
    gen.generate(&mut map);
    Json(map)
}

fn main() {
    rocket::ignite()
        .mount("/", routes![circle])
        .register(catchers![not_found])
        .launch();
}