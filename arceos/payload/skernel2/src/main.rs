#![no_std]
#![no_main]

use core::panic::PanicInfo;

#[no_mangle]
unsafe extern "C" fn _start() -> ! {
    core::arch::asm!(
        "csrr a1, mhartid", //mhartid 用于区分当前运行在哪个 CPU 核心
        "ld a0, 64(zero)", // a0 = *((u64 *)0x40) 读取地址 0x40 的数据
        "li a7, 8", //在 RISC-V 里，a7 通常用作 系统调用号寄存器，所以这里意味着要执行系统调用号 8, ENV_CALL_FROM_U_OR_VU
        "ecall",
        options(noreturn)
    )
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
