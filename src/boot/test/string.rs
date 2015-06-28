use collections::string::String;
logger_init!(Trace);

pub fn test() {
    trace!("\ntesting string");

    let mut x = String::new();
    trace!("empty string: {}", x);

    assert!(x.append("blah").is_ok());
    trace!("string1: {}", x);
    
    assert!(x.append(" blah").is_ok());
    trace!("string2: {}", x);

    assert!(x.append(" blah final").is_ok());
    trace!("string3: {}", x);

    let mut y = String::from_str("prefix: ");

    assert!(y.append(x.as_str()).is_ok());
    trace!("string4: {}", y);

    let y2 = y.split_at(5).unwrap();
    trace!("y[0-4] = {}", y);
    trace!("y[5-.] = {}", y2);
}
