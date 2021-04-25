#![no_std]
#![no_main]
#![feature(abi_avr_interrupt)]

// Pull in the panic handler from panic-halt
extern crate avr_device;
extern crate panic_halt;
use core::{cell::Cell, panic};

use avr_device::attiny85::Peripherals;

use avr_device::interrupt::{free, Mutex};

const PRESCALER: u32 = 1024;

static INTERRUPT_COUNTER: Mutex<Cell<u32>> = Mutex::new(Cell::new(0));

#[avr_device::entry]
fn main() -> ! {
    let dp = Peripherals::take().unwrap();

    timer_setup(&dp);

    loop {}
}

fn timer_setup(dp: &Peripherals) {
    // Set PB1 as output
    dp.PORTB.ddrb.write(|w| w.pb1().set_bit());

    // Normal mode timer
    dp.TC0.tccr0a.write(|w| {
        w.com0a()
            .disconnected()
            .com0b()
            .disconnected()
            .wgm0()
            .normal_top()
    });
    dp.TC0.tccr0b.write(|w| {
        // Prescaling
        match PRESCALER {
            8 => w.cs0().prescale_8(),
            64 => w.cs0().prescale_64(),
            256 => w.cs0().prescale_256(),
            1024 => w.cs0().prescale_1024(),
            _ => panic!(),
        }
        // Normal mode timer
        .wgm02()
        .clear_bit()
    });

    // enable global interrupts
    unsafe {
        avr_device::interrupt::enable();
    }

    // Reset the timer counter
    dp.TC0.tcnt0.reset();

    // Enable timer0 interrupt
    dp.TC0.timsk.write(|w| w.toie0().set_bit());

    // Set PB1
    dp.PORTB.portb.write(|w| w.pb1().set_bit());
}

#[allow(non_snake_case)]
#[avr_device::interrupt(attiny85)]
fn TIMER0_OVF() {
    let dp = Peripherals::take().unwrap();
    free(|cs| {
        let counter_cell = INTERRUPT_COUNTER.borrow(cs);
        let counter = counter_cell.get();
        if counter == 63 {
            if dp.PORTB.portb.read().pb1().bit_is_set() {
                dp.PORTB.portb.write(|w| w.pb1().set_bit());
            } else {
                dp.PORTB.portb.write(|w| w.pb1().clear_bit());
            }
        } else {
            counter_cell.set(counter + 1);
        }
    });
}
