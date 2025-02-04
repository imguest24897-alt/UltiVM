use std::env;
use std::process;
use actix_web::{web, App, HttpServer, Responder};
async fn greet() -> impl Responder {
    "Hello from UltiVM!"
}

#[actix_web::main]
async fn start_webserver() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .route("/", web::get().to(greet))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 && (args[1] == "--version" || args[1] == "-v") {
        println!("UltiVM 0.0.1");
        return;
    }

    println!("============================================");
    println!("=   WELCOME TO ULTIVM - A ONLINE VIRTUAL   =");
    println!("=         COLLABORATIVE MACHINE            =");
    println!("============================================");
    println!();
    if args.len() > 2 && (args[1] == "--qemu-args" || args[1] == "-q") {
        let qemu_args = &args[2];
        println!("Parsing QEMU (quick emulator) arguments: {}...", qemu_args);

        let allowed_args = vec![
            "-hda", "-cdrom", "-drive", "-accel", "--enable-kvm", "-usb", "-device"
        ];
        let mut valid = true;
        let mut iter = qemu_args.split_whitespace().peekable();

        while let Some(arg) = iter.next() {
            if allowed_args.contains(&arg) {
                match arg {
                    "-hda" | "-cdrom" => {
                        if iter.peek().is_none() {
                            valid = false;
                            break;
                        }
                        iter.next();
                    }
                    "-drive" => {
                        if iter.peek().is_none() || !iter.peek().unwrap().starts_with("file=") {
                            valid = false;
                            break;
                        }
                        iter.next();
                    }
                    "-accel" => {
                        if iter.peek().is_none() || !["kvm", "tcg"].contains(&iter.peek().unwrap()) {
                            valid = false;
                            break;
                        }
                        iter.next();
                    }
                    "--enable-kvm" | "-usb" | "-device" => {
                        // dont need to check anything in this
                    }
                    _ => {
                        valid = false;
                        break;
                    }
                }
            } else {
                valid = false;
                break;
            }
        }

        if valid {
            println!("Parsing success!");
            start_webserver();
        } else {
            println!("E: Parsing failed! Please check the QEMU (quick emulator) arguments.");
        }
    } else {
        println!("Missing or incorrect QEMU (quick emulator) arguments! Could not proceed.");
        println!("To put QEMU arguments, use: ultivm --qemu-args \"<args>\"");
    }
}
