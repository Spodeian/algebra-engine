import re
with open('crates/urae-notebook/src/app.rs', 'r', encoding='utf-8') as f:
    content = f.read()

# Fix screen_rect
content = content.replace('let rect = ctx.screen_rect();', 'let rect = ctx.input(|i| i.screen_rect);')
content = content.replace('let rect = ui.ctx().screen_rect();', 'let rect = ui.ctx().input(|i| i.screen_rect);')

# Fix Vec2 from default_size
content = content.replace('.default_size(320.0)', '.default_size([320.0, 320.0])')
content = content.replace('.default_size(280.0)', '.default_size([280.0, 280.0])')

# Fix Panel default_size to use Vec2 if needed. Wait, Panel::left... default_size takes f32! No, Panel didn't have a problem with default_size. The errors were ONLY for Window::default_size at 3917 and 5021!

with open('crates/urae-notebook/src/app.rs', 'w', encoding='utf-8') as f:
    f.write(content)
