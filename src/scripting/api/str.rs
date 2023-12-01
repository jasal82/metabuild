use base64::{engine::general_purpose, Engine as _};
use colored::{Color, ColoredString, Colorize};
use rune::runtime::{Formatter, VmResult};
use rune::{Any, ContextError, Module};
use rune::alloc::fmt::TryWrite;
use std::collections::HashMap;
use tera::{Context, Tera};

enum Source {
    File(String),
    Template(String),
}

fn template_internal(tpl: Source, context: HashMap<String, String>) -> Result<String, anyhow::Error> {
    let mut tera = Tera::default();
    match tpl {
        Source::File(file) => tera.add_template_file(file, Some("template"))?,
        Source::Template(tpl) => tera.add_raw_template("template", tpl.as_str())?,
    }
    let mut tera_context = Context::new();
    for (k, v) in context {
        tera_context.insert(k, &v);
    }
    tera.render("template", &tera_context).map_err(|e| e.into())
}

#[rune::function]
pub fn template_file(tpl_file: &str, context: HashMap<String, String>) -> Result<String, anyhow::Error> {
    template_internal(Source::File(tpl_file.to_string()), context)
}

#[rune::function]
pub fn template(tpl: &str, context: HashMap<String, String>) -> Result<String, anyhow::Error> {
    template_internal(Source::Template(tpl.to_string()), context)
}

#[rune::function]
pub fn encode_base64(data: &str) -> String {
    general_purpose::STANDARD.encode(data)
}

#[derive(Any)]
#[rune(item = ::str)]
pub struct Painter {
    text: ColoredString,
}

impl Painter {
    #[rune::function]
    pub fn black(self) -> Painter {
        Painter {
            text: self.text.color(Color::Black),
        }
    }

    #[rune::function]
    pub fn red(self) -> Painter {
        Painter {
            text: self.text.color(Color::Red),
        }
    }

    #[rune::function]
    pub fn green(self) -> Painter {
        Painter {
            text: self.text.color(Color::Green),
        }
    }

    #[rune::function]
    pub fn yellow(self) -> Painter {
        Painter {
            text: self.text.color(Color::Yellow),
        }
    }

    #[rune::function]
    pub fn blue(self) -> Painter {
        Painter {
            text: self.text.color(Color::Blue),
        }
    }

    #[rune::function]
    pub fn magenta(self) -> Painter {
        Painter {
            text: self.text.color(Color::Magenta),
        }
    }

    #[rune::function]
    pub fn cyan(self) -> Painter {
        Painter {
            text: self.text.color(Color::Cyan),
        }
    }

    #[rune::function]
    pub fn white(self) -> Painter {
        Painter {
            text: self.text.color(Color::White),
        }
    }

    #[rune::function]
    pub fn bright_black(self) -> Painter {
        Painter {
            text: self.text.color(Color::BrightBlack),
        }
    }

    #[rune::function]
    pub fn bright_red(self) -> Painter {
        Painter {
            text: self.text.color(Color::BrightRed),
        }
    }

    #[rune::function]
    pub fn bright_green(self) -> Painter {
        Painter {
            text: self.text.color(Color::BrightGreen),
        }
    }

    #[rune::function]
    pub fn bright_yellow(self) -> Painter {
        Painter {
            text: self.text.color(Color::BrightYellow),
        }
    }

    #[rune::function]
    pub fn bright_blue(self) -> Painter {
        Painter {
            text: self.text.color(Color::BrightBlue),
        }
    }

    #[rune::function]
    pub fn bright_magenta(self) -> Painter {
        Painter {
            text: self.text.color(Color::BrightMagenta),
        }
    }

    #[rune::function]
    pub fn bright_cyan(self) -> Painter {
        Painter {
            text: self.text.color(Color::BrightCyan),
        }
    }

    #[rune::function]
    pub fn bright_white(self) -> Painter {
        Painter {
            text: self.text.color(Color::BrightWhite),
        }
    }

    #[rune::function]
    pub fn on_black(self) -> Painter {
        Painter {
            text: self.text.on_color(Color::Black),
        }
    }

    #[rune::function]
    pub fn on_red(self) -> Painter {
        Painter {
            text: self.text.on_color(Color::Red),
        }
    }

    #[rune::function]
    pub fn on_green(self) -> Painter {
        Painter {
            text: self.text.on_color(Color::Green),
        }
    }

    #[rune::function]
    pub fn on_yellow(self) -> Painter {
        Painter {
            text: self.text.on_color(Color::Yellow),
        }
    }

    #[rune::function]
    pub fn on_blue(self) -> Painter {
        Painter {
            text: self.text.on_color(Color::Blue),
        }
    }

    #[rune::function]
    pub fn on_magenta(self) -> Painter {
        Painter {
            text: self.text.on_color(Color::Magenta),
        }
    }

    #[rune::function]
    pub fn on_cyan(self) -> Painter {
        Painter {
            text: self.text.on_color(Color::Cyan),
        }
    }

    #[rune::function]
    pub fn on_white(self) -> Painter {
        Painter {
            text: self.text.on_color(Color::White),
        }
    }

    #[rune::function]
    pub fn on_bright_black(self) -> Painter {
        Painter {
            text: self.text.on_color(Color::BrightBlack),
        }
    }

    #[rune::function]
    pub fn on_bright_red(self) -> Painter {
        Painter {
            text: self.text.on_color(Color::BrightRed),
        }
    }

    #[rune::function]
    pub fn on_bright_green(self) -> Painter {
        Painter {
            text: self.text.on_color(Color::BrightGreen),
        }
    }

    #[rune::function]
    pub fn on_bright_yellow(self) -> Painter {
        Painter {
            text: self.text.on_color(Color::BrightYellow),
        }
    }

    #[rune::function]
    pub fn on_bright_blue(self) -> Painter {
        Painter {
            text: self.text.on_color(Color::BrightBlue),
        }
    }

    #[rune::function]
    pub fn on_bright_magenta(self) -> Painter {
        Painter {
            text: self.text.on_color(Color::BrightMagenta),
        }
    }

    #[rune::function]
    pub fn on_bright_cyan(self) -> Painter {
        Painter {
            text: self.text.on_color(Color::BrightCyan),
        }
    }

    #[rune::function]
    pub fn on_bright_white(self) -> Painter {
        Painter {
            text: self.text.on_color(Color::BrightWhite),
        }
    }

    #[rune::function]
    pub fn clear(self) -> Painter {
        Painter {
            text: self.text.clear(),
        }
    }

    #[rune::function]
    pub fn normal(self) -> Painter {
        Painter {
            text: self.text.normal(),
        }
    }

    #[rune::function]
    pub fn bold(self) -> Painter {
        Painter {
            text: self.text.bold(),
        }
    }

    #[rune::function]
    pub fn dimmed(self) -> Painter {
        Painter {
            text: self.text.dimmed(),
        }
    }

    #[rune::function]
    pub fn italic(self) -> Painter {
        Painter {
            text: self.text.italic(),
        }
    }

    #[rune::function]
    pub fn underline(self) -> Painter {
        Painter {
            text: self.text.underline(),
        }
    }

    #[rune::function]
    pub fn blink(self) -> Painter {
        Painter {
            text: self.text.blink(),
        }
    }

    #[rune::function]
    pub fn reversed(self) -> Painter {
        Painter {
            text: self.text.reverse(),
        }
    }

    #[rune::function]
    pub fn hidden(self) -> Painter {
        Painter {
            text: self.text.hidden(),
        }
    }

    #[rune::function]
    pub fn strikethrough(self) -> Painter {
        Painter {
            text: self.text.strikethrough(),
        }
    }

    #[rune::function(protocol = STRING_DISPLAY)]
    pub fn string_display(&self, f: &mut Formatter) -> VmResult<()> {
        rune::vm_write!(f, "{}", self.text);
        VmResult::Ok(())
    }
}

#[rune::function]
pub fn paint(text: &str) -> Painter {
    Painter { text: text.into() }
}

pub fn module() -> Result<Module, ContextError> {
    let mut module = Module::with_crate("str")?;

    module.function_meta(template_file)?;
    module.function_meta(template)?;
    module.function_meta(encode_base64)?;

    module.ty::<Painter>()?;

    module.function_meta(paint)?;
    module.function_meta(Painter::black)?;
    module.function_meta(Painter::red)?;
    module.function_meta(Painter::green)?;
    module.function_meta(Painter::yellow)?;
    module.function_meta(Painter::blue)?;
    module.function_meta(Painter::magenta)?;
    module.function_meta(Painter::cyan)?;
    module.function_meta(Painter::white)?;
    module.function_meta(Painter::bright_black)?;
    module.function_meta(Painter::bright_red)?;
    module.function_meta(Painter::bright_green)?;
    module.function_meta(Painter::bright_yellow)?;
    module.function_meta(Painter::bright_blue)?;
    module.function_meta(Painter::bright_magenta)?;
    module.function_meta(Painter::bright_cyan)?;
    module.function_meta(Painter::bright_white)?;
    module.function_meta(Painter::on_black)?;
    module.function_meta(Painter::on_red)?;
    module.function_meta(Painter::on_green)?;
    module.function_meta(Painter::on_yellow)?;
    module.function_meta(Painter::on_blue)?;
    module.function_meta(Painter::on_magenta)?;
    module.function_meta(Painter::on_cyan)?;
    module.function_meta(Painter::on_white)?;
    module.function_meta(Painter::on_bright_black)?;
    module.function_meta(Painter::on_bright_red)?;
    module.function_meta(Painter::on_bright_green)?;
    module.function_meta(Painter::on_bright_yellow)?;
    module.function_meta(Painter::on_bright_blue)?;
    module.function_meta(Painter::on_bright_magenta)?;
    module.function_meta(Painter::on_bright_cyan)?;
    module.function_meta(Painter::on_bright_white)?;
    module.function_meta(Painter::clear)?;
    module.function_meta(Painter::normal)?;
    module.function_meta(Painter::bold)?;
    module.function_meta(Painter::dimmed)?;
    module.function_meta(Painter::italic)?;
    module.function_meta(Painter::underline)?;
    module.function_meta(Painter::blink)?;
    module.function_meta(Painter::reversed)?;
    module.function_meta(Painter::hidden)?;
    module.function_meta(Painter::strikethrough)?;
    module.function_meta(Painter::string_display)?;

    Ok(module)
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_template() {
        
    }
}