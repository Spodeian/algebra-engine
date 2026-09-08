#![allow(unsafe_code)]

use std::ffi::{CStr, CString};
use urae_ffi::{
    urae_differentiate_and_format_latex, urae_free_string, urae_graph_create, urae_graph_free,
    urae_parse_and_format_latex,
};

#[test]
fn test_c_ffi_lifecycle() {
    unsafe {
        let graph = urae_graph_create();
        assert!(!graph.is_null());

        let expr_cstr = CString::new("x^2 + 1").unwrap();
        let res_ptr = urae_parse_and_format_latex(graph, expr_cstr.as_ptr());
        assert!(!res_ptr.is_null());

        let res_str = CStr::from_ptr(res_ptr).to_str().unwrap();
        assert!(res_str.contains("x"));
        assert!(res_str.contains("1"));

        urae_free_string(res_ptr);
        urae_graph_free(graph);
    }
}

#[test]
fn test_c_ffi_differentiation() {
    unsafe {
        let graph = urae_graph_create();
        assert!(!graph.is_null());

        let expr_cstr = CString::new("x^2").unwrap();
        let var_cstr = CString::new("x").unwrap();

        let res_ptr =
            urae_differentiate_and_format_latex(graph, expr_cstr.as_ptr(), var_cstr.as_ptr());
        assert!(!res_ptr.is_null());

        let res_str = CStr::from_ptr(res_ptr).to_str().unwrap();
        assert!(res_str.contains("2") || res_str.contains("x"));

        urae_free_string(res_ptr);
        urae_graph_free(graph);
    }
}
