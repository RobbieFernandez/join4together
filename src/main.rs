#![no_std]
#![no_main]

pub mod resources;

#[panic_handler]
fn panic_handler(_: &core::panic::PanicInfo) -> ! {
  loop {}
}

#[no_mangle]
extern "C" fn main() -> ! {
  loop {}
}
