wit_bindgen_rust::export!("../../wit/tcp-line.wit");

struct TcpLine;

impl tcp_line::TcpLine for TcpLine {
    fn handle_line(line: String) -> String {
        println!("Received line: {line}");
        "¡HOLA FROM BARCELONA!\n".to_owned()
    }
}
