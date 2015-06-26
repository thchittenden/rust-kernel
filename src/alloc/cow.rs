enum Cow<T> {
    Data(T),
    Ref(*const T)
}

struct Cow<T> {
    data: T
}

impl<T: Clone> for Cow {
    
}
