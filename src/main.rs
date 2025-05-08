#![no_std]
#![no_main]

use crossbeam::atomic::AtomicCell;
use pc_keyboard::DecodedKey;
use pluggable_interrupt_os::{vga_buffer::clear_screen, HandlerTable};
use simple_swim_template::SwimInterface;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    HandlerTable::new()
        .keyboard(key)
        .timer(tick)
        .startup(startup)
        .cpu_loop(cpu_loop)
        .start()
}

static LAST_KEY: AtomicCell<Option<DecodedKey>> = AtomicCell::new(None);
static TICKED: AtomicCell<bool> = AtomicCell::new(false);

fn cpu_loop() -> ! {
    let mut kernel = SwimInterface::default();
    loop {
        if let Ok(_) = TICKED.compare_exchange(true, false) {
            kernel.tick();
        }
        
        if let Ok(k) = LAST_KEY.fetch_update(|k| if k.is_some() {Some(None)} else {None}) {
            if let Some(k) = k {
                kernel.key(k);
            }
        }
        if let Some(program) = self.programs.first_mut() {
            let doc_output = DocumentOutput {
                window: &mut self.windows[self.active_window],
            };
    
            match program.tick(&mut doc_output) {
                InterpreterOutput::AwaitInput => {
                    self.wait_for_input();
                },
                _ => {
                    self.programs.remove(0);
                },
            }
        }
    }
}

fn key(key: DecodedKey) {
    LAST_KEY.store(Some(key));
}

fn tick() {
    TICKED.store(true);
}

fn startup() {
    clear_screen();
}
