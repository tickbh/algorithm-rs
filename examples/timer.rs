
use algorithm::{TimerWheel, Timer};

struct Test {
    s: usize,
}

impl Timer for Test {
    fn when(&self) -> usize {
        self.s
    }
}

fn main() {
    let mut timer = TimerWheel::new();
    timer.append_timer_wheel(12, 60*60, "HourWheel");
    timer.append_timer_wheel(60, 60, "MinuteWheel");
    timer.append_timer_wheel(60, 1, "SecondWheel");

    timer.append_timer_wheel(24, 20, "wheel");
    timer.append_timer_wheel(20, 1, "wheel");

    timer.add_timer(Test { s: 100} );
    println!("timer delay = {}", timer.get_delay_id());
    let t = timer.add_timer(Test { s: 600} );
    println!("timer delay = {}", timer.get_delay_id());
    timer.add_timer(Test { s: 1} );
    println!("timer delay = {}", timer.get_delay_id());
    // timer.del_timer(t);

    timer.update_deltatime_with_callback(20, &mut |_, v| {
        println!("vvv = {}", v.s);
    });
    timer.add_timer(Test { s: 2} );

    timer.update_deltatime_with_callback(80, &mut |_, v| {
        println!("vvv1 = {}", v.s);
    });

    let xx = 0;
    timer.update_deltatime_with_callback(380, &mut |t, v| {
        println!("vvv2 = {}", v.s);
        t.add_timer(v);
        println!("xxx = {}", xx);
    });
}