use std::{
    io::{Read, Write},
    net,
};

struct Telemetry {
    speed: f32,
    distance: f32,
    time: f32,

    current_speedlimit: f32,
    distance_to_speedlimit: f32,
    next_speedlimit: f32,

    traffic_light_distance: f32,
    traffic_light_state: String,
    traffic_light_remaining_time: f32,
}

#[derive(PartialEq)]
enum TelemetryData {
    Speed,
    Distance,
    Time,
    Update,
    SpeedLimit,
    TrafficLight,
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

    if values[0] == "speed" {
        output.speed = values[1].parse::<f32>().unwrap();
        telemetry = TelemetryData::Speed;
    }

    if values[0] == "time" {
        output.time = values[1].parse::<f32>().unwrap();
        telemetry = TelemetryData::Time;
    }

    if values[0] == "distance" {
        output.distance = values[1].parse::<f32>().unwrap();
        telemetry = TelemetryData::Distance;
    }

    if values[0] == "speedlimit" {
        output.current_speedlimit = values[1].parse::<f32>().unwrap();
        output.distance_to_speedlimit = values[2].parse::<f32>().unwrap();
        output.next_speedlimit = values[3].parse::<f32>().unwrap();

        telemetry = TelemetryData::SpeedLimit;
    }

    if values[0] == "trafficlight" {
        output.traffic_light_distance = values[1].parse::<f32>().unwrap();
        output.traffic_light_state = String::from(values[2]);
        output.traffic_light_remaining_time = values[3].parse::<f32>().unwrap();

        telemetry = TelemetryData::TrafficLight;
    }

    telemetry
}

fn execute_action(input: &Telemetry) -> Action {
    let mut action = Action::None;

    if input.distance > 1500.0 {
        action = Action::Brake;
    }

    if input.distance < 1500.0 {
        action = Action::Accelerate;

        let mut distance_to_brake = input.current_speedlimit - input.next_speedlimit;
        distance_to_brake = distance_to_brake.abs() * 1.5;

        if input.current_speedlimit - input.speed > 30.0 {
            action = Action::AccelerateTempered;
        }

        if input.speed > input.current_speedlimit - 2.0 {
            action = Action::Cruise;
        }

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
    let distance_to_brake = (input.speed / 3.6) * 2.0;

    if (input.traffic_light_distance < distance_to_brake || input.traffic_light_distance < 10.0)
        && (input.traffic_light_state == "Red"
            || input.traffic_light_state == "Yellow"
            || input.traffic_light_state == "RedYellow")
    {
        action = Action::Brake;
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
        traffic_light_distance: 0.00,
        traffic_light_remaining_time: 0.00,
        traffic_light_state: String::from("none"),
    };

    let mut accel_value = 21;
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
                        _ = stream
                            .write(format!("throttle {:}\nbrake 0\n", accel_value).as_bytes())
                            .unwrap();

                        if accel_value > 75 {
                            accel_value -= 5;
                        }
                        if accel_value < 75 {
                            accel_value += 5;
                        }
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
