//! Engine spike: one Markdown widget and one Rust code widget on the OpenTUI engine. Quit with `q`.

use opentui::prelude::*;
use opentui::widget::{Code, Markdown};

const NOTES: &str = "# The Marauder's Map\n\nA **magical** map of `Hogwarts`, drawn by Moony, Wormtail, Padfoot, and Prongs.\n\n\
- [x] Find the Room of Requirement\n- [ ] Return the map to Harry Potter\n\n\
| Room | Floor |\n|---|---|\n| Great Hall | 1 |\n| Gryffindor Tower | 7 |\n\n\
> I solemnly swear that I am up to no good.\n";

const SPELL: &str = "/// Opens the vault at Gringotts.\nfn open_vault(number: u32) -> Result<String, String> {\n    \
if number == 713 {\n        return Err(\"Only Albus Dumbledore may open it\".into());\n    }\n    \
Ok(format!(\"Vault {number} is open\"))\n}\n";

fn main() -> opentui::Result<()> {
    let app = App::new(AppOptions::default())?;
    let notes = Markdown::new(&app).set_markdown(NOTES);
    let spell = Code::new(&app).filetype("rust").line_numbers(true).set_code(SPELL);
    app.root().add(&notes).add(&spell);
    app.on_key(|cx, key| {
        if key.matches("q") {
            cx.quit();
        }
    });
    app.run()
}
