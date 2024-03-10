use std::{
    io::{Read, Write},
    net, num,
};

struct Telemetry {
    speed: f32,
    distance: f32,
    time: f32,
    current_speedlimit: f32,
    distance_to_speedlimit: f32,
    next_speedlimit: f32,
}

#[derive(PartialEq)]
enum TelemetryData {
    Speed,
    Distance,
    Time,
    Update,
    SpeedLimit,
}

enum Action {
    Accelerate,
    AccelerateTempered,
    Brake,
    Cruise,
    None,
}

fn parse_line(input: &Vec<char>, output: &mut Telemetry) -> TelemetryData {
    let parsed_input = String::from_iter(input);

    let mut telemetry = TelemetryData::Update;
    if parsed_input == "update" {
        return telemetry;
    }

    let values = parsed_input.split(' ').collect::<Vec<&str>>();
    // print!("{:}, {:}", values[0], parsed_float);

    if values[0] == "speed" {
        // println!("SET SPEED");
        let parsed_float = values[1].parse::<f32>().unwrap();
        output.speed = parsed_float;
        telemetry = TelemetryData::Speed;
    }

    if values[0] == "time" {
        // println!("SET TIME");
        let parsed_float = values[1].parse::<f32>().unwrap();
        output.time = parsed_float;
        telemetry = TelemetryData::Time;
    }

    if values[0] == "distance" {
        // println!("SET DISTANCE");
        let parsed_float = values[1].parse::<f32>().unwrap();
        output.distance = parsed_float;
        telemetry = TelemetryData::Distance;
    }

    if values[0] == "speedlimit" {
        // println!("SET DISTANCE");
        let parsed_float_curr = values[1].parse::<f32>().unwrap();
        let parsed_float_dist = values[2].parse::<f32>().unwrap();
        let parsed_float_next = values[3].parse::<f32>().unwrap();

        output.current_speedlimit = parsed_float_curr;
        output.distance_to_speedlimit = parsed_float_dist;
        output.next_speedlimit = parsed_float_next;

        telemetry = TelemetryData::SpeedLimit;
    }

    // print!(
    //     "out: {:}, {:}, {:}, {:}\n",
    //     output.distance, output.speed, output.time, output.current_speedlimit
    // );

    telemetry
}

fn execute_action(input: &Telemetry) -> Action {
    let mut action = Action::None;
    // print!(
    //     "in: {:}, {:}, {:}\n",
    //     input.distance, input.speed, input.time
    // );

    if input.distance > 1500.0 {
        action = Action::Brake;
    }

    if input.distance < 1500.0 {
        action = Action::Accelerate;

        if input.speed > input.current_speedlimit - 2.0 {
            action = Action::Cruise;
        }

        let mut distance_to_brake = input.current_speedlimit - input.next_speedlimit;
        distance_to_brake = distance_to_brake.abs() * 1.5;

        if input.distance_to_speedlimit < distance_to_brake
            && input.speed > input.next_speedlimit
            && input.next_speedlimit != 0.0
        {
            action = Action::Brake;
        } else if input.distance_to_speedlimit < distance_to_brake
            && input.speed < input.next_speedlimit
        {
            action = Action::Cruise;
        }
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
        current_speedlimit: 0.00,
        distance_to_speedlimit: 0.00,
        next_speedlimit: 0.00,
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
                    Action::AccelerateTempered => {
                        _ = stream.write("throttle 30\nbrake 0\n".as_bytes()).unwrap();
                    }
                    Action::Brake => {
                        _ = stream.write("throttle 0\nbrake 100\n".as_bytes()).unwrap();
                    }
                    Action::Cruise => {
                        _ = stream.write("throttle 0\nbrake 0\n".as_bytes()).unwrap();
                    }
                    Action::None => {}
                }
            }

            println!();
            word.clear();
        } else {
            print!("{:}", buf[0] as char);
            word.push(buf[0] as char);
        }
    }
}
