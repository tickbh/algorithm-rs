use algorithm::TimerWheel;

fn main() {
    // let mut timer = TimerWheel::new();
    // timer.append_timer_wheel(60, "SecondWheel");
    // timer.append_timer_wheel(60, "MinuteWheel");
    // timer.append_timer_wheel(12, "HourWheel");

    // timer.add_timer(30);
    // assert_eq!(timer.get_delay_id(), 30);
    // timer.add_timer(149);
    // assert_eq!(timer.get_delay_id(), 30);
    // let t = timer.add_timer(600);
    // assert_eq!(timer.get_delay_id(), 30);
    // timer.add_timer(1);
    // assert_eq!(timer.get_delay_id(), 1);
    // timer.del_timer(t);
    // timer.add_timer(150);
    // assert_eq!(timer.get_delay_id(), 1);

    // let val = timer.update_deltatime(30).unwrap();
    // assert_eq!(val.iter().map(|(_, v)| *v).collect::<Vec<usize>>(), vec![1, 30]);

    // timer.add_timer(2);

    // let val = timer.update_deltatime(119).unwrap();
    // assert_eq!(val.iter().map(|(_, v)| *v).collect::<Vec<usize>>(), vec![2, 149]);

    // let val = timer.update_deltatime(1).unwrap();
    // assert_eq!(val.iter().map(|(_, v)| *v).collect::<Vec<usize>>(), vec![150]);
    
    // assert!(timer.is_empty());

    let mut timer = TimerWheel::new();
    timer.set_one_step(5);
    timer.append_timer_wheel(200, "MillisWheel");
    timer.append_timer_wheel(60, "SecondWheel");
    timer.append_timer_wheel(60, "MinuteWheel");

    timer.add_timer(1000);

    let val = timer.update_deltatime(2000).unwrap();
    println!("val = {:?}", val);

    
    let val = timer.update_deltatime(2000);
    println!("val = {:?}", val);
    
}
