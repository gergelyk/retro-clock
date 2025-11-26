use esp_idf_sys::{
    esp_get_free_heap_size, esp_get_minimum_free_heap_size, pcTaskGetName,
    uxTaskGetStackHighWaterMark, uxTaskPriorityGet, xTaskGetCurrentTaskHandle,
};
use serde::Serialize;
use std::ffi::CStr;

#[derive(Serialize, Debug, Default, Clone)]
pub struct HeapStats {
    current_free_kb: u32,
    minimum_free_kb: u32, // Minimum free heap size ever recorded
}

pub fn get_heap_stats() -> HeapStats {
    let (free_heap_size, minimum_free_heap_size) = unsafe {
        (
            esp_get_free_heap_size() >> 10,
            esp_get_minimum_free_heap_size() >> 10,
        )
    };
    HeapStats {
        current_free_kb: free_heap_size,
        minimum_free_kb: minimum_free_heap_size,
    }
}

pub fn log_stack_stats() {
    unsafe {
        let free_stack_words = uxTaskGetStackHighWaterMark(core::ptr::null_mut());
        log::info!("STACK: free words={}", free_stack_words);
    }
}

pub fn log_thread_info() {
    let (task_handle, task_name_ptr) = unsafe {
        let handle = xTaskGetCurrentTaskHandle();
        let name = pcTaskGetName(handle);
        (handle, name)
    };
    let task_priority = unsafe { uxTaskPriorityGet(xTaskGetCurrentTaskHandle()) };

    let task_name = if !task_name_ptr.is_null() {
        unsafe { CStr::from_ptr(task_name_ptr).to_string_lossy().into_owned() }
    } else {
        "<unnamed>".to_string()
    };

    let thread = std::thread::current();
    let thread_name = thread.name().unwrap_or("<unnamed>");
    let thread_id = thread.id();

    let task_core = esp_idf_hal::cpu::core();

    log::info!(
        "THREAD: {}@{:?} TASK: {}@{:?}, core={:?}, priority={}",
        thread_name,
        thread_id,
        task_name,
        task_handle,
        task_core,
        task_priority,
    );
}
