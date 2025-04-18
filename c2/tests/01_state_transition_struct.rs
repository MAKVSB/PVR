//! Run this file with `cargo test --test 01_state_transition_struct`.

//! This is a modified variant of the `03_state_transition` test from your home assignment.
//! Try to implement it using structs (without enums), and then later implement it using
//! enums in the assignment, and compare both approaches.

// TODO: Implement the `pc_transition` function.
// A computer can be in three states (off, running or sleeping).
// It can receive four events (turn on, turn off, pass some amount of time and mouse move).
//
// When the PC is running or sleeping, it remembers the time since it was started (`uptime`).
// When the PC is running, it also remembers `idle_time` (time since last mouse move).
// When the PC is sleeping, it also remembers `sleep_time` (time since going to sleep).
//

struct ComputerState {
    enabled: bool,
    sleeping: bool,
    uptime: u32,
    sleep_time: u32,
    idle_time: u32,
}

impl ComputerState {
    // Returns a computer that is turned off
    fn new_off() -> Self {
        return ComputerState { enabled: false, sleeping: false, uptime: 0, sleep_time: 0, idle_time: 0 };
    }

    // Returns a computer that is turned on
    fn new_on() -> Self {
        return ComputerState { enabled: true, sleeping: false, uptime: 0, sleep_time: 0, idle_time: 0 };
    }

    fn is_on(&self) -> bool {
        return self.enabled;
    }
    fn is_sleeping(&self) -> bool {
        return self.sleeping;
    }
    fn uptime(&self) -> u32 {
        return self.uptime;
    }
    fn idle_time(&self) -> u32 {
        return self.idle_time;
    }
    fn sleep_time(&self) -> u32 {
        return self.sleep_time;
    }
}

enum Event {
    TurnOn,
    TurnOff,
    PassTime(u32),
    MoveMouse,
}

// Here are the rules that the computer should abide by:
// 1) When `TurnOn` happens, if the PC is off, it switches to `Running`. Otherwise nothing happens.
// 2) When `TurnOff` happens, the PC switches to `Off`.
// 3) When `MoveMouse` happens:
//   - if the PC is sleeping, the PC switches to `Running`.
//   - if the PC is running, it resets its `idle_time` to zero.
// 4) When `PassTime(time)` happens, and the PC is on, it increments its `uptime` by `time`. Then:
//   - If the PC is running and its `idle_time` is larger than 1000, it switches to `Sleeping`.
//   - If the PC is sleeping and its `sleep_time` is larger than 500, it switches to `Off`.
fn pc_transition(mut computer: ComputerState, event: Event) -> ComputerState {
    match event {
        Event::TurnOn => {
            if !computer.enabled {
                return ComputerState::new_on();
            }
        },
        Event::TurnOff => {
            return ComputerState::new_off();
        },
        Event::PassTime(time) => {
            if computer.enabled {
                computer.uptime += time;
                if computer.sleeping {
                    computer.sleep_time += time;
                } else {
                    computer.idle_time += time;
                }

                if computer.idle_time > 1000 {
                    computer.sleeping = true;
                    computer.sleep_time = computer.idle_time - 1000;
                    computer.idle_time = 0;
                }
                if computer.sleeping && computer.sleep_time > 500 {
                    return ComputerState::new_off();
                }


            }
        },
        Event::MoveMouse => {
            match computer.sleeping {
                true => {
                    computer.sleeping = false;
                    computer.sleep_time = 0;
                },
                false => {
                    computer.idle_time = 0;
                },
            }
        },
    }

    return computer;
}

/// Below you can find a set of unit tests.
#[cfg(test)]
mod tests {
    use crate::{pc_transition, ComputerState, Event};

    #[test]
    fn turn_off_when_off() {
        // The matches!(<variable>, <pattern>) macro returns `true` if <variable> matches the
        // given <pattern>.
        // We could have nicer error messages with `assert_eq!`, but for that we would need to know
        // about traits first :) Stay tuned.

        let pc = ComputerState::new_off();
        let pc = pc_transition(pc, Event::TurnOff);
        assert!(!pc.is_on());
        assert!(!pc.is_sleeping());
        assert_eq!(pc.uptime(), 0);
        assert_eq!(pc.idle_time(), 0);
        assert_eq!(pc.sleep_time(), 0);
    }

    #[test]
    fn turn_off_when_running() {
        let pc = ComputerState::new_on();
        let pc = pc_transition(pc, Event::TurnOff);
        assert!(!pc.is_on());
        assert!(!pc.is_sleeping());
        assert_eq!(pc.uptime(), 0);
        assert_eq!(pc.idle_time(), 0);
        assert_eq!(pc.sleep_time(), 0);
    }

    #[test]
    fn turn_off_when_sleeping() {
        let pc = ComputerState::new_on();
        let pc = pc_transition(pc, Event::PassTime(1000));
        let pc = pc_transition(pc, Event::TurnOff);
        assert!(!pc.is_on());
        assert!(!pc.is_sleeping());
        assert_eq!(pc.uptime(), 0);
        assert_eq!(pc.idle_time(), 0);
        assert_eq!(pc.sleep_time(), 0);
    }

    #[test]
    fn turn_on_when_off() {
        let pc = ComputerState::new_off();
        let pc = pc_transition(pc, Event::TurnOn);

        assert!(pc.is_on());
        assert!(!pc.is_sleeping());
        assert_eq!(pc.uptime(), 0);
        assert_eq!(pc.idle_time(), 0);
        assert_eq!(pc.sleep_time(), 0);
    }

    #[test]
    fn turn_on_when_running() {
        let pc = ComputerState::new_on();
        let pc = pc_transition(pc, Event::TurnOn);
        let pc = pc_transition(pc, Event::TurnOn);

        assert!(pc.is_on());
        assert!(!pc.is_sleeping());
        assert_eq!(pc.uptime(), 0);
        assert_eq!(pc.idle_time(), 0);
        assert_eq!(pc.sleep_time(), 0);
    }

    #[test]
    fn turn_on_when_sleeping() {
        let pc = ComputerState::new_on();
        let pc = pc_transition(pc, Event::TurnOn);
        let pc = pc_transition(pc, Event::PassTime(1100));
        let pc = pc_transition(pc, Event::TurnOn);

        assert!(pc.is_on());
        assert!(pc.is_sleeping());
        assert_eq!(pc.uptime(), 1100);
        assert_eq!(pc.idle_time(), 0);
        assert_eq!(pc.sleep_time(), 100);
    }

    #[test]
    fn pass_time_off() {
        let pc = ComputerState::new_off();
        let pc = pc_transition(pc, Event::PassTime(1100));

        assert!(!pc.is_on());
        assert!(!pc.is_sleeping());
        assert_eq!(pc.uptime(), 0);
        assert_eq!(pc.idle_time(), 0);
        assert_eq!(pc.sleep_time(), 0);
    }

    #[test]
    fn pass_time_running() {
        let pc = ComputerState::new_on();
        let pc = pc_transition(pc, Event::PassTime(20));
        let pc = pc_transition(pc, Event::MoveMouse);
        let pc = pc_transition(pc, Event::PassTime(120));
        let pc = pc_transition(pc, Event::PassTime(123));

        assert!(pc.is_on());
        assert!(!pc.is_sleeping());
        assert_eq!(pc.uptime(), 263);
        assert_eq!(pc.idle_time(), 243);
        assert_eq!(pc.sleep_time(), 0);
    }

    #[test]
    fn pass_time_go_to_sleep() {
        let pc = ComputerState::new_on();
        let pc = pc_transition(pc, Event::PassTime(800));
        let pc = pc_transition(pc, Event::PassTime(320));

        assert!(pc.is_on());
        assert!(pc.is_sleeping());
        assert_eq!(pc.uptime(), 1120);
        assert_eq!(pc.idle_time(), 0);
        assert_eq!(pc.sleep_time(), 120);
    }

    #[test]
    fn pass_time_sleeping() {
        let pc = ComputerState::new_on();
        let pc = pc_transition(pc, Event::PassTime(1100));
        let pc = pc_transition(pc, Event::PassTime(320));

        assert!(pc.is_on());
        assert!(pc.is_sleeping());
        assert_eq!(pc.uptime(), 1420);
        assert_eq!(pc.idle_time(), 0);
        assert_eq!(pc.sleep_time(), 420);
    }

    #[test]
    fn pass_time_shutdown() {
        let pc = ComputerState::new_on();
        let pc = pc_transition(pc, Event::PassTime(800));
        let pc = pc_transition(pc, Event::PassTime(10000));

        assert!(!pc.is_on());
        assert!(!pc.is_sleeping());
        assert_eq!(pc.uptime(), 0);
        assert_eq!(pc.idle_time(), 0);
        assert_eq!(pc.sleep_time(), 0);
    }

    #[test]
    fn pass_time_sleeping_turn_off() {
        let pc = ComputerState::new_on();
        let pc = pc_transition(pc, Event::PassTime(800));
        let pc = pc_transition(pc, Event::PassTime(120));
        let pc = pc_transition(pc, Event::PassTime(700));

        assert!(!pc.is_on());
        assert!(!pc.is_sleeping());
        assert_eq!(pc.uptime(), 0);
        assert_eq!(pc.idle_time(), 0);
        assert_eq!(pc.sleep_time(), 0);
    }

    #[test]
    fn mouse_move_off() {
        let pc = ComputerState::new_off();
        let pc = pc_transition(pc, Event::PassTime(800));
        let pc = pc_transition(pc, Event::TurnOff);
        let pc = pc_transition(pc, Event::MoveMouse);

        assert!(!pc.is_on());
        assert!(!pc.is_sleeping());
        assert_eq!(pc.uptime(), 0);
        assert_eq!(pc.idle_time(), 0);
        assert_eq!(pc.sleep_time(), 0);
    }

    #[test]
    fn mouse_move_running() {
        let pc = ComputerState::new_on();
        let pc = pc_transition(pc, Event::PassTime(500));
        let pc = pc_transition(pc, Event::PassTime(100));
        let pc = pc_transition(pc, Event::MoveMouse);

        assert!(pc.is_on());
        assert!(!pc.is_sleeping());
        assert_eq!(pc.uptime(), 600);
        assert_eq!(pc.idle_time(), 0);
        assert_eq!(pc.sleep_time(), 0);
    }

    #[test]
    fn mouse_move_wake() {
        let pc = ComputerState::new_on();
        let pc = pc_transition(pc, Event::PassTime(500));
        let pc = pc_transition(pc, Event::PassTime(600));
        let pc = pc_transition(pc, Event::MoveMouse);

        assert!(pc.is_on());
        assert!(!pc.is_sleeping());
        assert_eq!(pc.uptime(), 1100);
        assert_eq!(pc.idle_time(), 0);
        assert_eq!(pc.sleep_time(), 0);
    }

    #[test]
    fn complex_transition_1() {
        let mut pc = ComputerState::new_off();
        let events = [
            Event::TurnOn,
            Event::PassTime(100),
            Event::PassTime(50),
            Event::MoveMouse,
            Event::PassTime(500),
            Event::PassTime(600),
            Event::PassTime(100),
            Event::MoveMouse,
            Event::PassTime(20),
            Event::PassTime(100),
        ];
        for event in events {
            pc = pc_transition(pc, event);
        }
        assert!(pc.is_on());
        assert!(!pc.is_sleeping());
        assert_eq!(pc.uptime(), 1470);
        assert_eq!(pc.idle_time(), 120);
        assert_eq!(pc.sleep_time(), 0);
    }

    #[test]
    fn complex_transition_2() {
        let mut pc = ComputerState::new_off();
        let events = [
            Event::TurnOn,
            Event::PassTime(100),
            Event::PassTime(50),
            Event::MoveMouse,
            Event::PassTime(500),
            Event::PassTime(600),
            Event::TurnOff,
            Event::MoveMouse,
            Event::PassTime(600),
            Event::TurnOn,
            Event::PassTime(100),
            Event::MoveMouse,
            Event::PassTime(20),
            Event::PassTime(100),
            Event::PassTime(1000),
            Event::TurnOn,
            Event::PassTime(150),
        ];
        for event in events {
            pc = pc_transition(pc, event);
        }
        assert!(pc.is_on());
        assert!(pc.is_sleeping());
        assert_eq!(pc.uptime(), 1370);
        assert_eq!(pc.idle_time(), 0);
        assert_eq!(pc.sleep_time(), 270);
    }
}
