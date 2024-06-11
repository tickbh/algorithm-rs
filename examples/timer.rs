use algorithm::{Timer, TimerWheel};

fn main() {
    let mut timer = TimerWheel::new();
    timer.append_timer_wheel(12, 60 * 60, "HourWheel");
    timer.append_timer_wheel(60, 60, "MinuteWheel");
    timer.append_timer_wheel(60, 1, "SecondWheel");

    timer.add_timer(30);
    assert_eq!(timer.get_delay_id(), 30);
    timer.add_timer(149);
    assert_eq!(timer.get_delay_id(), 30);
    let t = timer.add_timer(600);
    assert_eq!(timer.get_delay_id(), 30);
    timer.add_timer(1);
    assert_eq!(timer.get_delay_id(), 1);
    timer.del_timer(t);

    let val = timer.update_deltatime(30).unwrap();
    assert_eq!(val, vec![1, 30]);

    timer.add_timer(2);

    let val = timer.update_deltatime(119).unwrap();
    assert_eq!(val, vec![2, 149]);

    assert!(timer.is_empty());
}
