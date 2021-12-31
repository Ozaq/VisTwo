use imgui::ChildWindow;
use imgui::Condition;
use imgui::InputTextFlags;
use imgui::Ui;
use imgui::Window;

#[derive(Debug)]
pub struct Console {
    input: String,
    history: Vec<String>,
    refocus: bool,
}

impl Console {
    pub fn new() -> Self {
        Self {
            input: String::with_capacity(128),
            history: Vec::new(),
            refocus: true,
        }
    }

    pub fn draw(&mut self, ui: &Ui) {
        if let Some(window) = Window::new("Console")
            .size([800.0, 300.0], Condition::Always)
            .collapsible(false)
            .begin(ui)
        {
            if let Some(child) = ChildWindow::new("console_history")
                .size([0.0, -24.0])
                .border(true)
                .begin(ui)
            {
                for line in &self.history {
                    ui.text(&line);
                }
            }
            if self.refocus {
                ui.set_keyboard_focus_here();
            }
            if ui
                .input_text("Run Command", &mut self.input)
                .flags(InputTextFlags::ENTER_RETURNS_TRUE | InputTextFlags::ALWAYS_OVERWRITE)
                .hint("Your command...")
                .build()
            {
                self.history.push(self.input.clone());
                self.input.clear();
                self.refocus = true;
            } else {
                self.refocus = false;
            }
        }
    }
}
