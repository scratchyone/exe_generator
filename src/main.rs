#![feature(proc_macro_hygiene, decl_macro)]
use anyhow;
use base64;
use rocket::http::Method; // 1.
use rocket::State;
use rocket_cors::{AllowedHeaders, AllowedOrigins, Cors, CorsOptions};
use serde;
use serde::Serialize;
use serde_json;
use serde_json::json;
use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::thread;
use uuid::Uuid;
const PROGRAM_NAME: &str = "demo_program";
const TARGET: &str = "x86_64-pc-windows-gnu";
fn get_file_as_byte_vec(filename: &str) -> anyhow::Result<Vec<u8>> {
    let f = File::open(&filename)?;
    let bytes = f.bytes();
    let mut buf = vec![];
    for byte in bytes {
        buf.push(byte.unwrap());
    }
    Ok(buf)
}

#[derive(Serialize, Debug)]
struct CompileItem {
    info: String,
    uuid: String,
}
#[derive(Serialize, Debug)]
struct Completed {
    uuid: String,
    output: String,
    success: bool,
}
#[derive(Debug, Serialize)]
struct Queue {
    queue: Vec<CompileItem>,
    completed: Vec<Completed>,
}

#[macro_use]
extern crate rocket;

#[get("/")]
fn index(queue: State<Arc<Mutex<Queue>>>) -> String {
    format!("{:#?}", queue.lock().unwrap().queue)
}

#[get("/info/<uuid>")]
fn info(uuid: String, queue: State<Arc<Mutex<Queue>>>) -> String {
    let results = queue.lock().unwrap();
    let place_in_queue = results.queue.iter().position(|x| x.uuid == uuid);
    let unfinished_res = results.queue.iter().find(|x| x.uuid == uuid);
    let finished_res = results.completed.iter().find(|x| x.uuid == uuid);
    let unfinished = match unfinished_res {
        Some(_) => true,
        None => false,
    };
    let place_in_queue = match place_in_queue {
        Some(x) => x,
        None => 0,
    };
    let finished = match finished_res {
        Some(_) => true,
        None => false,
    };
    let mut success = false;
    if finished || unfinished {
        success = true;
    }
    match finished {
        false => format!(
            "{}",
            json!({"success": success, "finished": finished, "result": unfinished_res, "place_in_queue": place_in_queue+1})
                .to_string()
        ),
        true => format!(
            "{}",
            json!({"success": success, "finished": finished, "result": finished_res }).to_string()
        ),
    }
}

#[post("/add/<info>")]
fn add(queue: State<Arc<Mutex<Queue>>>, info: String) -> String {
    let uuid = Uuid::new_v4().to_simple().to_string();
    queue.lock().unwrap().queue.push(CompileItem {
        info: info,
        uuid: uuid.clone(),
    });
    format!(
        "{}",
        serde_json::json!({
            "place_in_queue": queue.lock().unwrap().queue.len(),
            "uuid": uuid.clone()
        })
        .to_string()
    )
}

fn make_cors() -> Cors {
    let allowed_origins = AllowedOrigins::some_exact(&[
        "http://localhost:3000",
        "https://scratchyone.com",
        "https://www.scratchyone.com",
    ]);

    CorsOptions {
        // 5.
        allowed_origins,
        allowed_methods: vec![Method::Get].into_iter().map(From::from).collect(), // 1.
        allowed_headers: AllowedHeaders::some(&[
            "Authorization",
            "Accept",
            "Access-Control-Allow-Origin", // 6.
        ]),
        allow_credentials: true,
        ..Default::default()
    }
    .to_cors()
    .expect("error while building CORS")
}

fn compile(files: Vec<(&str, String)>) -> anyhow::Result<Vec<u8>> {
    for file in files {
        let mut f = File::create(format!("{}/src/{}", PROGRAM_NAME, file.0)).unwrap();
        f.write_all(file.1.as_bytes())?;
    }
    if cfg!(target_os = "windows") {
        Command::new("cmd")
            .args(&["/C", &format!("cargo build --release --target {}", TARGET)])
            .current_dir("demo_program")
            .output()?;
    } else {
        Command::new("sh")
            .arg("-c")
            .arg(format!("cargo build --release --target {}", TARGET))
            .current_dir("demo_program")
            .output()?;
    };
    let output = get_file_as_byte_vec(&format!(
        "{}/target/{}/release/{}.exe",
        PROGRAM_NAME, TARGET, PROGRAM_NAME
    ))?;
    fs::remove_file(format!(
        "{}/target/{}/release/{}.exe",
        PROGRAM_NAME, TARGET, PROGRAM_NAME
    ))?;
    Ok(output)
}

fn main() {
    let queue = Arc::new(Mutex::new(Queue {
        queue: vec![],
        completed: vec![],
    }));
    let queue_thread = queue.clone();
    thread::spawn(move || loop {
        //thread::sleep(std::time::Duration::from_millis(2000));
        let queue = queue_thread.lock().unwrap();
        if queue.queue.len() > 0 {
            let info = queue.queue[0].info.clone();
            let uuid = queue.queue[0].uuid.clone();
            std::mem::drop(queue);
            let exe = compile(vec![("test.txt", base64::encode(info))]);
            let success = match exe {
                Ok(_) => true,
                Err(_) => false,
            };
            let output = match exe {
                Ok(x) => base64::encode(x),
                Err(_) => "".to_string(),
            };
            let comp = Completed {
                uuid: uuid,
                output: output,
                success: success,
            };
            let mut queue = queue_thread.lock().unwrap();
            queue.completed.push(comp);
            queue.queue.remove(0);
        }
    });
    let cfg = rocket::config::Config::build(rocket::config::Environment::Development)
        .port(99)
        .unwrap();
    rocket::custom(cfg)
        .manage(queue.clone())
        .attach(make_cors())
        .mount("/", routes![add, info])
        .launch();
}
