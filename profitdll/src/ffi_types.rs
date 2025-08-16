// Mini-Disclaimer: Uso educacional/experimental; sem recomendação de investimento ou afiliação; sem remuneração de terceiros; Profit/ProfitDLL © Nelógica; veja README & DISCLAIMER.
//! Tipos auxiliares e utilidades FFI (wide strings, wrappers de ponteiro) usadas pela camada real da DLL Profit.
//!
//! Consulte o [MANUAL.md](../MANUAL.md#tipos-auxiliares) para detalhes dos tipos e convenções.
#![cfg(all(target_os = "windows", feature = "real_dll"))]

use std::ffi::c_void;

/// Converte ponteiro para wide string (UTF-16) terminado em nul em [`String`].
///
/// Retorna string vazia se ponteiro for nulo.
///
/// Usado para interoperabilidade com funções da DLL que retornam ponteiros UTF-16.
///
/// # Safety
/// O ponteiro deve ser válido e apontar para uma sequência UTF-16 terminada em nul.
pub unsafe fn utf16_ptr_to_string(ptr: *const u16) -> String {
    if ptr.is_null() {
        return String::new();
    }
    // Descobre comprimento até nul (scan linear).
    let mut len = 0usize;
    loop {
        let v = *ptr.add(len);
        if v == 0 {
            break;
        }
        len += 1;
    }
    let slice = std::slice::from_raw_parts(ptr, len);
    String::from_utf16_lossy(slice)
}

/// Wrapper para ponteiros retornados pela DLL que devem ser liberados via **FreePointer**.
///
/// Se a função não existir, não tentamos liberar (evita double free). Pode gerar leak leve.
///
/// Consulte o [MANUAL.md](../MANUAL.md#freepointer) para detalhes.
pub struct ForeignBuffer {
    ptr: *mut c_void,
    free_fn: Option<unsafe extern "system" fn(*mut c_void)>,
}

impl ForeignBuffer {
    pub fn new(ptr: *mut c_void, free_fn: Option<unsafe extern "system" fn(*mut c_void)>) -> Self {
        Self { ptr, free_fn }
    }
    pub fn as_ptr(&self) -> *mut c_void {
        self.ptr
    }
    pub fn is_null(&self) -> bool {
        self.ptr.is_null()
    }
    /// Consome o buffer e retorna o ponteiro bruto.
    ///
    /// # Safety
    /// O ponteiro retornado deve ser liberado manualmente, se necessário, e não deve ser usado após o drop do objeto.
    pub unsafe fn into_raw(self) -> *mut c_void {
        let p = self.ptr;
        std::mem::forget(self);
        p
    }
}

impl Drop for ForeignBuffer {
    fn drop(&mut self) {
        if self.ptr.is_null() {
            return;
        }
        if let Some(f) = self.free_fn {
            unsafe { f(self.ptr) };
        }
    }
}
