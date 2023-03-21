wit_bindgen_rust::export!("../../wit/tcp-line.wit");

struct TcpLine;

impl tcp_line::TcpLine for TcpLine {
    fn handle_line(line: String) -> String {
        println!("Received line: {line}");

        let command = parse_command(&line);
        command.run()
    }
}

fn parse_command(line: &str) -> Command {
    if line.contains("cat") {
        Command::CatFact
    } else if line.contains("dog") {
        Command::DogFact
    } else {
        Command::Unknown
    }
}

#[derive(serde::Deserialize)]
struct Fact {
    pub fact: String,
}

enum Command {
    CatFact,
    DogFact,
    Unknown,
}

impl Command {
    fn run(&self) -> String {
        match self {
            Self::CatFact => {
                let fact = random_fact("animal/cat");
                format!("{}\n", fact.fact)
            }
            Self::DogFact => {
                let fact = random_fact("animal/dog");
                format!("{}\n", fact.fact)
            }
            Self::Unknown => {
                "You have chosen... unwisely.\n".to_owned()
            }
        }
    }
}

fn random_fact(category: &str) -> Fact {
    let request = http::Request::builder()
        .uri(format!("https://some-random-api.ml/{category}"))
        .body(None)
        .unwrap();
    let response = spin_sdk::outbound_http::send_request(request).unwrap();
    let body = response.body().as_ref().unwrap();
    serde_json::from_slice(body).unwrap()
}
