#![feature(type_ascription)]

macro_rules! state_machine {
    ( $sm: ident; $sn: ident; $($n: ident: $t: ty),* ) => {
        pub enum Trans {
            None,
            Pop,
            Push(Box<$sn>),
            Switch(Box<$sn>),
            Quit,
        }

        pub trait $sn {
            fn on_start(&mut self, $($n: $t),*) {}
            fn on_stop(&mut self, $($n: $t),*) {}
            fn on_pause(&mut self, $($n: $t),*) {}
            fn on_resume(&mut self, $($n: $t),*) {}
            /// Executed on every frame before updating, for use in reacting to events.
            fn handle_events(&mut self, $($n: $t),*) -> Trans {
                Trans::None
            }

            /// Executed repeatedly at stable, predictable intervals (1/60th of a second
            /// by default).
            fn fixed_update(&mut self, $($n: $t),*) -> Trans {
                Trans::None
            }

            /// Executed on every frame immediately, as fast as the engine will allow.
            fn update(&mut self, $($n: $t),*) -> Trans {
                Trans::None
            }
        }

        unsafe impl Sync for $sm {}
        unsafe impl Send for $sm {}

        pub struct $sm {
            running: bool,
            state_stack: Vec<Box<$sn>>,
        }

        impl $sm {
            pub fn new<S>(initial_state: S) -> $sm
            where
                S: $sn + 'static, {
                $sm {
                    running: false,
                    state_stack: vec![Box::new(initial_state)],
                }
            }

            pub fn is_running(&self) -> bool {
                self.running
            }

            pub fn start(
                &mut self,
                $($n: $t),*
            ) {
                if !self.running {
                    let state = self.state_stack.last_mut().unwrap();
                    state.on_start($($n: $t),*);
                    self.running = true;
                }
            }

            pub fn handle_events(
                &mut self,
                $($n: $t),*
            ) {
                if self.running {
                    let trans = match self.state_stack.last_mut() {
                        Some(state) => state.handle_events($($n: $t),*),
                        None => Trans::None,
                    };

                    self.transition(trans, $($n: $t),*);
                }
            }

            pub fn fixed_update(
                &mut self,
                $($n: $t),*
            ) {
                if self.running {
                    let trans = match self.state_stack.last_mut() {
                        Some(state) => state.fixed_update($($n: $t),*),
                        None => Trans::None,
                    };

                    self.transition(trans, $($n: $t),*);
                }
            }

            pub fn update(
                &mut self,
                $($n: $t),*
            ) {
                if self.running {
                    let trans = match self.state_stack.last_mut() {
                        Some(state) => state.update($($n: $t),*),
                        None => Trans::None,
                    };

                    self.transition(trans, $($n: $t),*);
                }
            }

            fn transition(
                &mut self,
                request: Trans,
                $($n: $t),*
            ) {
                if self.running {
                    match request {
                        Trans::None => (),
                        Trans::Pop => self.pop($($n: $t),*),
                        Trans::Push(state) => self.push(state, $($n: $t),*),
                        Trans::Switch(state) => self.switch(state, $($n: $t),*),
                        Trans::Quit => self.stop($($n: $t),*),
                    }
                }
            }

            fn switch(
                &mut self,
                state: Box<$sn>,
                $($n: $t),*
            ) {
                if self.running {
                    if let Some(mut state) = self.state_stack.pop() {
                        state.on_stop($($n: $t),*)
                    }

                    self.state_stack.push(state);
                    let state = self.state_stack.last_mut().unwrap();
                    state.on_start($($n: $t),*);
                }
            }

            fn push(
                &mut self,
                state: Box<$sn>,
                $($n: $t),*
            ) {
                if self.running {
                    if let Some(state) = self.state_stack.last_mut() {
                        state.on_pause($($n: $t),*);
                    }

                    self.state_stack.push(state);
                    let state = self.state_stack.last_mut().unwrap();
                    state.on_start($($n: $t),*);
                }
            }

            fn pop(
                &mut self,
                $($n: $t),*
            ) {
                if self.running {
                    if let Some(mut state) = self.state_stack.pop() {
                        state.on_stop($($n: $t),*);
                    }

                    if let Some(state) = self.state_stack.last_mut() {
                        state.on_resume($($n: $t),*);
                    } else {
                        self.running = false;
                    }
                }
            }

            fn stop(
                &mut self,
                $($n: $t),*
            ) {
                if self.running {
                    while let Some(mut state) = self.state_stack.pop() {
                        state.on_stop($($n: $t),*);
                    }

                    self.running = false;
                }
            }
        }
    }
}

state_machine!(TestStateMachine; TestState; _a: &mut isize, _b: isize);

pub struct Test;
impl TestState for Test {
    fn on_start(&mut self, a: &mut isize, b: isize) {
        *a += b;
    }

    fn on_resume(&mut self, a: &mut isize, b: isize) {
        self.on_start(a, b);
    }

    fn update(&mut self, _a: &mut isize, _b: isize) -> Trans {
        Trans::Push(Box::new(Test))
    }

    fn fixed_update(&mut self, _a: &mut isize, _b: isize) -> Trans {
        Trans::Pop
    }
}

#[test]
fn sm_test() {
    let mut sm = TestStateMachine::new(Test);

    let mut a = 0;
    let b = 10;

    sm.start(&mut a, b);
    assert!(a == 10);

    sm.update(&mut a, b);
    assert!(a == 20);

    sm.fixed_update(&mut a, b);
    assert!(a == 30);

    sm.fixed_update(&mut a, b);
    assert!(a == 30);

    assert!(!sm.is_running())
}
