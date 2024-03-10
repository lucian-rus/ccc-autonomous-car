use std::{
    io::{Read, Write},
    net,
};

/* update this based on target distance */
const TARGET_DISTANCE: f32 = 1000.0;

const MAX_SPEED_VALUE: f32 = 100.0;
const MAX_BRAKE_VALUE: f32 = 100.0;

const MIN_SPEED_VALUE: f32 = 0.0;
const MIN_BRAKE_VALUE: f32 = 0.0;

/* environemnt data */
struct Environment {
    speedlimit_current: f32,
    speedlimit_distance: f32,
    speedlimit_next: f32,

    trafficlight_distance: f32,
    trafficlight_time: f32,
    trafficlight_state: String,
}

/* car telemetry  */
struct Telemetry {
    speed: f32,
    distance: f32,
    throttle: f32,
    brake: f32,

    env: Environment,
}

impl Telemetry {
    fn new() -> Telemetry {
        Telemetry {
            speed: 0.00,
            distance: 0.00,
            throttle: 0.00,
            brake: 0.00,

            env: Environment {
                speedlimit_current: 0.00,
                speedlimit_distance: 0.00,
                speedlimit_next: 0.00,

                trafficlight_distance: 0.00,
                trafficlight_time: 0.00,
                trafficlight_state: String::from("none"),
            },
        }
    }

    fn set_speed(&mut self, input: f32) {
        self.speed = input;
    }

    fn set_distance(&mut self, input: f32) {
        self.distance = input;
    }

    /* can probably find a better name */
    fn set_motion(&mut self, throttle: f32, brake: f32) {
        self.throttle = throttle;
        self.brake = brake;
    }

    fn set_speedlimit(&mut self, current: f32, distance: f32, next: f32) {
        self.env.speedlimit_current = current;
        self.env.speedlimit_distance = distance;
        self.env.speedlimit_next = next;
    }

    fn set_traffic_light(&mut self, distance: f32, state: String, time: f32) {
        self.env.trafficlight_distance = distance;
        self.env.trafficlight_state = state;
        self.env.trafficlight_time = time;
    }
}

/* parse received line and check if should update server-side */
fn parse_line(input: &Vec<char>, output: &mut Telemetry) -> bool {
    let parsed_input = String::from_iter(input);

    if parsed_input == "update" {
        return true;
    }

    let values = parsed_input.split(' ').collect::<Vec<&str>>();

    match values[0] {
        "speed" => output.set_speed(values[1].parse::<f32>().unwrap()),
        "distance" => output.set_distance(values[1].parse::<f32>().unwrap()),
        "speedlimit" => output.set_speedlimit(
            values[1].parse::<f32>().unwrap(),
            values[2].parse::<f32>().unwrap(),
            values[3].parse::<f32>().unwrap(),
        ),
        "trafficlight" => output.set_traffic_light(
            values[1].parse::<f32>().unwrap(),
            String::from(values[2]),
            values[3].parse::<f32>().unwrap(),
        ),
        _ => {}
    }

    false
}

fn calculate_target_speed(distance: f32, time: f32) -> f32 {
    /* get target speed in km/h and add 5 as buffer to make it just in time */
    let f = ((distance / time) * 3.6) + 5.0;
    println!("target_speed: {:}", f);
    f
}

fn get_brake_by_time(speed: f32, target_speed: f32) -> f32 {
    /* get time to target speed + 2s buffer */
    let f = (((speed / 3.6) - (target_speed / 3.6)) / 6.0) + 2.0;
    println!("brake_by_time: {:}, {:}, {:}", speed, target_speed, f);
    f
}

fn update(input: &mut Telemetry) {
    /* default is  */
    // if input.env.trafficlight_state == "Green" {
    //     input.set_motion(
    //         calculate_target_speed(input.env.trafficlight_distance, input.env.trafficlight_time),
    //         MIN_BRAKE_VALUE,
    //     );
    // }

    input.set_motion(MAX_SPEED_VALUE, MIN_BRAKE_VALUE);

    if input.env.trafficlight_state == "Red"
        || input.env.trafficlight_state == "Yellow"
        || input.env.trafficlight_state == "RedYellow"
    {
        let f = get_brake_by_time(input.speed, 0.0);
        println!("{:} {:}\n", input.env.trafficlight_distance, f);

        if input.env.trafficlight_time < f {
            input.set_motion(MIN_SPEED_VALUE, MAX_BRAKE_VALUE);
        }
    }

    // /* set cruise speed */
    // if input.speed > input.env.speedlimit_current - 2.5 {
    //     input.set_motion(MIN_SPEED_VALUE, MIN_BRAKE_VALUE);
    // }
}

pub fn run() {
    let mut stream = net::TcpStream::connect("127.0.0.1:7000").unwrap();
    let mut word: Vec<char> = vec![];

    let mut telemetry: Telemetry = Telemetry::new();
    loop {
        /* read byte-by-byte */
        let mut buf = [0; 1];
        /* discard return */
        _ = stream.read(&mut buf);

        /* check if last received char is a newline */
        if buf[0] == 10 {
            if parse_line(&word, &mut telemetry) == true {
                update(&mut telemetry);

                /* discard return */
                _ = stream
                    .write(
                        format!(
                            "throttle {:}\nbrake {:}\n",
                            telemetry.throttle, telemetry.brake
                        )
                        .as_bytes(),
                    )
                    .unwrap();
            }

            // println!();
            word.clear();
        } else {
            // print!("{:}", buf[0] as char);
            word.push(buf[0] as char);
        }
    }
}
