// Не открывать лишнюю консоль на Windows в релизе.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    ntfy_zero_inbox_lib::run();
}
