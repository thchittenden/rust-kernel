use mem::phys::FrameReserve;
logger_init!(Trace);

pub fn test() {
    trace!("\ntesting mem");
    let resv = FrameReserve::new();
    resv.reserve(3).unwrap();
    let f1 = resv.get_frame();
    let f2 = resv.get_frame();
    let f3 = resv.get_frame();
    let f4 = resv.get_frame_unreserved();
    trace!("f1: {:?}", f1);
    trace!("f2: {:?}", f2);
    trace!("f3: {:?}", f3);
    trace!("f4: {:?}", f4);
}
