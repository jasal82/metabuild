use colored::{Color, ColoredString, Colorize};
use rune::{Any, ContextError, Module};
use rune::runtime::{Object, Protocol};
use std::fmt;
use std::fmt::Write;
use tera::{Tera, Context};

enum Source {
    File(String),
    Template(String),
}

fn template_internal(tpl: Source, context: Object) -> String {
    let mut tera = Tera::default();
    match tpl {
        Source::File(file) => tera.add_template_file(file, Some("template")).unwrap(),
        Source::Template(tpl) => tera.add_raw_template("template", &tpl).unwrap(),
    }
    let mut tera_context = Context::new();
    for (k, v) in context {
        tera_context.insert(k, v.into_string().unwrap().borrow_ref().unwrap().as_str());
    }
    tera.render("template", &tera_context).unwrap()
}

pub fn template_file(tpl_file: &str, context: Object) -> String {
    template_internal(Source::File(tpl_file.to_string()), context)
}

pub fn template(tpl: &str, context: Object) -> String {
    template_internal(Source::Template(tpl.to_string()), context)
}

#[derive(Any)]
struct Painter {
    text: ColoredString,
}

impl Painter {
    pub fn new(text: &str) -> Self {
        Self {
            text: text.into(),
        }
    }

    pub fn black(self) -> Painter {
        Painter {
            text: self.text.color(Color::Black),
        }
    }

    pub fn red(self) -> Painter {
        Painter {
            text: self.text.color(Color::Red),
        }
    }

    pub fn green(self) -> Painter {
        Painter {
            text: self.text.color(Color::Green),
        }
    }

    pub fn yellow(self) -> Painter {
        Painter {
            text: self.text.color(Color::Yellow),
        }
    }

    pub fn blue(self) -> Painter {
        Painter {
            text: self.text.color(Color::Blue),
        }
    }

    pub fn magenta(self) -> Painter {
        Painter {
            text: self.text.color(Color::Magenta),
        }
    }

    pub fn cyan(self) -> Painter {
        Painter {
            text: self.text.color(Color::Cyan),
        }
    }

    pub fn white(self) -> Painter {
        Painter {
            text: self.text.color(Color::White),
        }
    }

    pub fn bright_black(self) -> Painter {
        Painter {
            text: self.text.color(Color::BrightBlack),
        }
    }

    pub fn bright_red(self) -> Painter {
        Painter {
            text: self.text.color(Color::BrightRed),
        }
    }

    pub fn bright_green(self) -> Painter {
        Painter {
            text: self.text.color(Color::BrightGreen),
        }
    }

    pub fn bright_yellow(self) -> Painter {
        Painter {
            text: self.text.color(Color::BrightYellow),
        }
    }

    pub fn bright_blue(self) -> Painter {
        Painter {
            text: self.text.color(Color::BrightBlue),
        }
    }

    pub fn bright_magenta(self) -> Painter {
        Painter {
            text: self.text.color(Color::BrightMagenta),
        }
    }

    pub fn bright_cyan(self) -> Painter {
        Painter {
            text: self.text.color(Color::BrightCyan),
        }
    }

    pub fn bright_white(self) -> Painter {
        Painter {
            text: self.text.color(Color::BrightWhite),
        }
    }

    pub fn on_black(self) -> Painter {
        Painter {
            text: self.text.on_color(Color::Black),
        }
    }

    pub fn on_red(self) -> Painter {
        Painter {
            text: self.text.on_color(Color::Red),
        }
    }

    pub fn on_green(self) -> Painter {
        Painter {
            text: self.text.on_color(Color::Green),
        }
    }

    pub fn on_yellow(self) -> Painter {
        Painter {
            text: self.text.on_color(Color::Yellow),
        }
    }

    pub fn on_blue(self) -> Painter {
        Painter {
            text: self.text.on_color(Color::Blue),
        }
    }

    pub fn on_magenta(self) -> Painter {
        Painter {
            text: self.text.on_color(Color::Magenta),
        }
    }

    pub fn on_cyan(self) -> Painter {
        Painter {
            text: self.text.on_color(Color::Cyan),
        }
    }

    pub fn on_white(self) -> Painter {
        Painter {
            text: self.text.on_color(Color::White),
        }
    }

    pub fn on_bright_black(self) -> Painter {
        Painter {
            text: self.text.on_color(Color::BrightBlack),
        }
    }

    pub fn on_bright_red(self) -> Painter {
        Painter {
            text: self.text.on_color(Color::BrightRed),
        }
    }

    pub fn on_bright_green(self) -> Painter {
        Painter {
            text: self.text.on_color(Color::BrightGreen),
        }
    }

    pub fn on_bright_yellow(self) -> Painter {
        Painter {
            text: self.text.on_color(Color::BrightYellow),
        }
    }

    pub fn on_bright_blue(self) -> Painter {
        Painter {
            text: self.text.on_color(Color::BrightBlue),
        }
    }

    pub fn on_bright_magenta(self) -> Painter {
        Painter {
            text: self.text.on_color(Color::BrightMagenta),
        }
    }

    pub fn on_bright_cyan(self) -> Painter {
        Painter {
            text: self.text.on_color(Color::BrightCyan),
        }
    }

    pub fn on_bright_white(self) -> Painter {
        Painter {
            text: self.text.on_color(Color::BrightWhite),
        }
    }

    pub fn clear(self) -> Painter {
        Painter {
            text: self.text.clear(),
        }
    }

    pub fn normal(self) -> Painter {
        Painter {
            text: self.text.normal(),
        }
    }

    pub fn bold(self) -> Painter {
        Painter {
            text: self.text.bold(),
        }
    }

    pub fn dimmed(self) -> Painter {
        Painter {
            text: self.text.dimmed(),
        }
    }

    pub fn italic(self) -> Painter {
        Painter {
            text: self.text.italic(),
        }
    }

    pub fn underline(self) -> Painter {
        Painter {
            text: self.text.underline(),
        }
    }

    pub fn blink(self) -> Painter {
        Painter {
            text: self.text.blink(),
        }
    }

    pub fn reversed(self) -> Painter {
        Painter {
            text: self.text.reverse(),
        }
    }

    pub fn hidden(self) -> Painter {
        Painter {
            text: self.text.hidden(),
        }
    }

    pub fn strikethrough(self) -> Painter {
        Painter {
            text: self.text.strikethrough(),
        }
    }

    pub fn display(&self, buf: &mut String) -> fmt::Result {
        write!(buf, "{}", self.text)
    }
}

pub fn module() -> Result<Module, ContextError> {
    let mut module = Module::with_crate("str");
    module.function(["template_file"], template_file)?;
    module.function(["template"], template)?;
    module.ty::<Painter>()?;
    module.function(["paint"], Painter::new)?;
    module.inst_fn("black", Painter::black)?;
    module.inst_fn("red", Painter::red)?;
    module.inst_fn("green", Painter::green)?;
    module.inst_fn("yellow", Painter::yellow)?;
    module.inst_fn("blue", Painter::blue)?;
    module.inst_fn("magenta", Painter::magenta)?;
    module.inst_fn("cyan", Painter::cyan)?;
    module.inst_fn("white", Painter::white)?;
    module.inst_fn("bright_black", Painter::bright_black)?;
    module.inst_fn("bright_red", Painter::bright_red)?;
    module.inst_fn("bright_green", Painter::bright_green)?;
    module.inst_fn("bright_yellow", Painter::bright_yellow)?;
    module.inst_fn("bright_blue", Painter::bright_blue)?;
    module.inst_fn("bright_magenta", Painter::bright_magenta)?;
    module.inst_fn("bright_cyan", Painter::bright_cyan)?;
    module.inst_fn("bright_white", Painter::bright_white)?;
    module.inst_fn("on_black", Painter::on_black)?;
    module.inst_fn("on_red", Painter::on_red)?;
    module.inst_fn("on_green", Painter::on_green)?;
    module.inst_fn("on_yellow", Painter::on_yellow)?;
    module.inst_fn("on_blue", Painter::on_blue)?;
    module.inst_fn("on_magenta", Painter::on_magenta)?;
    module.inst_fn("on_cyan", Painter::on_cyan)?;
    module.inst_fn("on_white", Painter::on_white)?;
    module.inst_fn("on_bright_black", Painter::on_bright_black)?;
    module.inst_fn("on_bright_red", Painter::on_bright_red)?;
    module.inst_fn("on_bright_green", Painter::on_bright_green)?;
    module.inst_fn("on_bright_yellow", Painter::on_bright_yellow)?;
    module.inst_fn("on_bright_blue", Painter::on_bright_blue)?;
    module.inst_fn("on_bright_magenta", Painter::on_bright_magenta)?;
    module.inst_fn("on_bright_cyan", Painter::on_bright_cyan)?;
    module.inst_fn("on_bright_white", Painter::on_bright_white)?;
    module.inst_fn("clear", Painter::clear)?;
    module.inst_fn("normal", Painter::normal)?;
    module.inst_fn("bold", Painter::bold)?;
    module.inst_fn("dimmed", Painter::dimmed)?;
    module.inst_fn("italic", Painter::italic)?;
    module.inst_fn("underline", Painter::underline)?;
    module.inst_fn("blink", Painter::blink)?;
    module.inst_fn("reversed", Painter::reversed)?;
    module.inst_fn("hidden", Painter::hidden)?;
    module.inst_fn("strikethrough", Painter::strikethrough)?;
    module.inst_fn(Protocol::STRING_DISPLAY, Painter::display)?;
    Ok(module)
}