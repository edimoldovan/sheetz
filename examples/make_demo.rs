//! Generates a small demo workbook: `cargo run --example make_demo [path]`
use ironcalc::base::UserModel;
use ironcalc::export::save_xlsx_to_writer;
use std::io::BufWriter;

fn main() {
    let path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "demo.xlsx".to_string());
    let mut um = UserModel::new_empty("demo", "en", "UTC", "en").unwrap();
    for (i, (item, qty, price)) in [
        ("Widgets", 12, 3.5),
        ("Gadgets", 5, 12.0),
        ("Sprockets", 40, 0.85),
        ("Doohickeys", 7, 22.4),
    ]
    .iter()
    .enumerate()
    {
        let r = i as i32 + 2;
        um.set_user_input(0, r, 1, item).unwrap();
        um.set_user_input(0, r, 2, &qty.to_string()).unwrap();
        um.set_user_input(0, r, 3, &price.to_string()).unwrap();
        um.set_user_input(0, r, 4, &format!("=B{r}*C{r}")).unwrap();
    }
    for (c, h) in ["Item", "Qty", "Price", "Total"].iter().enumerate() {
        um.set_user_input(0, 1, c as i32 + 1, h).unwrap();
    }
    um.set_user_input(0, 7, 1, "Grand total").unwrap();
    um.set_user_input(0, 7, 4, "=SUM(D2:D5)").unwrap();
    let file = std::fs::File::create(&path).unwrap();
    save_xlsx_to_writer(um.get_model(), BufWriter::new(file)).unwrap();
    println!("wrote {path}");
}
