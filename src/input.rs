use std::io::{self, IsTerminal, Read};

pub fn read_stdin_if_available() -> io::Result<Option<String>> {
    if io::stdin().is_terminal() {
        return Ok(None);
    }
    let text = read_all_stdin()?;
    if text.trim().is_empty() {
        Ok(None)
    } else {
        Ok(Some(text))
    }
}

pub fn read_all_stdin() -> io::Result<String> {
    let mut text = String::new();
    io::stdin().read_to_string(&mut text)?;
    Ok(text)
}

pub fn stdin_is_terminal() -> bool {
    io::stdin().is_terminal()
}
