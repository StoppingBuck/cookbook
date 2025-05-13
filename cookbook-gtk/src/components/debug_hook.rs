use gtk4::prelude::*;
use gtk4::glib;
use std::time::{Duration, Instant};
use std::cell::RefCell;

thread_local! {
    static DEBUG_STATE: RefCell<DebugState> = RefCell::new(DebugState {
        last_check: Instant::now(),
        check_count: 0,
    });
}

struct DebugState {
    last_check: Instant,
    check_count: usize,
}

pub fn install_debug_hook() {
    println!("Installing debug hook for GUI updates");
    
    // Run periodic checks to monitor UI state
    glib::timeout_add_local(Duration::from_millis(1000), || {
        DEBUG_STATE.with(|state| {
            let mut state = state.borrow_mut();
            state.check_count += 1;
            println!("Debug hook check #{}: {} seconds since last check", 
                state.check_count, 
                state.last_check.elapsed().as_secs_f32());
            state.last_check = Instant::now();
        });
        
        // Continue running the timer
        glib::Continue(true)
    });
}
