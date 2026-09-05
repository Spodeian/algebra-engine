import re
with open('crates/urae-notebook/src/app.rs', 'r', encoding='utf-8') as f:
    content = f.read()

content = content.replace('|ctx, _class| {\n                                egui::CentralPanel::default().show(ui, |ui| {', '|ui, _class| {\n                                egui::CentralPanel::default().show(ui, |ui| {')

with open('crates/urae-notebook/src/app.rs', 'w', encoding='utf-8') as f:
    f.write(content)
