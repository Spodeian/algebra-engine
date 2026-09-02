use algebra_core::SymbolTable;

#[test]
fn test_symbol_interning() {
    let table = SymbolTable::new();
    let id_x = table.get_or_intern("x");
    let id_y = table.get_or_intern("y");
    let id_x_again = table.get_or_intern("x");

    assert_eq!(id_x, id_x_again);
    assert_ne!(id_x, id_y);
    assert_eq!(table.resolve(id_x).unwrap(), "x");
    assert_eq!(table.resolve(id_y).unwrap(), "y");
}
