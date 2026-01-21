// Waveless
// Copyright (C) 2026 Oscar Alvarez Gonzalez

use crate::*;

/// An exit handler to prettify success and error contexts.
/// TODO: Replace this with custom error types.
pub async fn handle_main<T>(main_fn: T)
where
    T: AsyncFn() -> Result<CompactString>,
{
    match main_fn().await {
        Ok(res) => {
            if !res.is_empty() {
                element! {
                View(
                    padding_left: 1,
                    padding_right: 1,
                    border_style: BorderStyle::Round,
                    border_color: iocraft::Color::Green,
                ) {
                    MixedText(align: TextAlign::Left, contents: vec![
                        MixedTextContent::new("✅ "),
                        MixedTextContent::new("SUCCESS: ").color(iocraft::Color::Green).weight(Weight::Bold),
                        MixedTextContent::new(res).color(iocraft::Color::White),
                    ])
                }
            }
            .print();
            }
        }
        Err(err) => {
            let err = err.to_string();
            let (res, cx) = err.split_once("%").unwrap_or((err.as_str(), "")); // TODO: This should be a custom error type.
            element! {
                View(
                    padding_left: 1,
                    padding_right: 1,
                    border_style: BorderStyle::Round,
                    border_color: iocraft::Color::Red,
                ) {
                    MixedText(align: TextAlign::Left, contents: vec![
                        MixedTextContent::new("🔴 "),
                        MixedTextContent::new("ERROR: ").color(iocraft::Color::Red).weight(Weight::Bold),
                        MixedTextContent::new(res).color(iocraft::Color::White),
                        MixedTextContent::new(format!("\n{}", cx)).color(iocraft::Color::Blue),
                    ])
                }
            }
            .print();
        }
    }
}
