use gba::prelude::*;

extern "C" fn irq_handler(irq: IrqBits) {
    let mut handled_interrupts = IrqBits::new();

    if irq.vblank() {
        super::gba::update_input();
        handled_interrupts = handled_interrupts.with_vblank(true);
    }

    IF.write(handled_interrupts);
}

pub fn init_irq() {
    IE.write(IrqBits::new().with_vblank(true));

    IME.write(true);

    RUST_IRQ_HANDLER.write(Some(irq_handler));
}
