#![no_std]
#![no_main]

use core::panic::PanicInfo;

#[no_mangle]
unsafe extern "C" fn _start() -> ! {
    // csrr is a pseudo-instructions where rs1 = x0
    // csrrw rd, csr, rs1
    // dataflow rd <- csr <- rs1
    core::arch::asm!(
        "csrr a1, mhartid",
        "ld a0, 64(zero)",
        "li a7, 8",
        "ecall",
        options(noreturn)
    )
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
