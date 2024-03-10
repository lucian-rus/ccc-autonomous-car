use std::{
    io::{Read, Write},
    net,
};

struct Telemetry {
    speed: f32,
    distance: f32,
    time: f32,
}

#[derive(PartialEq)]
enum TelemetryData {
    Speed,
    Distance,
    Time,
    Update,
}

enum Action {
    Accelerate,
    Brake,
    None,
}

fn parse_line(input: &Vec<char>, output: &mut Telemetry) -> TelemetryData {
    let parsed_input = String::from_iter(input);

    let mut telemetry = TelemetryData::Update;
    if parsed_input == "update" {
        return telemetry;
    }

    let values = parsed_input.split(' ').collect::<Vec<&str>>();
    let parsed_float = values[1].parse::<f32>().unwrap();
    print!("{:}, {:}", values[0], parsed_float);

    if values[0] == "speed" {
        // println!("SET SPEED");
        output.speed = parsed_float;
        telemetry = TelemetryData::Speed;
    }

    if values[0] == "time" {
        // println!("SET TIME");
        output.time = parsed_float;
        telemetry = TelemetryData::Time;
    }

    if values[0] == "distance" {
        // println!("SET DISTANCE");
        output.distance = parsed_float;
        telemetry = TelemetryData::Distance;
    }

    // print!(
    //     "out: {:}, {:}, {:}\n",
    //     output.distance, output.speed, output.time
    // );

    telemetry
}

fn execute_action(input: &Telemetry) -> Action {
    let mut action = Action::None;
    // print!(
    //     "in: {:}, {:}, {:}\n",
    //     input.distance, input.speed, input.time
    // );

    if input.distance < 500.0 {
        action = Action::Accelerate;
        // println!("ACCELERATE");
    }

    if input.distance > 500.0 {
        action = Action::Brake;
        // println!("BREAK");
    }

    action
}

pub fn run() {
    let mut stream = net::TcpStream::connect("127.0.0.1:7000").unwrap();
    let mut word: Vec<char> = vec![];

    let mut telemetry: Telemetry = Telemetry {
        speed: 0.00,
        distance: 0.00,
        time: 0.00,
    };

    loop {
        let mut buf = [0; 1];
        _ = stream.read(&mut buf);

        if buf[0] == 10 {
            let data = parse_line(&word, &mut telemetry);
            if data == TelemetryData::Update {
                match execute_action(&telemetry) {
                    Action::Accelerate => {
                        _ = stream.write("throttle 100\nbrake 0\n".as_bytes()).unwrap();
                    }
                    Action::Brake => {
                        _ = stream.write("throttle 0\nbrake 100\n".as_bytes()).unwrap();
                    }
                    Action::None => {}
                }
            }

            println!();
            word.clear();
        } else {
            word.push(buf[0] as char);
        }
    }
}
