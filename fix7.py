import re
with open('crates/urae-notebook/src/app.rs', 'r', encoding='utf-8') as f:
    content = f.read()

content = content.replace('let ctx = ui.ctx();', 'let ctx = ui.ctx().clone();')
content = content.replace('self.render_help_modal(ctx)', 'self.render_help_modal(&ctx)')
content = content.replace('self.render_popped_out_windows(ctx)', 'self.render_popped_out_windows(&ctx)')
content = content.replace('self.render_cli_terminal_contents(ctx, ui)', 'self.render_cli_terminal_contents(&ctx, ui)')
content = content.replace('if self.render_smart_stream_view(ui, ctx) {', 'if self.render_smart_stream_view(ui, &ctx) {')

with open('crates/urae-notebook/src/app.rs', 'w', encoding='utf-8') as f:
    f.write(content)
