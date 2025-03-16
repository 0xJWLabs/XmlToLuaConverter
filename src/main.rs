use egui::{Color32, RichText, Stroke};
use quick_xml::Reader;
use quick_xml::events::Event;
use std::collections::BTreeMap;
use std::fs::File;
use std::io::Read;

#[derive(Debug)]
struct CharacterOffset {
    x: i32,
    y: i32,
}

#[derive(Debug)]
struct CharacterSize {
    width: i32,
    height: i32,
}

#[derive(Debug)]
struct CharacterPosition {
    x: i32,
    y: i32,
}

#[derive(Debug)]
struct Character {
    size: CharacterSize,
    position: CharacterPosition,
    offset: CharacterOffset,
    advance: i32,
}

fn parse_fnt(
    filename: &str,
) -> Result<(i32, BTreeMap<u32, Character>), Box<dyn std::error::Error>> {
    let mut file = File::open(filename)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    let mut reader = Reader::from_str(&contents);
    let mut characters = BTreeMap::new();
    let mut buf = Vec::new();
    let mut font_size = 0;

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Eof) => break,
            Ok(Event::Empty(ref e)) if e.name().as_ref() == b"char" => {
                let mut id = 0;
                let mut width = 0;
                let mut height = 0;
                let mut x = 0;
                let mut y = 0;
                let mut xoffset = 0;
                let mut yoffset = 0;
                let mut xadvance = 0;

                for attr in e.attributes() {
                    let attr = attr?;
                    match attr.key.as_ref() {
                        b"id" => id = std::str::from_utf8(&attr.value)?.parse()?,
                        b"x" => x = std::str::from_utf8(&attr.value)?.parse()?,
                        b"y" => y = std::str::from_utf8(&attr.value)?.parse()?,
                        b"width" => width = std::str::from_utf8(&attr.value)?.parse()?,
                        b"height" => height = std::str::from_utf8(&attr.value)?.parse()?,
                        b"xoffset" => xoffset = std::str::from_utf8(&attr.value)?.parse()?,
                        b"yoffset" => yoffset = std::str::from_utf8(&attr.value)?.parse()?,
                        b"xadvance" => xadvance = std::str::from_utf8(&attr.value)?.parse()?,
                        _ => {}
                    }
                }

                characters.insert(
                    id,
                    Character {
                        size: CharacterSize { width, height },
                        position: CharacterPosition { x, y },
                        offset: CharacterOffset {
                            x: xoffset,
                            y: yoffset,
                        },
                        advance: xadvance,
                    },
                );
            }
            Ok(Event::Empty(ref e)) if e.name().as_ref() == b"info" => {
                for attr in e.attributes() {
                    let attr = attr?;
                    if attr.key.as_ref() == b"size" {
                        font_size = std::str::from_utf8(&attr.value)?.parse()?;
                    }
                }
            }
            Err(e) => {
                eprintln!("Error parsing XML: {:?}", e);
                break;
            }
            _ => {}
        }
        buf.clear();
    }

    Ok((font_size, characters))
}

fn format_output(font_size: i32, font_data: &BTreeMap<u32, Character>) -> String {
    let indentation = 4;
    let spaces = " ".repeat(indentation);
    let mut output = format!("return {{\n{spaces}Size = {font_size},\n{spaces}Characters = {{\n");

    for (id, data) in font_data {
        let char_repr = match *id {
            0 | 13 => "".to_string(),
            _ => match std::char::from_u32(*id) {
                Some(c) if c == '"' => "\\\"".to_string(), // Escape double quotes
                Some(c) if c == '\\' => "\\\\".to_string(), // Escape backslashes
                Some(c) if c.is_control() => format!("\\u{{{:X}}}", id),
                Some(c) => c.to_string(),
                None => format!("\\u{{{:X}}}", id),
            },
        };

        output.push_str(&format!(
            "{spaces}{spaces}[\"{}\"] = {{ Vector2.new({}, {}), Vector2.new({}, {}), Vector2.new({}, {}), {} }},\n",
            char_repr, data.size.width, data.size.height, data.position.x, data.position.y, data.offset.x, data.offset.y, data.advance
        ));
    }

    output.push_str(&format!("{spaces}}}\n}}\n"));
    output
}

struct ParsingStatus {
    message: String,
    status: Option<String>
}

struct FontParserApp {
    selected_file: Option<String>,
    status: ParsingStatus
}

impl Default for FontParserApp {
    fn default() -> Self {
        Self {
            selected_file: None,
            status: ParsingStatus {
                message: String::new(),
                status: None
            }
        }
    }
}

impl eframe::App for FontParserApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.style_mut(|style| {
            style.visuals.override_text_color = Some(Color32::from_rgb(204, 214, 244));
            style.visuals.panel_fill = Color32::from_rgb(17, 17, 27);
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("🎨 .fnt to .lua Converter");
            ui.separator();

            let response = ui.add(
                egui::Button::new(
                    egui::RichText::new("📂 Select .fnt file")
                        .size(12.0)
                        .color(Color32::from_rgb(204, 214, 244)),
                )
                .corner_radius(4.0)
                .fill(Color32::from_rgb(17, 17, 27)) // Default background color
                .stroke(Stroke::new(1.0, Color32::from_rgb(49, 50, 68))) // Default border
            );
            
            if response.hovered() {
                // Re-render the button with the hover styles
                ui.painter().rect_filled(
                    response.rect,
                    4.0,
                    Color32::from_rgb(137, 180, 250), // Hover background
                );
            
                ui.painter().text(
                    response.rect.center(),
                    egui::Align2::CENTER_CENTER,
                    "📂 Select .fnt file",
                    egui::FontId::proportional(12.0),
                    Color32::from_rgb(17, 17, 27), // Hover text color
                );

                ui.painter().rect_stroke(
                    response.rect,
                    4.0,
                    Stroke::new(1.0, Color32::from_rgb(137, 180, 250)),
                    egui::StrokeKind::Outside
                );
            }

            if response.clicked() {
                if let Some(path) = rfd::FileDialog::new()
                    .add_filter("FNT files", &["fnt"])
                    .pick_file()
                {
                    self.selected_file = Some(path.display().to_string());
                    self.status.message.clear();
                    self.status.status = None;
                }
            }

            if let Some(ref file) = self.selected_file {
                ui.label(format!("📄 Selected: {}", file));
            }

            let convert_button = ui.add(
                egui::Button::new(
                    egui::RichText::new("⚡ Convert")
                        .size(20.0) // Larger text
                        .color(Color32::from_rgb(17, 17, 27)), // Dark text
                )
                .corner_radius(8.0)
                .fill(Color32::from_rgb(137, 180, 250)) // Gradient-like blue
            );
            
            // Hover effect
            if convert_button.hovered() {
                ui.painter().rect_filled(
                    convert_button.rect,
                    8.0,
                    Color32::from_rgb(203, 166, 247), // Lighter blue on hover
                );

                ui.painter().text(
                    convert_button.rect.center(),
                    egui::Align2::CENTER_CENTER,
                    "⚡ Convert",
                    egui::FontId::proportional(20.0),
                    Color32::from_rgb(17, 17, 27), // Hover text color
                );
            }

            if convert_button.clicked() {
                if let Some(ref file) = self.selected_file {
                    if let Ok((font_size, font_data)) = parse_fnt(file) {
                        if let Some(output_file) = rfd::FileDialog::new()
                            .add_filter("Lua files", &["lua"])
                            .save_file()
                        {
                            match std::fs::write(&output_file, format_output(font_size, &font_data)) {
                                Ok(_) => {
                                    self.status.message = format!("✅ Saved to {}", output_file.display());
                                    self.status.status = Some("success".to_string());
                                }
                                Err(e) => {
                                    self.status.message = format!("❌ Error saving file: {}", e);
                                    self.status.status = Some("error".to_string());
                                }
                            }
                        }
                    } else {
                        self.status.message = "❌ Error parsing file!".to_string();
                        self.status.status = Some("error".to_string());
                    }
                } else {
                    self.status.message = "⚠️ Please select a .fnt file first".to_string();
                    self.status.status = Some("warning".to_string());
                }
            }

            if !self.status.message.is_empty() {
                let message = RichText::new(self.status.message.clone())
                    .color(match self.status.status.as_deref() {
                        Some("success") => Color32::from_rgb(166, 227, 161),
                        Some("error") => Color32::from_rgb(243, 139, 168),
                        Some("warning") => Color32::from_rgb(249, 226, 175),
                        _ => Color32::from_rgb(204, 214, 244),
                    });
                ui.label(message.clone());
            }
        });
    }
}

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([300.0, 200.0]).with_title("Converter"), // Smaller window
        ..Default::default()
    };

    eframe::run_native(
        "Converter",
        options,
        Box::new(|_cc| Ok(Box::new(FontParserApp::default()))),
    )
}
