import re

with open('crates/urae-notebook/src/app.rs', 'r', encoding='utf-8') as f:
    content = f.read()

# eframe::App trait signature
content = content.replace(
    'fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {',
    'fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {\n        let ctx = ui.ctx();'
)

# Panels
content = content.replace('egui::TopBottomPanel::top', 'egui::Panel::top')
content = content.replace('egui::TopBottomPanel::bottom', 'egui::Panel::bottom')
content = content.replace('egui::SidePanel::left', 'egui::Panel::left')
content = content.replace('egui::SidePanel::right', 'egui::Panel::right')

# Panel show calls inside App::ui (we can just replace all Panel...show(ctx, with Panel...show(ui,)
# CentralPanel::default().show(ctx, -> CentralPanel::default().show(ui,
content = re.sub(r'(egui::Panel::[^\)]+\)(?:[^s]|s(?!how))*?)\.show\(ctx,', r'\1.show(ui,', content)
content = re.sub(r'(egui::CentralPanel::default\(\)(?:[^s]|s(?!how))*?)\.show\(ctx,', r'\1.show(ui,', content)

# Panel sizes
content = content.replace('.default_width(', '.default_size(')
content = content.replace('.default_height(', '.default_size(')

# ui closures
content = content.replace('ui.close_menu()', 'ui.close()')
content = content.replace('raw_scroll_delta', 'smooth_scroll_delta')

# fonts mutable
content = content.replace('ui.fonts(|f| f.row_height', 'ui.fonts_mut(|f| f.row_height')

# Context style
content = content.replace('&ctx.style()', '&ctx.style_of(ctx.theme())')
content = content.replace('ctx.set_style(self.theme.to_style())', 'ctx.set_style_of(ctx.theme(), self.theme.to_style())')

# render methods missing references
content = content.replace('render_open_inspectors(ctx)', 'render_open_inspectors(&ctx)')
content = content.replace('CommandPalette::show(ctx,', 'CommandPalette::show(&ctx,')
content = content.replace('render_cad_machinery_palette(\n            ctx,', 'render_cad_machinery_palette(\n            &ctx,')

# Viewport closure
content = re.sub(r'show_viewport_immediate\(([^,]+),\n\s*egui::ViewportBuilder[^\n]+\n[^\n]+\n\s*\|ctx, _class\| \{', r'show_viewport_immediate(\1,\n                            egui::ViewportBuilder::default()\n                                .with_title(format!("URAE Standalone Graph: {}", expr_key))\n                                .with_inner_size([480.0, 320.0]),\n                            |ui, _class| {', content)

# rect height
content = content.replace('row.rect.height()', 'row.rect().height()')

# layouter
content = content.replace('let mut math_layouter = |ui: &egui::Ui, string: &str, wrap_width: f32| {', 'let mut math_layouter = |ui: &egui::Ui, string: &dyn egui::TextBuffer, wrap_width: f32| {\n                                let string = string.as_str();')

# CharIndex
content = content.replace('let clamped = char_idx.min(text.len());', 'let clamped = usize::from(char_idx).min(text.len());')
content = content.replace('find_numeric_literal_at(&self.state.session.raw_document_text, c_idx)', 'find_numeric_literal_at(&self.state.session.raw_document_text, usize::from(c_idx))')

# egui_plot Line::new
content = re.sub(r'Line::new\(([^)]+)\)\s*\.name\(([^)]+)\)', r'Line::new(\2, \1)', content)
content = content.replace('Line::new(plot_points)', 'Line::new("", plot_points)')

with open('crates/urae-notebook/src/app.rs', 'w', encoding='utf-8') as f:
    f.write(content)
