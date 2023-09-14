use gba::prelude::*;

extern "C" fn irq_handler(irq: IrqBits) {
    let mut handled_interrupts = IrqBits::new();

    if irq.vblank() {
        super::gba::update_input();
        handled_interrupts = handled_interrupts.with_vblank(true);
    }

    if irq.timer2() {
        crate::audio::music::timer2_interrupt();
        handled_interrupts = handled_interrupts.with_timer2(true);
    }

    if irq.timer1() {
        crate::audio::music::timer1_interrupt();
        handled_interrupts = handled_interrupts.with_timer1(true);
    }

    IF.write(handled_interrupts);
}

pub fn init_irq() {
    IE.write(
        IrqBits::new()
            .with_vblank(true)
            .with_timer1(true)
            .with_timer2(true),
    );

    IME.write(true);

    RUST_IRQ_HANDLER.write(Some(irq_handler));
}
