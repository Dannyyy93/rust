//@ only-wasm32
//@ compile-flags: -C panic=unwind -Z emscripten-wasm-eh

#![crate_type = "lib"]
#![feature(core_intrinsics, wasm_exception_handling_intrinsics)]

extern "C-unwind" {
    fn may_panic();
}

extern "C" {
    fn log_number(number: usize);
}

struct LogOnDrop;

impl Drop for LogOnDrop {
    fn drop(&mut self) {
        unsafe {
            log_number(0);
        }
    }
}

// CHECK-LABEL: @test_cleanup() {{.*}} @__gxx_wasm_personality_v0
#[no_mangle]
pub fn test_cleanup() {
    let _log_on_drop = LogOnDrop;
    unsafe {
        may_panic();
    }

    // CHECK-NOT: call
    // CHECK: invoke void @may_panic()
    // CHECK: %cleanuppad = cleanuppad within none []
}

// CHECK-LABEL: @test_rtry() {{.*}} @__gxx_wasm_personality_v0
#[no_mangle]
pub fn test_rtry() {
    unsafe {
        core::intrinsics::catch_unwind(
            |_| {
                may_panic();
            },
            core::ptr::null_mut(),
            |data, exception| {
                log_number(data as usize);
                log_number(exception as usize);
            },
        );
    }

    // CHECK-NOT: call
    // CHECK: invoke void @may_panic()
    // CHECK: {{.*}} = catchswitch within none [label {{.*}}] unwind to caller
    // CHECK: {{.*}} = catchpad within {{.*}} [ptr null]
    // CHECK: catchret
}

// Make sure the intrinsic is not inferred as nounwind. This is a regression test for #132416.
// CHECK-LABEL: @test_intrinsic() {{.*}} @__gxx_wasm_personality_v0
#[no_mangle]
pub fn test_intrinsic() {
    let _log_on_drop = LogOnDrop;
    unsafe {
        core::arch::wasm32::throw::<0>(core::ptr::null_mut());
    }

    // CHECK-NOT: call
    // CHECK: invoke void @llvm.wasm.throw(i32 noundef 0, ptr noundef null)
    // CHECK: %cleanuppad = cleanuppad within none []
}
