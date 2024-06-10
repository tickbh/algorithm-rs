
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
    timer.append_timer_wheel(24, 20, "wheel");
    timer.append_timer_wheel(20, 1, "wheel");

    timer.add_timer(Test { s: 100} );
    println!("timer delay = {}", timer.get_delay_id());
    timer.add_timer(Test { s: 600} );
    println!("timer delay = {}", timer.get_delay_id());
    timer.add_timer(Test { s: 1} );
    println!("timer delay = {}", timer.get_delay_id());

    timer.update_deltatime(20, &mut |v| {
        println!("vvv = {}", v.s);
    });
    timer.add_timer(Test { s: 2} );

    timer.update_deltatime(80, &mut |v| {
        println!("vvv1 = {}", v.s);
    });

    timer.update_deltatime(380, &mut |v| {
        println!("vvv2 = {}", v.s);
    });
}