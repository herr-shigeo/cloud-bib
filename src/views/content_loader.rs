use std::error;
use std::fs;

pub fn read_file(file_path: &str) -> Result<String, Box<dyn error::Error>> {
    let data = fs::read_to_string(file_path)?;
    Ok(data)
}

pub fn read_csv(file_path: &str) -> Result<Vec<csv::StringRecord>, Box<dyn error::Error>> {
    let csv = read_file(&file_path)?;

    let mut records = vec![];
    let mut reader = csv::Reader::from_reader(csv.as_bytes());
    for record in reader.records() {
        let record = record?;
        records.push(record);
    }
    Ok(records)
}
