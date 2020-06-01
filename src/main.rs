extern crate iced;
use iced::{button, Button, window, text_input, 
		   TextInput, Align, Column, 
		   Settings, Element, Text, Sandbox};

extern crate libloading as lib;

extern crate server;
use server::router::Router;

use std::env;
use std::net::TcpListener;
use std::path::Path;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::thread;


#[derive(Default)]
struct Server {
	ip_address: 	String,
	port: 			String,
	directory: 		String,
	filename: 		String,
	log:			String,
	state: 			State,
}

#[derive(Default)]
struct State {
	ip_input: 		text_input::State,
	port_input: 	text_input::State,
	dir_input: 		text_input::State,
	filename_input: text_input::State,
	start_button: 	button::State,
}


#[derive(Debug, Clone)]
pub enum Message {
	Start,
	IpAddress(String),
	Port(String),
	Directory(String),
	FileName(String),
}

impl Sandbox for Server {
	type Message = Message;

	fn new() -> Self {
		Self::default()
	}

	fn title(&self) -> String {
		String::from("Веб-сервер")
	}

	fn view(&mut self) -> Element<Message>{
		let font_size = 20;

		let ip_address = TextInput::new(
			&mut self.state.ip_input,
			"Введите ip-адрес",
			&self.ip_address,
			Message::IpAddress
		)
		.size(font_size)
		.padding(5);

		let port = TextInput::new(
			&mut self.state.port_input,
			"Введите порт",
			&self.port,
			Message::Port
		)
		.size(font_size)
		.padding(5);

		let directory = TextInput::new(
			&mut self.state.dir_input,
			"Введите путь к директории сайта",
			&self.directory,
			Message::Directory,
		)
		.size(font_size)
		.padding(5);

		let filename = TextInput::new(
			&mut self.state.filename_input,
			"Введите название файла",
			&self.filename,
			Message::FileName,
		)
		.size(font_size)
		.padding(5);

		let start_button = Button::new(
			&mut self.state.start_button,
			Text::new("Запуск")
		)
		.on_press(Message::Start);


		Column::new()
			.spacing(20)
			.padding(10)
			.align_items(Align::Center)
			.push(ip_address)
			.push(port)
			.push(directory)						
			.push(filename)
			.push(start_button)
			.push(Text::new(&self.log).size(14))
			.into()
	}


	fn update(&mut self, message: Message) {
		match message {
			Message::IpAddress(s) => {
				self.ip_address = s;
			}
			Message::Port(s) => {
				self.port = s;
			}
			Message::Directory(s) => {
				self.directory = s;
			}
			Message::FileName(s) => {
				self.filename = s;
			}
			Message::Start => {
				self.log = start(
					self.ip_address.clone(),
					self.port.clone(),
					self.directory.clone(),
					self.filename.clone()
				);
			}
		}
	}
}


fn start(ip: String, port: String, directory: String, name: String) -> String {

    let site = if directory.ends_with("/") {
        format!("{}{}", directory, name)
    } else {
        format!("{}/{}", directory, name)
    };

    let dir = Path::new(&directory);
    match env::set_current_dir(&dir) {
        Err(_) => return "Не удалось открыть директорию".into(),
        _ => {}
    }

    if port.is_empty() || ip.is_empty() || directory.is_empty() || name.is_empty() {
        return "Вы не ввели все нужные данные".into();
    }

    let file = Path::new(&site);

    if !file.exists() {
        return "Файл сайта не существует".into();
    }
    if !site.ends_with(".so") && !site.ends_with(".dll") && !site.ends_with(".dylib") {
        return "Файл должен быть в формате .so, .dll или .dylibw".into();
    }

    match TcpListener::bind(format!("{}:{}", ip, port)) {
        Err(_) => {
            return "IP-адрес недоступен".into();
        }
        _ => {}
    }

    let file = lib::Library::new(site.as_str()).unwrap();
    let mut server = server::Server::new(&ip, &port);
    let (sender, reciever): (Sender<bool>, Receiver<bool>) = mpsc::channel();
    unsafe {
        let thread_s = sender.clone();
        thread::spawn(move || {
            let site: lib::Symbol<unsafe extern "C" fn() -> Router> = match file.get(b"site") {
                Ok(site) => {
                    thread_s.send(true).unwrap();
                    site
                }
                Err(_) => {
                    thread_s.send(false).unwrap();
                    return;
                }
            };
            let router = site();
            server.start(router);
        });
    }

    let success: bool = reciever.recv().unwrap();
    if !success {
        return "Ошибка. не была найдена функция site".into();
    }
    format!("Сервер запущен по адресу http://{}:{}", ip, port)
}


type Flags = ();

pub fn main() {
	Server::run(Settings {
		window: window::Settings{
			size: (320, 300),
			resizable: false,
			decorations: true,	
		},
		flags: Flags::default(),
		default_font: None,
		antialiasing: false,
	});
}
