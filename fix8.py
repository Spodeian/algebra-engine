import re

with open('crates/urae-notebook/src/app.rs', 'r', encoding='utf-8') as f:
    content = f.read()

replacements = [
    ('self.setup_fonts(ctx);', 'self.setup_fonts(&ctx);'),
    ('self.poll_storage_diagnostics(ctx);', 'self.poll_storage_diagnostics(&ctx);'),
    ('self.render_storage_modal(ctx);', 'self.render_storage_modal(&ctx);'),
    ('self.render_save_modal(ctx);', 'self.render_save_modal(&ctx);'),
    ('self.render_matrix_builder_modal(ctx);', 'self.render_matrix_builder_modal(&ctx);'),
    ('self.render_domain_picker_modal(ctx);', 'self.render_domain_picker_modal(&ctx);'),
    ('self.render_branch_cut_modal(ctx);', 'self.render_branch_cut_modal(&ctx);'),
    ('self.render_ode_wizard_modal(ctx);', 'self.render_ode_wizard_modal(&ctx);'),
    ('self.render_units_palette_modal(ctx);', 'self.render_units_palette_modal(&ctx);'),
    ('self.render_example_gallery_modal(ctx);', 'self.render_example_gallery_modal(&ctx);'),
    ('.show(ctx, |ui|', '.show(&ctx, |ui|')
]

for old, new in replacements:
    content = content.replace(old, new)

with open('crates/urae-notebook/src/app.rs', 'w', encoding='utf-8') as f:
    f.write(content)
