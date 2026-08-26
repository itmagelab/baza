use crate::BazaR;

pub async fn print_qr(name: String) -> BazaR<()> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        use crate::storage::get_content;
        use colored::Colorize;

        let content = get_content(&name).await?;
        let first_line = content.lines().next().unwrap_or("").trim();

        if first_line.is_empty() {
            return Err(crate::error::Error::Message("Content is empty".to_string()).into());
        }

        println!("\n{}\n", "Scan this QR code:".bright_yellow().bold());
        let code = qrcode::QrCode::new(first_line.as_bytes()).map_err(|e| {
            crate::error::Error::Message(format!("Failed to generate QR code: {}", e))
        })?;
        let image = code
            .render::<qrcode::render::unicode::Dense1x2>()
            .dark_color(qrcode::render::unicode::Dense1x2::Light)
            .light_color(qrcode::render::unicode::Dense1x2::Dark)
            .build();
        println!("{}", image);
    }
    #[cfg(target_arch = "wasm32")]
    {
        // QR rendering to console is not supported in WASM
        return Err(
            crate::error::Error::Message("Not implemented for this platform".into()).into(),
        );
    }
    Ok(())
}
