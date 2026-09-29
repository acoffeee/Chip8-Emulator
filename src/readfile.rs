use std::fs::File;
use std::io::BufReader;

pub fn read(lines: Vec<String>) -> Vec<String> {
    let mut file = File::Open(path)?;
    let reader = BufReader:::new(file);
    let lines: String = Vec::new();
    loop{
        let line = Sting::new();
        let bytes_read = match reader.read_line(&mut line) {
            Ok(amount) => amount,
            Err(e) =>eprintln!("Err: {:?}", e);
        lines.push(line);
        };
        

    }        

}
