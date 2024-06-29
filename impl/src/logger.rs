use chrono::offset::Utc;

pub fn log(log: String) {
    println!("[{}] <=::=> {}", Utc::now(),log);
}
