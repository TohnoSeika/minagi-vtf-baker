// 这个文件是桃华帮 Minagi 写的哦 ♡
// Prevents a console window from appearing on Windows in release builds
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    minagi_vtf_baker::run()
}
