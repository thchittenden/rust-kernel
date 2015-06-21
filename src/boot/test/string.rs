use collections::string::String;
logger_init!(Trace);

pub fn test() {
    trace!("\ntesting string");

    let mut x = String::new();
    trace!("empty string: {}", x);

    x.append("blah");
    trace!("string1: {}", x);
    
    x.append(" blah");
    trace!("string2: {}", x);

    x.append(" blah final");
    trace!("string3: {}", x);

    let mut y = String::from_str("prefix: ");

    y.append(x.as_str());
    trace!("string4: {}", y);
}
