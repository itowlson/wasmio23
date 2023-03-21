wit_bindgen_rust::export!("../../wit/tcp-line.wit");

struct TcpLine;

impl tcp_line::TcpLine for TcpLine {
    fn handle_line(line: String) -> String {
        println!("Received line: {line}");
        // I admit this is not actually very fancy
        "¡HOLA FROM BARCELONA!\n".to_owned()
    }
}
