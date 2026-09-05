import re
with open('crates/urae-notebook/src/app.rs', 'r', encoding='utf-8') as f:
    content = f.read()

content = content.replace('i.screen_rect', 'i.screen_rect()')
content = content.replace('ui.ctx().set_style(self.theme.to_style());', 'ui.ctx().set_style_of(ui.ctx().theme(), self.theme.to_style());')

with open('crates/urae-notebook/src/app.rs', 'w', encoding='utf-8') as f:
    f.write(content)
