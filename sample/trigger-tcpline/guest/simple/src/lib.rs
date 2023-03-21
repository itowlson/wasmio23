wit_bindgen_rust::export!("../../wit/tcp-line.wit");

struct TcpLine;

impl tcp_line::TcpLine for TcpLine {
    fn handle_line(line: String) -> String {
        println!("Received line: {line}");
        "HELLO FROM SPIN\n".to_owned()
    }
}
