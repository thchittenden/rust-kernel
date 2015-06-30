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

    // Test pop.
    let mut z1 = String::from_str("abc");
    assert!(z1.pop().unwrap() == 'c');
    assert!(z1.as_str() == "ab");
    assert!(z1.pop().unwrap() == 'b');
    assert!(z1.as_str() == "a");
    assert!(z1.pop().unwrap() == 'a');
    assert!(z1.as_str() == "");
    assert!(z1.pop().is_none());
    assert!(z1.as_str() == "");

    // Test push/pop.
    let mut z2 = String::new();
    assert!(z2.push('x').is_ok());
    assert!(z2.as_str() == "x");
    assert!(z2.push('y').is_ok());
    assert!(z2.as_str() == "xy");
    assert!(z2.pop().unwrap() == 'y');
    assert!(z2.pop().unwrap() == 'x');
    assert!(z2.pop().is_none());
    assert!(z2.as_str() == "");
}
